//! Deserialization related code.
pub use serde::de::Deserialize;
use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use byteorder::{LittleEndian, ReadBytesExt};
use failure::{Compat, Fail};
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::string::FromUtf8Error;

// TODO: represent the other deserialize errors as well?
/// A custom deserialize error.
#[derive(Debug)]
pub struct DeserializeError(String);

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Error for DeserializeError {
    fn description(&self) -> &str {
        &self.0
    }
}

/// A failure at the deserialization layer.
#[derive(Fail, Debug)]
pub enum DeFail {
    /// A string was requested, but the data was not valid UTF-8.
    #[fail(display = "Invalid UTF8: {}", _0)]
    Utf8(#[cause] FromUtf8Error),
    /// An IO error occurred from the Read source.
    #[fail(display = "IO Error: {}", _0)]
    Io(#[cause] io::Error),
    /// An error came from the deserialize impl.
    #[fail(display = "Deserialize Error: {}", _0)]
    DeserializeError(#[cause] DeserializeError),
}

impl DeFail {
    /// Whether or not the contained error is an io::ErrorKind::UnexpectedEof.
    /// Useful since this can merely mean the client disconnected and is not
    /// necessarily an error.
    pub fn is_eof(&self) -> bool {
        if let DeFail::Io(err) = self {
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

impl From<io::Error> for DeFail {
    fn from(src: io::Error) -> Self {
        DeFail::Io(src)
    }
}

impl From<FromUtf8Error> for DeFail {
    fn from(src: FromUtf8Error) -> Self {
        DeFail::Utf8(src)
    }
}

/// A wrapper for a `DeFail` that has it implement `std::error::Error`.
///This is necessary because of the blanket impl of `Fail` for `std::error::Error`,
/// and because `Deserializer` requires the error type to implement `std::error::Error`.
pub struct DeError(pub Compat<DeFail>);

impl fmt::Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl fmt::Debug for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Error for DeError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        Error::cause(&self.0)
    }
}

impl de::Error for DeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeFail::DeserializeError(DeserializeError(format!("{}", msg))).into()
    }
}


impl<T> From<T> for DeError
where
    T: Into<DeFail>,
{
    fn from(t: T) -> Self {
        DeError(t.into().compat())
    }
}

/// A read deserializer can deserialize the 9p data format from any type
/// that implemented `std::io::Read`.
pub struct ReadDeserializer<R: Read>(pub R);

type ORD = LittleEndian;

// TODO: deserializing data types that don't appear in 9p shouldn't be a panic.
impl<'a, 'de: 'a, R: Read> Deserializer<'de> for &'a mut ReadDeserializer<R> {
    type Error = DeError;

    // could we have someone define a "protocol" and let deserialize_any be the workaround
    // for protocol sets? No need for Length::Zero since messages wouldn't go through
    // deserialize_struct directly
    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!()
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
        unimplemented!()
    }
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
        unimplemented!()
    }
    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
        unimplemented!()
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
        unimplemented!()
    }
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
