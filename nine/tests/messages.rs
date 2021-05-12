///! Tests relating to message type serialization.
extern crate byteorder;
extern crate nine;
use byteorder::{WriteBytesExt, LE};
use nine::de::*;
use nine::p2000::*;

use nine::ser::*;
use std::io::{Cursor, Write};

//region test helpers
fn write_qid<W: WriteBytesExt>(bytes: &mut W, qid: &Qid) {
    bytes.write_u8(qid.file_type.bits()).unwrap();
    bytes.write_u32::<LE>(qid.version).unwrap();
    bytes.write_u64::<LE>(qid.path).unwrap();
}

/// NOT including size field. Add another 2 if that's wrong
fn stat_len(s: &Stat) -> u16 {
    2 // type: kernel use
        + 4 // dev
        + 13 // qid
        + 4 // mode
        + 4 // atime
        + 4 //mtime
        + 8 //length
        + s.name.len() as u16
        + s.uid.len() as u16
        + s.gid.len() as u16
        + s.muid.len() as u16
        + 8 // sizes for above strings
}

fn write_str<W: Write + WriteBytesExt>(w: &mut W, s: &str) {
    w.write_u16::<LE>(s.len() as u16).unwrap();
    w.write(s.as_bytes()).unwrap();
}

fn write_stat<W: Write + WriteBytesExt>(bytes: &mut W, s: &Stat) {
    bytes.write_u16::<LE>(stat_len(s) + 2).unwrap();
    bytes.write_u16::<LE>(stat_len(s)).unwrap();
    bytes.write_u16::<LE>(s.type_).unwrap();
    bytes.write_u32::<LE>(s.dev).unwrap();
    write_qid(bytes, &s.qid);
    bytes.write_u32::<LE>(s.mode.bits()).unwrap();
    bytes.write_u32::<LE>(s.atime).unwrap();
    bytes.write_u32::<LE>(s.mtime).unwrap();
    bytes.write_u64::<LE>(s.length).unwrap();
    write_str(bytes, &s.name);
    write_str(bytes, &s.uid);
    write_str(bytes, &s.gid);
    write_str(bytes, &s.muid);
}
//endregion

macro_rules! message_test {
    ($name:ident, $size:expr) => {
        ::paste::paste! {
            #[test]
            fn [<$name _size>]() {
                assert_eq!(size_for(&[<$name _msg>]()).unwrap(), $size);
            }

            #[test]
            fn [<$name _ser_vec>]() {
                assert_eq!(
                    [<$name _bytes>]().into_inner(),
                    into_bytes(&[<$name _msg>]()).unwrap()
                );
            }

            #[test]
            fn [<$name _de_read>]() {
                assert_eq!([<$name _msg>](), from_reader([<$name _bytes>]()).unwrap());
            }
        }
    };
}
// region version
fn version_bytes() -> Cursor<Vec<u8>> {
    let mut des_buf = Cursor::new(Vec::<u8>::new());
    des_buf.write_u16::<LE>(NOTAG).unwrap();
    des_buf.write_u32::<LE>(u16::max_value() as u32).unwrap();
    write_str(&mut des_buf, "9p2000");
    des_buf.set_position(0);
    des_buf
}

fn version_msg() -> Tversion {
    return Tversion {
        tag: NOTAG,
        msize: u16::max_value() as u32,
        version: "9p2000".into(),
    };
}

message_test!(version, 2 + 4 + 8);
//endregion

// region rauth
fn rauth_msg() -> Rauth {
    return Rauth {
        tag: 1,
        aqid: Qid {
            file_type: FileType::AUTH,
            version: 1,
            path: 0,
        },
    };
}

fn rauth_bytes() -> Cursor<Vec<u8>> {
    let mut des_buf = Cursor::new(Vec::<u8>::new());
    des_buf.write_u16::<LE>(rauth_msg().tag).unwrap();
    write_qid(&mut des_buf, &rauth_msg().aqid);
    des_buf.set_position(0);

    des_buf
}

message_test!(rauth, 2 + 13);
//endregion

// region rstat
fn rstat_msg() -> Rstat {
    return Rstat {
        tag: 1,
        stat: Stat {
            type_: 1,
            dev: 2,
            qid: Qid {
                file_type: FileType::FILE,
                version: 3,
                path: 4,
            },
            mode: FileMode::OWNER_READ | FileMode::OWNER_WRITE,
            atime: 5,
            mtime: 6,
            length: 512,
            name: "hello".into(),
            uid: "glenda".into(),
            gid: "glenda".into(),
            muid: "glenda".into(),
        },
    };
}

fn rstat_bytes() -> Cursor<Vec<u8>> {
    let mut bytes = Cursor::new(Vec::<u8>::new());
    bytes.write_u16::<LE>(rstat_msg().tag).unwrap();
    write_stat(&mut bytes, &rstat_msg().stat);
    bytes.set_position(0);
    bytes
}

message_test!(rstat, 2 + 4 + stat_len(&rstat_msg().stat) as u32);
//endregion
fn twalk_msg() -> Twalk {
    return Twalk {
        tag: 1,
        fid: 2,
        newfid: 3,
        wname: vec!["one".into(), "two".into()],
    };
}
fn twalk_bytes() -> Cursor<Vec<u8>> {
    let expected_msg = twalk_msg();

    let mut expected_des_buf = Cursor::new(Vec::<u8>::new());
    expected_des_buf.write_u16::<LE>(expected_msg.tag).unwrap();
    expected_des_buf.write_u32::<LE>(expected_msg.fid).unwrap();
    expected_des_buf
        .write_u32::<LE>(expected_msg.newfid)
        .unwrap();
    expected_des_buf.write_u16::<LE>(2).unwrap();
    write_str(&mut expected_des_buf, "one");
    write_str(&mut expected_des_buf, "two");

    expected_des_buf.set_position(0);

    expected_des_buf
}

message_test!(twalk, 2 + 4 + 4 + 2 + 4 + 6);

//region rread
fn rread_msg() -> Rread {
    return Rread {
        tag: 1,
        data: "hello".to_string().into_bytes(),
    };
}

fn rread_bytes() -> Cursor<Vec<u8>> {
    let mut expected_des_buf = Cursor::new(Vec::<u8>::new());
    let expected_msg = rread_msg();
    expected_des_buf.write_u16::<LE>(expected_msg.tag).unwrap();
    expected_des_buf.write_u32::<LE>(5).unwrap();
    expected_des_buf.write(&expected_msg.data).unwrap();
    expected_des_buf.set_position(0);

    expected_des_buf
}

message_test!(rread, 2 + 4 + 5);
//endregion