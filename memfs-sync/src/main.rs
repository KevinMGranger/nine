mod server;
mod utils;

use nine::de::*;
use nine::p2000::*;
use nine::ser::*;
use crate::server::*;
use std::collections::{HashMap, HashSet};
use std::env::{args, var};
use std::io::prelude::*;
use std::io::{self, Cursor};
use std::os::unix::net::UnixListener;

macro_rules! match_message {
    {
        ($self:ident, $serializer:ident, $received_discriminant:ident)
        $($message_type:ty => $func:ident),*
    } => {
        match $received_discriminant {
            $(
            <$message_type as ConstMessageTypeId>::MSG_TYPE_ID => {
                let msg: $message_type = $self.read_a().unwrap();
                let tag = msg.tag;
                println!("<{:?}", &msg);
                let type_id = match $self.$func(msg) {
                    Ok(response) => {
                        println!(">{:?}", &response);
                        response.serialize(&mut $serializer).unwrap();
                        response.msg_type_id()
                    },
                    Err(ServerError::NonFatal { msg: response, .. }) => {
                        let response = Rerror { tag: tag, ename: format!("{}", response).into() };
                        println!(">{:?}", &response);
                        response.serialize(&mut $serializer).unwrap();
                        response.msg_type_id()
                    },
                    e => { e.unwrap(); 0}
                };

                let buf = $serializer.into_writer().into_inner();

                $self.write_msg(type_id, buf).unwrap();
            },
            )*
            _ => {
                eprintln!("unknown msg type {}", $received_discriminant);
            }
        }
    }
}

struct Server<Stream>
where
    for<'a> &'a Stream: Read + Write,
{
    stream: Stream,
    session: Option<Session>,
    file_tree: Option<FileTree>,
}

