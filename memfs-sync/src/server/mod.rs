#![allow(unused)]

mod errors;
pub use self::errors::*;
use std::collections::HashSet;

use nine::p2000::*;
use nine::ser;
use std::collections::hash_map::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::iter::FusedIterator;
use crate::utils::atomic_maybe_change;

#[derive(Clone)]
pub struct CommonFileMetaData {
    pub mode: FileMode,
    pub version: u32,
    pub path: u64,
    pub atime: u32,
    pub mtime: u32,
    pub name: String,
    pub uid: String,
    pub gid: String,
    pub muid: String,
}

// TODO: reconsider / compare / benchmark having paths versus pointers (rcs or Arcs)

pub struct User {
    pub user: String,
    pub group: String,
}

pub struct File {
    pub meta: CommonFileMetaData,
    /// byte content for file, stat_bytes for dir
    pub content: Vec<u8>,
    pub parent: u64,
    /// a set of child paths
    pub children: HashSet<u64>,
}

impl File {
    fn new(meta: CommonFileMetaData, parent: u64) -> File {
        let dir = meta.mode.contains(FileMode::DIR);
        File {
            meta,
            content: Vec::new(),
            parent,
            children: HashSet::new(),
        }
    }

    fn is_dir(&self) -> bool {
        self.meta.mode.contains(FileMode::DIR)
    }

    fn is_file(&self) -> bool {
        !self.is_dir()
    }

    fn qid(&self) -> Qid {
        Qid {
            file_type: self.meta.mode.into(),
            version: self.meta.version,
            path: self.meta.path,
        }
    }

    fn stat(&self) -> Stat {
        Stat {
            type_: 1,
            dev: 2,
            qid: Qid {
                file_type: self.meta.mode.into(),
                version: self.meta.version,
                path: self.meta.path,
            },
            mode: self.meta.mode,
            atime: self.meta.atime,
            mtime: self.meta.mtime,
            length: if self.is_file() {
                self.content.len() as u64
            } else {
                0
            },
            name: self.meta.name.clone().into(),
            uid: self.meta.uid.clone().into(),
            gid: self.meta.gid.clone().into(),
            muid: self.meta.muid.clone().into(),
        }
    }

    // TODO: exec
    fn open(&mut self, user: &User, mode: OpenMode) -> NineResult<()> {
        // TODO: group
        let user = &user.user;

        if self.is_dir() {
            if mode.is_writable() {
                return rerr("can't write to a directory");
            }
            if mode.contains(OpenMode::TRUNC) {
                return rerr("can't truncate a directory");
            }
            if mode.contains(OpenMode::CLOSE) {
                return rerr("can't remove dir on close");
            }

            let stat = self.stat();
            if mode.is_readable() {
                if stat.readable_for(user) {
                    Ok(())
                } else {
                    rerr("not readable")
                }
            } else {
                unreachable!()
            }
        } else {
            let stat = self.stat();
            if mode.is_readable() && !stat.readable_for(user) {
                return rerr("file not readable");
            }
            if mode.is_writable() {
                if !stat.writable_for(user) {
                    return rerr("file not writable");
                }
                if mode.contains(OpenMode::TRUNC) {
                    self.meta.muid = user.to_string();
                    self.content.clear();
                }
            }

            Ok(())
        }
    }

    fn read(&mut self, offset: u64, buf: &mut [u8]) -> u32 {
        let mut cursor = Cursor::new(&self.content);
        cursor.seek(SeekFrom::Start(offset)).unwrap();
        cursor.read(buf).unwrap() as u32
    }

    fn write(&mut self, user: &str, offset: u64, buf: &[u8]) -> Option<(u32, bool)> {
        if self.is_dir() {
            return None;
        }

        let mut should_invalidate = false;
        if self.meta.muid != user {
            should_invalidate = true;
            self.meta.muid = user.to_string();
        }
        let start_len = self.content.len();

        let amt = {
            let mut cursor = Cursor::new(&mut self.content);
            cursor.seek(SeekFrom::Start(offset)).unwrap();
            cursor.write(buf).unwrap() as u32
        };

        should_invalidate = should_invalidate || start_len != self.content.len();

        Some((amt, should_invalidate))
    }

    fn children_of<'a>(
        &'a self,
        all_files: &'a HashMap<u64, File>,
    ) -> impl Iterator<Item = &'a File> {
        self.children
            .iter()
            .map(move |x| all_files.get(x).unwrap())
    }

    fn walk<'a, S, I>(
        &'a self,
        all_files: &'a HashMap<u64, File>,
        wname: I,
    ) -> impl FusedIterator<Item = &'a File>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        Walker {
            current_file: self,
            file_tree: all_files,
            wname: wname.fuse(),
        }.fuse()
    }
}

