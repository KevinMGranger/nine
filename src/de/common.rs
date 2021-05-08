use serde::de::{self, *};
pub use serde::de::{Deserialize, Deserializer};
use std::fmt::{self, Formatter};
use std::io;
use std::marker::PhantomData;
use std::string::FromUtf8Error;
use thiserror::Error;

// region Byte Arrays
/// A visitor that converts byte slices / vecs into the desired type
struct GimmeBytesVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for GimmeBytesVisitor<T>
where
    for<'a> T: From<Vec<u8>> + From<&'a [u8]>,
{
    type Value = T;
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "some sorta bytes")
    }

    fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.into())
    }

    fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
        Ok(v.into())
    }

    fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        Ok(v.into())
    }
}

/// A deserialize function that produces a Vec<u8> using the correct deserializer method.
pub fn deserialize_owned_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_byte_buf(GimmeBytesVisitor(PhantomData))
}
//endregion

// TODO: deserialize functions for Bytes

//region Error Handling
/// A failure at the deserialization layer.
#[derive(Error, Debug)]
pub enum DeError {
    /// A string was requested, but the data was not valid UTF-8.
    #[error("Invalid UTF8: {0}")]
    Utf8(#[from] FromUtf8Error),
    /// An IO error occurred from the Read source.
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    /// An error came from the deserialize impl.
    #[error("Deserialize Error: {0}")]
    DeserializeError(String),
    /// The serde type is unspecified in 9p.
    #[error("Type {0} is unspecified in 9p")]
    UnspecifiedType(&'static str),
}

impl DeError {
    /// Whether or not the contained error is an io::ErrorKind::UnexpectedEof.
    /// Useful since this can merely mean the client disconnected and is not
    /// necessarily an error.
    pub fn is_eof(&self) -> bool {
        if let DeError::Io(err) = self {
            if let io::ErrorKind::UnexpectedEof = err.kind() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl de::Error for DeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeError::DeserializeError(format!("{}", msg)).into()
    }
}
//endregion
