pub use super::{
    Qid, Rattach, Rauth, Rclunk, Rcreate, Rerror, Rflush, Ropen, Rread, Rremove, Rstat, Rversion,
    Rwalk, Rwrite, Rwstat, Tclunk, Tcreate, Tflush, Topen, Tread, Tremove, Tstat, Tversion, Twalk,
    Twrite, Twstat,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tauth {
    pub tag: u16,
    pub afid: u32,
    pub uname: String,
    pub aname: String,
    pub n_uname: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tattach {
    pub tag: u16,
    pub fid: u32,
    pub afid: u32,
    pub uname: String,
    pub aname: String,
    pub n_uname: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rlerror {
    pub tag: u16,
    pub ecode: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tstatfs {
    pub tag: u16,
    pub fid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rstatfs {
    pub tag: u16,
    pub r#type: u32,
    pub bsize: u32,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub fsid: u64,
    pub namelen: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tlopen {
    pub tag: u16,
    pub fid: u32,
    pub flags: u32, // TODO
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rlopen {
    pub tag: u16,
    pub fid: u32,
    pub iounit: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tlcreate {
    pub tag: u16,
    pub fid: u32,
    pub name: String,
    pub flags: u32, // TODO
    pub mode: u32,  // todo
    pub gid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rlcreate {
    pub tag: u16,
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tsymlink {
    pub tag: u16,
    pub fid: u32,
    pub name: String,
    pub symtgt: String,
    pub gid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rsymlink {
    pub tag: u16,
    pub qid: Qid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tmknod {
    pub tag: u16,
    pub dfid: u32,
    pub name: String,
    pub mode: u32, // TODO
    pub major: u32,
    pub minor: u32,
    pub gid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rmknod {
    pub tag: u16,
    pub qid: Qid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Trename {
    pub tag: u16,
    pub fid: u32,
    pub dfid: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rrename {
    pub tag: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Treadlink {
    pub tag: u16,
    pub fid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rreadlink {
    pub tag: u16,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tgetattr {
    pub tag: u16,
    pub fid: u32,
    pub request_mask: u64, // TODO
}

// TODO: flags for all of this, perhaps temporal types?
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rgetattr {
    pub tag: u16,
    pub valid: u64,
    pub qid: Qid,
    pub mode: u32, // TODO
    pub uid: u32,
    pub gid: u32,
    pub nlink: u64,

    pub rdev: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,

    pub atime_sec: u64,
    pub atime_nsec: u64,
    pub mtime_sec: u64,
    pub mtime_nsec: u64,

    pub ctime_sec: u64,
    pub ctime_nsec: u64,
    pub btime_sec: u64,
    pub btime_nsec: u64,

    pub gen: u64,
    pub data_version: u64,
}

// TODO validity flags, chrono
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Tsetattr {
    pub tag: u16,
    pub valid: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,

    pub atime_sec: u64,
    pub atime_nsec: u64,
    pub mtime_sec: u64,
    pub mtime_nsec: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rsetattr {
    pub tag: u16,
}

// TODO: the rest
