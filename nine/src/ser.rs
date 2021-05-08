//! Serializers and serializer convenience functions.

use std::io::{Cursor, Seek, SeekFrom};

mod common;
pub use common::*;

mod write_seek;
pub use write_seek::*;

mod count;
pub use count::*;

//region Functions
// TODO: this is mainly used for converting stat calls.
// It should take a ref to the writer, which would be the existing buffer on the dir.
//
/// Serialize the given object into a new vec buffer.
/// ```
/// # use nine::ser::*;
/// use nine::p2000::Rerror;
/// let version = Rerror { tag: 0, ename: "foo".into() };
/// let bytes: Vec<u8> = into_bytes(&version).unwrap();
/// ```
pub fn into_bytes<T: Serialize>(t: &T) -> Result<Vec<u8>, SerError> {
    let mut ser = WriteSerializer {
        writer: Cursor::new(Vec::new()),
        in_stat: false,
    };

    let res = t.serialize(&mut ser);

    res.map(|_| ser.into_writer().into_inner())
        .map_err(|e| e.unwrap_ser_error())
}

// TODO: document the error conditions better.
/// Serializes the given item into the given buffer.
///
/// Returns the number of bytes written.
///
/// Can fail if the buffer isn't big enough.
/// ```
/// # use nine::ser::*;
/// use nine::p2000::Rerror;
/// let mut buf = [0u8; 31];
/// let err = Rerror { tag: 0, ename: "foo".into() };
/// let amount = into_buf(&err, &mut buf).unwrap();
/// let serialized_message = &buf[0..amount as usize];
/// ```
pub fn into_buf<T: Serialize, B: AsMut<[u8]>>(t: &T, mut buf: B) -> Result<u32, SerError> {
    let writer = Cursor::new(buf.as_mut());
    into_write_seeker(t, writer).map_err(|e| e.unwrap_ser_error())
}

/// Serializes the given item into the given Vec.
///
/// Returns the number of bytes written.
///
/// Will typically not fail because of space issues.
/// ```
/// # use nine::ser::*;
/// use nine::p2000::Rerror;
/// let mut buf = Vec::new();
/// let err = Rerror { tag: 0, ename: "foo".into() };
/// let amount = into_vec(&err, &mut buf).unwrap();
/// ```
pub fn into_vec<T: Serialize, V: AsMut<Vec<u8>>>(t: &T, mut vec: V) -> Result<u32, SerError> {
    let writer = Cursor::new(vec.as_mut());
    into_write_seeker(t, writer).map_err(|e| e.unwrap_ser_error())
}

/// Serializes the given item at the _end_ of the given Vec.
///
/// Returns the number of bytes written.
///
/// Will typically not fail because of space issues.
/// ```
/// # use nine::ser::*;
/// use nine::p2000::Rerror;
/// let mut buf = Vec::new();
/// let err = Rerror { tag: 0, ename: "foo".into() };
/// let amount = append_vec(&err, &mut buf).unwrap();
/// ```
pub fn append_vec<T: Serialize, V: AsMut<Vec<u8>>>(t: &T, mut vec: V) -> Result<u32, SerError> {
    let vec = vec.as_mut();
    let pos = vec.len();
    let mut writer = Cursor::new(vec);
    writer.seek(SeekFrom::Start(pos as u64)).unwrap();
    into_write_seeker(t, writer).map_err(|e| e.unwrap_ser_error())
}
//endregion
