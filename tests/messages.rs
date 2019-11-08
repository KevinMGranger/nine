//! Tests relating to the message types.

extern crate byteorder;
extern crate nine;
use byteorder::{WriteBytesExt, LE};
use nine::de::*;
use nine::p2000::*;

use nine::ser::*;
use std::io::{Cursor, Read, Write};

fn des<R: Read>(r: R) -> ReadDeserializer<R> {
    ReadDeserializer(r)
}

fn ser() -> WriteSerializer<Cursor<Vec<u8>>> {
    WriteSerializer::new(Cursor::new(Vec::<u8>::new()))
}

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

#[test]
fn version() {
    let mut des_buf = Cursor::new(Vec::<u8>::new());
    des_buf.write_u16::<LE>(NOTAG).unwrap();
    des_buf.write_u32::<LE>(u16::max_value() as u32).unwrap();
    write_str(&mut des_buf, "9p2000");
    des_buf.set_position(0);

    let expected_ser_buf = des_buf.clone().into_inner();

    let mut des = des(des_buf);

    let actual_msg: Tversion = Deserialize::deserialize(&mut des).unwrap();

    let expected_msg = Tversion {
        tag: NOTAG,
        msize: u16::max_value() as u32,
        version: "9p2000".into(),
    };

    assert_eq!(actual_msg, expected_msg);

    let mut serializer = ser();

    expected_msg.serialize(&mut serializer).unwrap();

    let actual_ser_buf = serializer.writer.into_inner();

    assert_eq!(actual_ser_buf, expected_ser_buf);
}

#[test]
fn rauth() {
    let mut des_buf = Cursor::new(Vec::<u8>::new());
    let tag = 1;
    let file_type = FileType::AUTH;
    let version = 1;
    let path = 0;
    let qid = Qid {
        file_type,
        version,
        path,
    };
    des_buf.write_u16::<LE>(tag).unwrap();
    write_qid(&mut des_buf, &qid);
    let expected_msg = Rauth { tag, aqid: qid };

    des_buf.set_position(0);

    let expected_ser_buf = des_buf.clone().into_inner();

    let mut des = des(des_buf);

    let actual_msg: Rauth = Deserialize::deserialize(&mut des).unwrap();

    assert_eq!(actual_msg, expected_msg);

    let mut serializer = ser();

    expected_msg.serialize(&mut serializer).unwrap();

    let actual_ser_buf = serializer.writer.into_inner();

    assert_eq!(actual_ser_buf, expected_ser_buf);
}

#[test]
fn rstat() {
    let mut bytes = Cursor::new(Vec::<u8>::new());
    let tag = 1;
    let stat = Stat {
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
    };

    bytes.write_u16::<LE>(tag).unwrap();
    write_stat(&mut bytes, &stat);
    let expected_msg = Rstat { tag, stat };
    bytes.set_position(0);
    let expected_ser_buf = bytes.clone().into_inner();

    let mut des = des(bytes);

    let mut serializer = ser();

    let actual_msg: Rstat = Deserialize::deserialize(&mut des).unwrap();

    assert_eq!(actual_msg, expected_msg);

    expected_msg.serialize(&mut serializer).unwrap();

    let actual_ser_buf = serializer.writer.into_inner();

    assert_eq!(actual_ser_buf, expected_ser_buf);
}

#[test]
fn twalk() {
    let mut expected_des_buf = Cursor::new(Vec::<u8>::new());
    let expected_msg = Twalk {
        tag: 1,
        fid: 2,
        newfid: 3,
        wname: vec!["one".into(), "two".into()],
    };
    expected_des_buf.write_u16::<LE>(expected_msg.tag).unwrap();
    expected_des_buf.write_u32::<LE>(expected_msg.fid).unwrap();
    expected_des_buf
        .write_u32::<LE>(expected_msg.newfid)
        .unwrap();
    expected_des_buf.write_u16::<LE>(2).unwrap();
    write_str(&mut expected_des_buf, "one");
    write_str(&mut expected_des_buf, "two");

    expected_des_buf.set_position(0);
    let expected_ser_buf = expected_des_buf.clone().into_inner();
    let mut des = des(expected_des_buf);

    let actual_msg: Twalk = Deserialize::deserialize(&mut des).unwrap();

    assert_eq!(actual_msg, expected_msg);

    let mut serializer = ser();

    expected_msg.serialize(&mut serializer).unwrap();

    let actual_ser_buf = serializer.writer.into_inner();

    assert_eq!(actual_ser_buf, expected_ser_buf);
}

#[test]
fn rread() {
    let mut expected_des_buf = Cursor::new(Vec::<u8>::new());
    let expected_msg = Rread {
        tag: 1,
        data: "hello".to_string().into_bytes()
    };
    expected_des_buf.write_u16::<LE>(expected_msg.tag).unwrap();
    expected_des_buf.write_u32::<LE>(5).unwrap();
    expected_des_buf.write(&expected_msg.data).unwrap();
    expected_des_buf.set_position(0);

    let expected_ser_buf = expected_des_buf.clone().into_inner();
    let mut des = des(expected_des_buf);

    let actual_msg = Rread::deserialize(&mut des).unwrap();

    assert_eq!(actual_msg, expected_msg);

    let mut serializer = ser();

    expected_msg.serialize(&mut serializer).unwrap();

    assert_eq!(expected_ser_buf, serializer.writer.into_inner());
}