impl<Stream> Server<Stream>
where
    for<'a> &'a Stream: Read + Write,
{
    fn read_a<'de, T: Deserialize<'de>>(&self) -> std::result::Result<T, DeError> {
        let mut deserializer = ReadDeserializer(&self.stream);
        Deserialize::deserialize(&mut deserializer)
    }

    fn write_msg(&self, mtype: u8, buf: &[u8]) -> io::Result<()> {
        let writer = &mut &self.stream;
        let msize = buf.len() as u32 + 5;
        writer.write_all(&msize.to_le_bytes())?;

        // println!(
        //     "writing msg of length {} and type {}: {:?}",
        //     msize, mtype, buf
        // );
        Write::write_all(writer, &[mtype])?;
        Ok(Write::write_all(writer, buf)?)
    }

    fn sess(&mut self) -> &mut Session {
        self.session.as_mut().unwrap()
    }

    fn handle_client(&mut self) {
        let mut write_buffer = Vec::new();

        let quit_msg = loop {
            write_buffer.clear();
            let mut serializer = WriteSerializer::new(Cursor::new(&mut write_buffer));
            let _mlen: u32 = match self.read_a::<u32>() {
                Ok(x) => x,
                Err(derror) => if derror.is_eof() {
                    break "Client disconnected";
                } else {
                    panic!("{}", derror)
                },
            };

            let mtype: u8 = self.read_a().unwrap();
            match_message! {
                (self, serializer, mtype)
                Tversion => version,
                Tauth => auth,
                Tattach => attach,
                Tflush => flush,
                Twalk => walk,
                Topen => open,
                Tcreate => create,
                Tread => read,
                Twrite => write,
                Tclunk => clunk,
                Tremove => remove,
                Tstat => stat,
                Twstat => wstat
            }
        };

        println!("{}", quit_msg);
    }

    fn end_session(&mut self) -> FileTree {
        if let Some(sess) = self.session.take() {
            println!("tree from session");
            sess.tree
        } else if let Some(tree) = self.file_tree.take() {
            println!("tree from tree");
            tree
        } else {
            panic!()
        }
    }

    fn version(&mut self, msg: Tversion) -> NineResult<Rversion> {
        Ok(Rversion {
            tag: msg.tag,
            msize: msg.msize,
            version: "9P2000".into(),
        })
    }

    fn auth(&mut self, _msg: Tauth) -> NineResult<Rauth> {
        rerr("no auth needed")
    }

    fn attach(&mut self, msg: Tattach) -> NineResult<Rattach> {
        // TODO: what if session is already present?
        // the take().unwrap() will fail but figure out what's
        // correct here protocol wise
        let mut fids = HashMap::new();
        fids.insert(
            msg.fid,
            FileHandle {
                file: 0,
                open: None,
            },
        );
        let tree = self.file_tree.take().unwrap();
        let qid = tree.qid(0).unwrap();
        self.session = Some(Session {
            fids,
            tree,
            user: User {
                user: msg.uname,
                group: "TODO_GROUP".into(),
            },
        });

        Ok(Rattach { tag: msg.tag, qid })
    }

    fn flush(&mut self, msg: Tflush) -> NineResult<Rflush> {
        // all calls are blocking so this is meaningless to us
        Ok(Rflush { tag: msg.tag })
    }

    fn walk(&mut self, msg: Twalk) -> NineResult<Rwalk> {
        Ok(Rwalk {
            tag: msg.tag,
            wqid: self
                .session
                .as_mut()
                .unwrap()
                .walk(msg.fid, msg.newfid, msg.wname.into_iter()),
        })
    }

    fn open(&mut self, msg: Topen) -> NineResult<Ropen> {
        Ok(Ropen {
            tag: msg.tag,
            qid: self.session.as_mut().unwrap().open(msg.fid, msg.mode),
            iounit: u32::max_value(),
        })
    }

    fn create(&mut self, msg: Tcreate) -> NineResult<Rcreate> {
        Ok(Rcreate {
            tag: msg.tag,
            qid: self
                .session
                .as_mut()
                .unwrap()
                .create(msg.fid, &msg.name, msg.perm, msg.mode),
            iounit: u32::max_value(),
        })
    }

    fn read(&mut self, msg: Tread) -> NineResult<Rread> {
        let mut vec = Vec::with_capacity(msg.count as usize);
        unsafe {
            vec.set_len(msg.count as usize);
        }
        let length = self.sess().read(msg.fid, msg.offset, vec.as_mut_slice());
        vec.truncate(length as usize);

        Ok(Rread {
            tag: msg.tag,
            data: vec,
        })
    }

    fn write(&mut self, msg: Twrite) -> NineResult<Rwrite> {
        Ok(Rwrite {
            tag: msg.tag,
            count: self.sess().write(msg.fid, msg.offset, msg.data.as_ref()),
        })
    }

    fn clunk(&mut self, msg: Tclunk) -> NineResult<Rclunk> {
        self.sess().clunk(msg.fid);
        Ok(Rclunk { tag: msg.tag })
    }

    fn remove(&mut self, msg: Tremove) -> NineResult<Rremove> {
        self.sess().remove(msg.fid);
        Ok(Rremove { tag: msg.tag })
    }

    fn stat(&mut self, msg: Tstat) -> NineResult<Rstat> {
        Ok(Rstat {
            tag: msg.tag,
            stat: self.sess().stat(msg.fid),
        })
    }

    fn wstat(&mut self, msg: Twstat) -> NineResult<Rwstat> {
        self.sess().wstat(msg.fid, msg.stat);
        Ok(Rwstat { tag: msg.tag })
    }
}

fn mktree(user: String) -> FileTree {
    let root = File {
        parent: 0,
        content: Vec::new(),
        meta: CommonFileMetaData {
            version: 0,
            path: 0,
            mode: FileMode::DIR
                | FileMode::OWNER_READ
                | FileMode::GROUP_READ
                | FileMode::OTHER_READ

                | FileMode::OWNER_WRITE
                | FileMode::OTHER_WRITE

                | FileMode::OWNER_EXEC
                | FileMode::OTHER_EXEC
                
                ,
            atime: 3,
            mtime: 4,
            name: "/".into(),
            uid: user.into(),
            gid: "gid".into(),
            muid: "muid".into(),
        },
        children: HashSet::new(),
    };

    let mut all_files = HashMap::new();
    all_files.insert(root.meta.path, root);

    FileTree {
        last_path: 0,
        all_files,
    }
}

fn main() {
    let bind_path = args().nth(1).unwrap();

    let listener = {
        let x = UnixListener::bind(&bind_path);
        if x.is_err() {
            std::fs::remove_file(&bind_path).unwrap();
            UnixListener::bind(&bind_path).unwrap()
        } else {
            x.unwrap()
        }
    };

    let whoami = var("USER").unwrap();

    let mut tree = Some(mktree(whoami));

    println!("Serving at {}", &bind_path);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut server = Server {
                    stream,
                    session: None,
                    file_tree: tree.take(),
                };
                server.handle_client();

                tree = Some(server.end_session());
            }
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }
}