/// Root is always 0.
pub struct FileTree {
    pub last_path: u64,
    pub all_files: HashMap<u64, File>,
}

impl FileTree {
    pub fn stat(&self, path: u64) -> Option<Stat> {
        self.all_files.get(&path).map(|x| x.stat())
    }

    pub fn qid(&self, path: u64) -> Option<Qid> {
        self.all_files.get(&path).map(|x| x.qid())
    }

    pub fn open(&mut self, path: u64, user: &User, mode: OpenMode) -> NineResult<&File> {
        let mut file = self.all_files.get_mut(&path).unwrap();

        file.open(user, mode).unwrap();

        Ok(file)
    }

    pub fn create(
        &mut self,
        path: u64,
        user: impl Into<String> + AsRef<str>,
        name: impl Into<String> + AsRef<str>,
        perm: FileMode,
        _mode: OpenMode,
    ) -> NineResult<&File> {
        let new_file = {
            let file = self.all_files.get(&path).unwrap();
            if file.is_file() {
                return rerr("can't create under a file");
            }

            if file
                .children
                .iter()
                .map(|x| self.all_files.get(x).unwrap())
                .any(|x| name.as_ref() == x.meta.name)
            {
                return rerr("a file with that name already exists");
            }

            // TODO: the fancy permissions stuff

            self.last_path += 1;

            let name = name.into();
            let uid = user.into();
            let muid = uid.clone();
            let gid = "some_group".into(); // TODO

            let meta = CommonFileMetaData {
                mode: perm,
                version: 0,
                path: self.last_path,
                atime: 0, // TODO
                mtime: 0, // TODO
                name,
                uid,
                muid,
                gid,
            };

            File::new(meta, file.meta.path)
        };

        let new_path = new_file.meta.path;

        {
            let file = self.all_files.get_mut(&path).unwrap();

            file.children.insert(new_path);
            file.content.clear();
        }

        self.all_files.insert(new_path, new_file);

        Ok(self.all_files.get(&new_path).unwrap())
    }

    fn read(&mut self, path: u64, offset: u64, buf: &mut [u8]) -> NineResult<u32> {
        let maybe_new_content = {
            let file = self.all_files.get(&path).unwrap();

            if file.is_dir() && file.children.len() != 0 && file.content.len() == 0 {
                let mut content = Vec::new();
                for stat in file.children
                    .iter()
                    .map(|x| self.all_files.get(x).unwrap().stat())
                {
                    ser::append_vec(&stat, &mut content).unwrap();
                }

                Some(content)
            } else {
                None
            }
           
        };

        let file = self.all_files.get_mut(&path).unwrap();

        if let Some(content) = maybe_new_content {
            file.content = content;
        }

        Ok(file.read(offset, buf))
    }

    fn write(&mut self, path: u64, user: &str, offset: u64, buf: &[u8]) -> NineResult<u32> {
        let ((amt, should_invalidate), parent) = {
            let file = self.all_files.get_mut(&path).unwrap();
            let res = file.write(user, offset, buf);
            if res.is_none() {
                return rerr("can't write to a directory");
            }
            (res.unwrap(), file.parent)
        };

        if should_invalidate {
            self.invalidate(parent);
        }

        Ok(amt)
    }

    fn invalidate(&mut self, path: u64) {
        self.all_files.get_mut(&path).unwrap().content.clear();
    }

