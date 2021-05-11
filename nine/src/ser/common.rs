use serde::ser::{self, *};
pub use serde::ser::{Serialize, Serializer};
use std::fmt::Display;
use std::io;
use thiserror::Error;

/// A serializer function that serializes any byte slice like object.
pub fn serialize_bytes<T, S>(t: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_bytes(t.as_ref())
}

//region Error Handling

/// A failure at the serialization lagyer.
#[derive(Error, Debug)]
pub enum SerError {
    #[error("Serialize Error: {0}")]
    SerializeError(String),
    #[error("String was too long")]
    StringTooLong,
    #[error("Byte buffer was too long")]
    BytesTooLong,
    #[error("Sequence too long")]
    SeqTooLong,
    #[error("Total size was bigger than u32")]
    TooBig,
    #[error("Type {0} is unspecified in 9p")]
    UnspecifiedType(&'static str),
    #[error("{0}")]
    Unsupported(&'static str),
}

impl ser::Error for SerError {
    fn custom<T: Display>(msg: T) -> Self {
        SerError::SerializeError(format!("{}", msg)).into()
    }
}

/// A failure at the serialization layer.
#[derive(Error, Debug)]
pub enum SerErrorWithIo {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    SerError(#[from] SerError),
}

impl SerErrorWithIo {
    pub fn unwrap_ser_error(self) -> SerError {
        match self {
            SerErrorWithIo::Io(_) => {
                panic!("Unwrapped a SerErrorWithIo expecting a SerError but was IO")
            }
            SerErrorWithIo::SerError(x) => x,
        }
    }
}

impl ser::Error for SerErrorWithIo {
    fn custom<T: Display>(msg: T) -> Self {
        SerError::SerializeError(format!("{}", msg)).into()
    }
}
//endregion

/// Tells a sub-serializer how to handle size-prefixing a struct.
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum StructSizeBehavior {
    /// A stat is being serialized to be sent for a directory read.
    /// Prefix its two-bytes size.
    Two,
    /// A stat is being serialized for a stat-related message (twstat or rstat).
    /// Its binary representation is prefixed with yet another two-byte size.
    DoubleTwo,
    /// This struct is not a Stat. Do not size-prefix.
    None,
}

impl StructSizeBehavior {
    pub(crate) fn offset(&self) -> usize {
        match self {
            StructSizeBehavior::DoubleTwo => 4,
            StructSizeBehavior::Two => 2,
            StructSizeBehavior::None => 0,
        }
    }
}

/// The maximum possible length of a byte array in 9p.
pub(crate) const BYTES_LEN_MAX: u32 = u32::MAX - 11; // 4 for message size, 1 for type, 2 for tag, 4 for byte length
pub(crate) const STRUCT_SIZE_TWO_MAX: u32 = u16::MAX as u32 - 2;
pub(crate) const STRUCT_SIZE_DOUBLE_TWO_MAX: u32 = u16::MAX as u32 - 4;

//region Unimplemented
/// Stand-in code for types of serialization that will never happen
/// because the types are unspecified.
pub struct Unimplemented<Ok, Err> {
    _ok: std::marker::PhantomData<Ok>,
    _err: std::marker::PhantomData<Err>,
    _never: Never,
}
pub enum Never {}

impl<Ok, Err: ser::Error> SerializeMap for Unimplemented<Ok, Err> {
    type Ok = Ok;
    type Error = Err;
    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }
    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok, Err: ser::Error> SerializeTupleVariant for Unimplemented<Ok, Err> {
    type Ok = Ok;
    type Error = Err;
    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok, Err: ser::Error> SerializeStructVariant for Unimplemented<Ok, Err>{
    type Ok = Ok;
    type Error = Err;
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
//endregion
