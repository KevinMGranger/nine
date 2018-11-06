//! Tests relating to the serde data model directly.
extern crate nine;

use nine::{de::*, ser};
use ser::*;
use std::io::{Read, Result, Seek, SeekFrom, Write};

pub struct BlackHoleWriter;

impl Write for BlackHoleWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for BlackHoleWriter {
    fn seek(&mut self, _from: SeekFrom) -> Result<u64> {
        Ok(0)
    }
}

fn ser() -> WriteSerializer<BlackHoleWriter> {
    WriteSerializer::new(BlackHoleWriter)
}

fn de(r: impl Read) -> ReadDeserializer<impl Read> {
    ReadDeserializer(r)
}

#[test]
fn overlong_str() {
    let long_string = "x".repeat(u16::max_value() as usize + 1);
    let mut serializer = ser();

    assert!(long_string.serialize(&mut serializer).is_err())
}

#[test]
fn max_str() {
    let length = u16::max_value() as usize;
    let string = "x".repeat(length);
    let mut serializer = ser();

    assert_eq!(length + 2, string.serialize(&mut serializer).unwrap())
}

#[test]
#[ignore]
fn overlong_bytes() {
    let bytes = Data(vec![1u8; u32::max_value() as usize + 1]);
    let mut serializer = ser();

    assert!(bytes.serialize(&mut serializer).is_err());
}

#[test]
#[ignore]
fn max_bytes() {
    let length = u32::max_value() as usize;
    let bytes = Data(vec![1u8; length]);
    let mut serializer = ser();

    assert_eq!(length + 4, bytes.serialize(&mut serializer).unwrap());
}

#[test]
fn overlong_seq() {
    let seq = [0u8; u16::max_value() as usize + 1];
    let mut serializer = ser();

    assert!(seq.serialize(&mut serializer).is_err())
}

#[test]
fn max_seq() {
    let length = u16::max_value() as usize;
    let seq = vec![0u8; length];
    let mut serializer = ser();

    assert_eq!(length + 2, seq.serialize(&mut serializer).unwrap());
}

#[test]
#[ignore]
fn invalid_utf() {
    let bytes: &[u8] = &[0, 0]; // TODO: an invalid utf example
    let mut de = de(bytes);

    assert!(String::deserialize(&mut de).is_err())
}

#[test]
fn io_error() {
    // unexpected EOF
    let bytes: &[u8] = &[1, 0];
    let mut de = de(bytes);

    assert!(String::deserialize(&mut de).is_err())
}