    /// # Changable fields
    /// - Name: by anyone with write permission in the parent dir
    /// - Length: changes actual length of file
    /// - Mode, Mtime: byowner or group leader. Not the dir it
    /// - gid, owner if also a member of new group, or leader of current group
    ///
    /// "None of the other data can be altered by a wstat and attempts to change
    /// them will trigger an error. In particular, it is illegal to attempt to
    /// change the owner of a file.
    /// (These conditions may be relaxed when establishing the initial state of a
    /// file server; see Plan 9’s fsconfig(8).)"
    ///
    /// wat
    fn wstat(&mut self, path: u64, user: &str, wstat: Stat) -> NineResult<()> {
        let changing_length = wstat.length != u64::max_value();
        let (should_invalidate, parent) = {
            let file = self.all_files.get_mut(&path).unwrap();
            let parent = file.parent;
            let maybe_new_meta = {
                let is_owner = user == file.meta.uid;
                let is_dir = file.is_dir();
                let old_meta = &file.meta;
                let content = &mut file.content;
                atomic_maybe_change(old_meta, |new_meta| {
                    if wstat.mode.bits() != u32::max_value() {
                        if !is_owner {
                            return rerr("only the owner or group leader can chang a file's mode");
                        }
                        if wstat.mode.contains(FileMode::DIR) ^ is_dir {
                            return rerr("can't change dir bit");
                        }

                        new_meta.to_mut().mode = wstat.mode;
                    }

                    if wstat.mtime != u32::max_value() {
                        new_meta.to_mut().mtime = wstat.mtime;
                    }

                    // TODO: gid

                    if wstat.name.len() != 0 {
                        new_meta.to_mut().name = wstat.name
                    }

                    if is_dir && wstat.length != 0 && changing_length {
                        return rerr("can't set length of dir");
                    }
                    if !is_dir && changing_length {
                        content.resize(wstat.length as usize, 0);
                    }

                    Ok(())
                })?
            };

            if let Some(new_meta) = maybe_new_meta {
                file.meta = new_meta;
                (true, parent)
            } else if changing_length {
                (true, parent)
            } else {
                (false, parent)
            }
        };

        if should_invalidate {
            self.invalidate(parent)
        }
        Ok(())
    }

    fn remove(&mut self, path: u64, user: &User) {
        {
            let parent = self
                .all_files
                .get(&path)
                .expect("unknown path for file to remove")
                .parent;
            let parent = self.all_files.get_mut(&parent);

            if let Some(parent) = parent {
                parent.children.remove(&path);
            } else {
                panic!("file was orphaned");
            }
        }

        self.all_files.remove(&path);
    }

    fn children_of(&self, path: &u64) -> impl Iterator<Item = &File> {
        self.all_files
            .get(&path)
            .unwrap()
            .children_of(&self.all_files)
    }

    fn walk<S, I>(&self, wname: I) -> impl FusedIterator<Item = &File>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.walk_from(0, wname)
    }

    fn walk_from<S, I>(&self, path: u64, wname: I) -> impl FusedIterator<Item = &File>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        Walker {
            current_file: self.all_files.get(&path).unwrap(),
            file_tree: &self.all_files,
            wname: wname.into_iter().fuse(),
        }.fuse()
    }
}

struct Walker<'a, S, I>
where
    S: AsRef<str>,
    I: FusedIterator<Item = S>,
{
    current_file: &'a File,
    file_tree: &'a HashMap<u64, File>,
    wname: I,
}

// TODO: this is only correct if fused. Maybe hide the impl in a method on File
// and fuse it ourselves
impl<'a, S, I> Iterator for Walker<'a, S, I>
where
    S: AsRef<str>,
    I: FusedIterator<Item = S>,
{
    type Item = &'a File;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_file.is_file() {
            return None;
        }
        match self.wname.next().as_ref().map(|x| x.as_ref()) {
            None => None,
            Some(name) if name == ".." => {
                if self.current_file.meta.path != 0 {
                    self.current_file = self.file_tree.get(&self.current_file.parent).unwrap();
                }
                Some(self.current_file)
            }
            Some(name) => match self
                .current_file
                .children_of(self.file_tree)
                .find(|file| file.meta.name == name)
            {
                None => None,
                Some(file) => {
                    self.current_file = file;
                    Some(self.current_file)
                }
            },
        }
    }
}

#[derive(Clone)]
pub struct OpenView {
    pub mode: OpenMode,
    pub last_offset: u64,
}

#[derive(Clone)]
pub struct FileHandle {
    pub file: u64,
    pub open: Option<OpenView>,
}

impl FileHandle {
    pub fn new(file: u64) -> Self {
        FileHandle { file, open: None }
    }

    pub fn is_readable(&self, offset: u64, file_tree: &HashMap<u64, File>) -> bool {
        match self.open {
            None => false,
            Some(ref view) => if view.mode.is_readable() {
                let file = file_tree.get(&self.file).unwrap();
                file.is_file() || (view.last_offset == offset || offset == 0)
            } else {
                false
            },
        }
    }

    pub fn is_writable(&self, file_tree: &HashMap<u64, File>) -> bool {
        if let Some(ref view) = self.open {
            if view.mode.is_writable() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn create<'a>(
        &mut self,
        user: &str,
        name: &str,
        perm: FileMode,
        mode: OpenMode,
        tree: &'a mut FileTree,
    ) -> NineResult<&'a File> {
        if self.open.is_some() {
            return rerr("cannot create on an open fid");
        }

        let res = tree.create(self.file, user, name, perm, mode);

        if let Ok(file) = &res {
            self.file = file.meta.path;
            self.open = Some(OpenView {
                mode,
                last_offset: 0,
            });
        }

        res
    }

