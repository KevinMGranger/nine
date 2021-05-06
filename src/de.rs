//! Deserializers and deserializer convenience functions.

use byteorder::{LittleEndian, ReadBytesExt};

pub use serde::de::Deserialize;
use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use thiserror::Error;
use std::fmt;
use std::io::{self, Cursor, Read};
use std::string::FromUtf8Error;

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

/// A read deserializer can deserialize the 9p data format from any type
/// that implements `std::io::Read`.
pub struct ReadDeserializer<R: Read>(pub R);

type ORD = LittleEndian;

impl<'a, 'de: 'a, R: Read> Deserializer<'de> for &'a mut ReadDeserializer<R> {
    type Error = DeError;

    // could we have someone define a "protocol" and let deserialize_any be the workaround
    // for protocol sets? No need for Length::Zero since messages wouldn't go through
    // deserialize_struct directly
    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(DeError::UnspecifiedType("any").into())
    }

    // TODO: do bools actually appear anywhere in 9p?
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.0.read_u8()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.0.read_u8()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u16(self.0.read_u16::<ORD>()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.0.read_u32::<ORD>()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.0.read_u64::<ORD>()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.0.read_i8()?)
    }
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.0.read_i16::<ORD>()?)
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.0.read_i32::<ORD>()?)
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.0.read_i64::<ORD>()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("f32").into())
    }
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("f64").into())
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("char").into())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len: u16 = Deserialize::deserialize(&mut *self)?;
        let mut buf = vec![0u8; len as usize];

        self.0.read_exact(buf.as_mut_slice())?;

        visitor.visit_byte_buf(buf)
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len: u32 = Deserialize::deserialize(&mut *self)?;

        let mut buf = vec![0u8; len as usize];

        self.0.read_exact(buf.as_mut_slice())?;

        visitor.visit_byte_buf(buf)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("option").into())
    }
    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("unit").into())
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("unit_struct").into())
    }
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(CountedVecReader {
            remain: Deserialize::deserialize(&mut *self)?,
            des: &mut *self,
        })
    }
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }
    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("map").into())
    }
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if name == "Stat" {
            let _ = self.0.read_u32::<ORD>()?;
        }

        visitor.visit_seq(self)
    }
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("enum").into())
    }
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("identifier").into())
    }
    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::UnspecifiedType("ignored_any").into())
    }
}

impl<'de, R: Read> SeqAccess<'de> for ReadDeserializer<R> {
    type Error = DeError;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Ok(Some(seed.deserialize(self)?))
    }
}

struct CountedVecReader<'a, R: Read + 'a> {
    remain: u16,
    des: &'a mut ReadDeserializer<R>,
}

impl<'a, 'de: 'a, R: Read + 'a> SeqAccess<'de> for CountedVecReader<'a, R> {
    type Error = DeError;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remain == 0 {
            Ok(None)
        } else {
            self.remain -= 1;
            Ok(Some(seed.deserialize(&mut *self.des)?))
        }
    }
}

/// Deserialize from any type that implements `io::Read`.
pub fn from_reader<'de, T: Deserialize<'de>, R: Read>(reader: R) -> Result<T, DeError> {
    let mut des = ReadDeserializer(reader);
    <T as Deserialize<'de>>::deserialize(&mut des)
}

/// Deserialize from any byte slice.
/// ```
/// # use nine::de::*;
///
/// let bytes = [1u8, 0u8];
/// let num: u16 = from_bytes(&bytes).unwrap();
/// assert_eq!(num, 1u16);
/// ```
pub fn from_bytes<'de, T: Deserialize<'de>, B: AsRef<[u8]>>(bytes: B) -> Result<T, DeError> {
    let cursor = Cursor::new(bytes.as_ref());
    let mut des = ReadDeserializer(cursor);
    <T as Deserialize<'de>>::deserialize(&mut des)
}
