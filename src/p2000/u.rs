pub use super::{
    OpenMode, Qid, Rattach, Rauth, Rclunk, Rcreate, Rerror, Rflush, Ropen, Rread, Rremove, Rstat,
    Rversion, Rwalk, Rwrite, Rwstat, Tattach, Tauth, Tclunk, Tcreate, Tflush, Topen, Tread,
    Tremove, Tstat, Tversion, Twalk, Twrite, Twstat,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Topenfd {
    pub tag: u16,
    pub fid: u32,
    pub mode: OpenMode,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Ropenfd {
    pub tag: u16,
    pub qid: Qid,
    pub iounit: u32,
    pub unixfd: u32,
}

message_type_ids! {
    Topenfd = 98,
    Ropenfd = 99
}
