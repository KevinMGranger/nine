extern crate nine;

mod util;

use nine::ser::*;
use std::iter::repeat;
use std::{u16, u32};
use util::BlackHoleWriter;

fn ser() -> WriteSerializer<BlackHoleWriter> {
    WriteSerializer::new(BlackHoleWriter)
}

fn expect_err<T: Serialize>(t: &T) -> SerFail {
    let mut serializer = ser();
    t.serialize(&mut serializer).unwrap_err().0.into_inner()
}

#[test]
fn overlong_seq_known() {
    let long_seq = repeat(false)
        .take(u16::MAX as usize + 1)
        .collect::<Vec<bool>>();

    assert!(if let SerFail::SeqTooLong = expect_err(&long_seq) {
        true
    } else {
        false
    });
}

#[test]
fn max_seq() {
    let long_seq = repeat(false).take(u16::MAX as usize).collect::<Vec<bool>>();
    let mut serializer = ser();

    long_seq.serialize(&mut serializer).unwrap();
}

#[test]
fn overlong_str() {
    let long_string = "x".repeat(u16::MAX as usize + 1);

    assert!(if let SerFail::StringTooLong = expect_err(&long_string) {
        true
    } else {
        false
    })
}

#[test]
fn max_str() {
    let length = u16::MAX as u32;
    let string = "x".repeat(length as usize);
    let mut serializer = ser();

    assert_eq!(length + 2, string.serialize(&mut serializer).unwrap())
}

#[test]
#[ignore]
fn overlong_bytes() {
    let bytes = vec![1u8; u32::MAX as usize];
    let mut serializer = ser();

    assert!(
        if let SerFail::BytesTooLong = nine::common::serialize_bytes(&bytes, &mut serializer)
            .unwrap_err()
            .0
            .into_inner()
        {
            true
        } else {
            false
        }
    )
}

#[test]
#[ignore]
fn max_bytes() {
    let bytes = vec![1u8; u32::MAX as usize - 8];
    let mut serializer = ser();

    nine::common::serialize_bytes(&bytes, &mut serializer).unwrap();
}