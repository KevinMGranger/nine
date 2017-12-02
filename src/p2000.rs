pub use common::*;

pub const NOTAG: u16 = !0u16;

bitflags! {
    /// The type of a file. Used within Qids.
    #[derive(Serialize, Deserialize)]
    pub struct FileType: u8 {
        const FILE      = 0b00000000;
        const DIR       = 0b10000000;
        const APPEND    = 0b01000000;
        const EXCL      = 0b00100000;
      //const SKIPPED   = 0b00010000;
        const AUTH      = 0b00001000;
        const TEMPORARY = 0b00000100;
    }
}

bitflags! {
    /// The mode in which a file is to be opened.
    #[derive(Serialize, Deserialize)]
    pub struct OpenMode: u8 {
        const READ = 0;
        const WRITE = 1;
        const RDWR = 2;
        const EXEC = 3;
        const TRUNC = 0x10;
        const CLOSE = 0x40;
    }
}

impl OpenMode {
    /// Whether or not the OpenMode means a file is (or is requested to be)
    /// readable.
    pub fn is_readable(&self) -> bool {
        let lower = self.bits() & 3;
        lower == 0 || lower == 2
    }

    /// Whether or not the OpenMode means a file is (or is requested to be)
    /// writable.
    pub fn is_writable(&self) -> bool {
        let lower = self.bits() & 3;
        lower == 1 || lower == 2
    }
}

bitflags! {
    /// The mode of a file, representing its type as well as permissions.
    #[derive(Serialize, Deserialize)]
    pub struct FileMode: u32 {
        const DIR = 1 << 31;
        const APPEND = 1 << 30;
        const EXCL = 1 << 29;
        const AUTH = 1 << 27;
        const TMP = 1 << 26;

        const OWNER_READ  = 1 << 8;
        const OWNER_WRITE = 1 << 7;
        const OWNER_EXEC  = 1 << 6;

        const GROUP_READ = 1 << 5;
        const GROUP_WRITE = 1 << 4;
        const GROUP_EXEC = 1 << 3;

        const OTHER_READ = 1 << 2;
        const OTHER_WRITE = 1 << 1;
        const OTHER_EXEC = 1;

        const MODE_MASK = 0b11111111 << 24;
        const PERM_MASK = 0b111111111;
    }
}

/// Extracts the FileType information from a FileMode.
impl From<FileMode> for FileType {
    fn from(mode: FileMode) -> Self {
        FileType::from_bits((mode.bits() >> 24) as u8).unwrap()
    }
}

/// A Qid contains information about the type of a file, its
/// edit version, and its uniquely identified "path".
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Qid {
    pub file_type: FileType,
    pub version: u32,
    pub path: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Stat {
    pub type_: u16,
    pub dev: u32,
    pub qid: Qid,
    pub mode: FileMode,
    pub atime: u32,
    pub mtime: u32,
    pub length: u64,
    pub name: CowStr,
    pub uid: CowStr,
    pub gid: CowStr,
    pub muid: CowStr,
}

impl Stat {
    /// Whether or not the stat allows the given other permission, or has the
    /// given owner permission for the given owner.
    fn perm_for(&self, other: FileMode, owner: FileMode, user: &str) -> bool {
        self.mode.contains(other)
            || (self.mode.contains(owner) && self.uid == user)
    }
    /// Whether or not the stat says it is readable for the given user,
    /// explicitly or by extension (groups, others).
    pub fn readable_for(&self, user: &str) -> bool {
        self.perm_for(FileMode::OTHER_READ, FileMode::OWNER_READ, user)
    }
    /// Whether or not the stat says it is writable for the given user,
    /// explicitly or by extension (groups, others).
    pub fn writable_for(&self, user: &str) -> bool {
        self.perm_for(FileMode::OTHER_WRITE, FileMode::OWNER_WRITE, user)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tversion {
    pub tag: u16,
    pub msize: u32,
    pub version: CowStr,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rversion {
    pub tag: u16,
    pub msize: u32,
    pub version: CowStr,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tauth {
    pub tag: u16,
    pub afid: u32,
    pub uname: CowStr,
    pub aname: CowStr,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rauth {
    pub tag: u16,
    pub aqid: Qid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rerror {
    pub tag: u16,
    pub ename: CowStr,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tflush {
    pub tag: u16,
    pub oldtag: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rflush {
    pub tag: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tattach {
    pub tag: u16,
    pub fid: u32,
    pub afid: u32,
    pub uname: CowStr,
    pub aname: CowStr,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rattach {
    pub tag: u16,
    pub qid: Qid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Twalk {
    pub tag: u16,
    pub fid: u32,
    pub newfid: u32,
    pub wname: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rwalk {
    pub tag: u16,
    pub wqid: Vec<Qid>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Topen {
    pub tag: u16,
    pub fid: u32,
    pub mode: OpenMode,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Ropen {
    pub tag: u16,
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tcreate {
    pub tag: u16,
    pub fid: u32,
    pub name: CowStr,
    pub perm: FileMode,
    pub mode: OpenMode,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rcreate {
    pub tag: u16,
    pub qid: Qid,
    pub iounit: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tread {
    pub tag: u16,
    pub fid: u32,
    pub offset: u64,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rread {
    pub tag: u16,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Twrite {
    pub tag: u16,
    pub fid: u32,
    pub offset: u64,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rwrite {
    pub tag: u16,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tclunk {
    pub tag: u16,
    pub fid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rclunk {
    pub tag: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tremove {
    pub tag: u16,
    pub fid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rremove {
    pub tag: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tstat {
    pub tag: u16,
    pub fid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rstat {
    pub tag: u16,
    pub stat: Stat,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Twstat {
    pub tag: u16,
    pub fid: u32,
    pub stat: Stat,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rwstat {
    pub tag: u16,
}

message_type_ids! {
    Tversion = 100,
    Rversion = 101,

    Tauth = 102,
    Rauth = 103,

    Tattach = 104,
    Rattach = 105,

    Rerror = 107,

    Tflush = 108,
    Rflush = 109,

    Twalk = 110,
    Rwalk = 111,

    Topen = 112,
    Ropen = 113,

    Tcreate = 114,
    Rcreate = 115,

    Tread = 116,
    Rread = 117,

    Twrite = 118,
    Rwrite = 119,

    Tclunk = 120,
    Rclunk = 121,

    Tremove = 122,
    Rremove = 123,

    Tstat = 124,
    Rstat = 125,

    Twstat = 126,
    Rwstat = 127
}
