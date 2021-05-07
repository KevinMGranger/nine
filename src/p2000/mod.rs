//! Message types and other constructs for 9p protocols based on 9p2000.

pub mod l;
pub mod u;

pub use crate::common::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// The tag number used to represent that tags are irrelevant for this message.
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
    /// Whether or not the OpenMode means a file is
    /// (or is requested to be) readable.
    pub fn is_readable(&self) -> bool {
        let lower = self.bits() & 3;
        lower == 0 || lower == 2
    }

    /// Whether or not the OpenMode means a file is
    /// (or is requested to be) writable.
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
    pub name: String,
    pub uid: String,
    pub gid: String,
    pub muid: String,
}

impl Stat {
    /// Whether or not the stat allows the given other permission, or has the
    /// given owner permission for the given owner.
    fn perm_for(&self, other: FileMode, owner: FileMode, user: &str) -> bool {
        self.mode.contains(other) || (self.mode.contains(owner) && self.uid == user)
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

messages! {
    #[derive(Debug, PartialEq, Eq)]
     Tversion {
        msize: u32,
        version: String,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rversion {
        msize: u32,
        version: String,
    }
    #[derive(Debug, PartialEq, Eq)]
     Tauth {
        afid: u32,
        uname: String,
        aname: String,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rauth {
        aqid: Qid,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rerror {
        ename: String,
    }
    #[derive(Debug, PartialEq, Eq)]
     Tflush {
    }
    #[derive(Debug, PartialEq, Eq)]
     Rflush {
    }
    #[derive(Debug, PartialEq, Eq)]
     Tattach {
        fid: u32,
        afid: u32,
        uname: String,
        aname: String,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rattach {
        qid: Qid,
    }
    #[derive(Debug, PartialEq, Eq)]
     Twalk {
        fid: u32,
        newfid: u32,
        wname: Vec<String>,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rwalk {
        wqid: Vec<Qid>,
    }
    #[derive(Debug, PartialEq, Eq)]
     Topen {
        fid: u32,
        mode: OpenMode,
    }
    #[derive(Debug, PartialEq, Eq)]
     Ropen {
        qid: Qid,
        iounit: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Tcreate {
        fid: u32,
        name: String,
        perm: FileMode,
        mode: OpenMode,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rcreate {
        qid: Qid,
        iounit: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Tread {
        fid: u32,
        offset: u64,
        count: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rread {
        #[serde(
            serialize_with = "serialize_bytes",
            deserialize_with = "deserialize_owned_bytes"
        )]
        data: Vec<u8>,
    }
    #[derive(Debug, PartialEq, Eq)]
     Twrite {
        fid: u32,
        offset: u64,
        #[serde(
            serialize_with = "serialize_bytes",
            deserialize_with = "deserialize_owned_bytes"
        )]
        data: Vec<u8>,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rwrite {
        count: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Tclunk {
        fid: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rclunk {
    }
    #[derive(Debug, PartialEq, Eq)]
     Tremove {
        fid: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rremove {
    }
    #[derive(Debug, PartialEq, Eq)]
     Tstat {
        fid: u32,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rstat {
        stat: Stat,
    }
    #[derive(Debug, PartialEq, Eq)]
     Twstat {
        fid: u32,
        stat: Stat,
    }
    #[derive(Debug, PartialEq, Eq)]
     Rwstat {
    }

}

pub use tagged::*;

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