    fn open<'a>(
        &mut self,
        user: &User,
        mode: OpenMode,
        tree: &'a mut HashMap<u64, File>,
    ) -> NineResult<&'a File> {
        if self.open.is_some() {
            return rerr("file is already open");
        }

        let mut file = tree.get_mut(&self.file).unwrap();

        file.open(user, mode).unwrap();

        self.open = Some(OpenView {
            mode,
            last_offset: 0,
        });

        Ok(file)
    }
}

pub struct Session {
    pub fids: HashMap<u32, FileHandle>,
    pub tree: FileTree,
    pub user: User,
}

impl Session {
    pub fn walk<S, I>(&mut self, fid: u32, newfid: u32, wname: I) -> Vec<Qid>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S> + ExactSizeIterator,
    {
        let nwname = wname.len();
        let path = self.fids.get(&fid).unwrap().file;
        let wfiles: Vec<Qid> = self
            .tree
            .walk_from(path, wname)
            .map(|x| x.qid())
            .collect();

        if wfiles.len() == nwname {
            self.fids.insert(
                newfid,
                FileHandle {
                    file: if nwname != 0 { wfiles.last().unwrap().path } else { path },
                    open: None,
                },
            );
        }

        wfiles
    }

    pub fn open(&mut self, fid: u32, mode: OpenMode) -> Qid {
        let mut fid = self.fids.get_mut(&fid).expect("bad fid for openin");

        fid.open(&self.user, mode, &mut self.tree.all_files)
            .unwrap()
            .qid()
    }

    pub fn create(&mut self, fid: u32, name: &str, perm: FileMode, mode: OpenMode) -> Qid {
        let mut fid = self.fids.get_mut(&fid).expect("bad fid for creation");

        let parent_id = fid.file;
        let qid = {
            let file = fid
            .create(&self.user.user, name, perm, mode, &mut self.tree)
            .unwrap();

            file.qid()
        };
        self.tree.invalidate(parent_id);

        qid
    }

    // TODO: make a type for buf that will let the caller supply it if
    // they already have a buf, but if not, can let it be lazily allocated
    // in case of not having the right perms
    pub fn read(&mut self, fid: u32, offset: u64, buf: &mut [u8]) -> u32 {
        let fid = self.fids.get_mut(&fid).expect("unknown fid");
        let path = fid.file;

        if fid.open.is_none() {
            panic!("fid not open");
        }

        // if !fid.open.unwrap().is_readable() {
        //     panic!("fid not open for reading");
        // }

        let view = fid.open.as_mut().unwrap();

        if self
            .tree
            .all_files
            .get(&path)
            .expect("unknown path for known fid")
            .is_dir()
            && (offset != 0 && offset != view.last_offset)
        {
            panic!("bad dir read offset");
        }

        let amt = self.tree.read(path, offset, buf).expect("read err");

        view.last_offset += amt as u64;

        amt
    }

    pub fn write(&mut self, fid: u32, offset: u64, data: &[u8]) -> u32 {
        let fid = self.fids.get(&fid).expect("unknown fid");
        let path = fid.file;

        if fid.open.is_none() {
            panic!("fid not open");
        }

        // if !fid.open.unwrap().is_writable() {
        //     panic!("fid not open for writing");
        // }

        self.tree
            .write(path, &self.user.user, offset, data)
            .unwrap()
    }

    pub fn clunk(&mut self, fid: u32) {
        if let Some(fid) = self.fids.remove(&fid) {
            if let Some(view) = fid.open {
                // Whether or not other fids can still use the file is implementation defined
                if view.mode.contains(OpenMode::CLOSE) {
                    self.tree.remove(fid.file, &self.user);
                }
            }
        } else {
            panic!("Unknown fid for clunk");
        }
    }

    pub fn remove(&mut self, fid: u32) {
        if let Some(fid) = self.fids.remove(&fid) {
            // TODO: correctness with regards to others having it open
            self.tree.remove(fid.file, &self.user);
        } else {
            panic!("Unknown fid for clunk");
        }
    }

    pub fn stat(&self, fid: u32) -> Stat {
        let fid = self.fids.get(&fid).expect("unknown fid for stat");

        self.tree.stat(fid.file).expect("unknown path for stat")
    }

    pub fn wstat(&mut self, fid: u32, wstat: Stat) {
        let fid = self.fids.get(&fid).expect("unknown fid for wstat");

        self.tree
            .wstat(fid.file, &self.user.user, wstat)
            .expect("wstat didn't go so well I guess")
    }
}
