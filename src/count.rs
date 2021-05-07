pub use crate::common::*;
pub use serde::ser::Serialize;
use serde::ser::{self, *};
use std::{u16, u32};
use std::fmt;
use thiserror::Error;

/// The maximum possible length of a byte array in 9p.
pub const BYTES_LEN_MAX: u32 = u32::MAX - 8; // 4 for message size, 4 for byte length

/// A failure at the serialization lagyer.
#[derive(Error, Debug)]
pub enum CountError {
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
}

impl ser::Error for CountError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        CountError::SerializeError(format!("{}", msg)).into()
    }
}

#[derive(Debug)]
struct SizeCounterSerializer {
    in_stat: bool, // if in a rstat/twstat message, double-prefix the size
}

impl<'ser> Serializer for &'ser mut SizeCounterSerializer {
    type Ok = u32;
    type Error = CountError; // TODO overflow

    type SerializeSeq = CountingSequenceSerializer<'ser>;
    type SerializeTuple = AccountingStructSerializer<'ser>;
    type SerializeTupleStruct = AccountingStructSerializer<'ser>;
    type SerializeTupleVariant = Unimplemented;
    type SerializeMap = Unimplemented;
    type SerializeStruct = AccountingStructSerializer<'ser>;
    type SerializeStructVariant = Unimplemented;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(1)
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(1)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(2)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(4)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(8)
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(1)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(2)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(4)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(8)
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("f32"))
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("f64"))
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("char"))
    }
    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        if s.len() > u16::MAX as usize {
            return Err(CountError::StringTooLong);
        }
        let len = s.len() as u16;

        Ok(len as u32 + 2)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if v.len() > BYTES_LEN_MAX as usize {
            return Err(CountError::BytesTooLong);
        }
        Ok(v.len() as u32 + 4)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("none"))
    }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(CountError::UnspecifiedType("some"))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("unit"))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("unit struct"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CountError::UnspecifiedType("unit variant"))
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(CountError::UnspecifiedType("newtype variant"))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            if len > u16::MAX as usize {
                return Err(CountError::SeqTooLong);
            }
        }
        Ok(CountingSequenceSerializer {
            serializer: self,
            current_count: 0,
            byte_count: 0,
        })
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(AccountingStructSerializer {
            serializer: self,
            byte_count: 0,
            size_behavior: StructSizeBehavior::None,
        })
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(AccountingStructSerializer {
            serializer: self,
            byte_count: 0,
            size_behavior: StructSizeBehavior::None,
        })
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(CountError::UnspecifiedType("tuple variant").into())
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(CountError::UnspecifiedType("map").into())
    }
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let size_behavior = if name == "Twstat" || name == "Rstat" {
            self.in_stat = true;
            StructSizeBehavior::None
        } else if name == "Stat" {
            if self.in_stat {
                StructSizeBehavior::DoubleTwo
            } else {
                StructSizeBehavior::Two
            }
        } else {
            StructSizeBehavior::None
        };

        Ok(AccountingStructSerializer {
            serializer: self,
            byte_count: 0,
            size_behavior,
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(CountError::UnspecifiedType("struct_variant"))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

/// A sequence serializer that counts how many items it
/// gets and then prefixes with the 2-byte count.
#[derive(Debug)]
pub struct CountingSequenceSerializer<'ser> {
    serializer: &'ser mut SizeCounterSerializer,
    current_count: u16,
    byte_count: u32,
}

impl<'ser> SerializeSeq for CountingSequenceSerializer<'ser> {
    type Ok = u32;
    type Error = CountError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.current_count = self
            .current_count
            .checked_add(1)
            .ok_or(CountError::SeqTooLong)?;
        let amt = value.serialize(&mut *self.serializer)?;
        self.byte_count = self.byte_count.checked_add(amt).ok_or(CountError::TooBig)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.byte_count.checked_add(2).ok_or(CountError::TooBig)
    }
}

/// Tells a sub-serializer how to handle size-prefixing a struct.
#[derive(Debug)]
enum StructSizeBehavior {
    /// A stat is being serialized to be sent for a directory read.
    /// Prefix its two-bytes size.
    Two,
    /// A stat is being serialized for a stat-related message (twstat or rstat).
    /// Its binary representation is prefixed with yet another two-byte size.
    DoubleTwo,
    /// This struct is not a Stat. Do not size-prefix.
    None,
}

/// A struct serializer that counts the byte size of everything serialized so far.
#[derive(Debug)]
pub struct AccountingStructSerializer<'ser> {
    serializer: &'ser mut SizeCounterSerializer,
    byte_count: u32,
    size_behavior: StructSizeBehavior,
}

impl<'ser> SerializeStruct for AccountingStructSerializer<'ser> {
    type Ok = u32;
    type Error = CountError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeTuple::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

const STRUCT_SIZE_TWO_MAX: u32 = u16::MAX as u32 - 2;
const STRUCT_SIZE_DOUBLE_TWO_MAX: u32 = u16::MAX as u32 - 4;

impl<'ser> SerializeTuple for AccountingStructSerializer<'ser> {
    type Ok = u32;
    type Error = CountError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let amount = value.serialize(&mut *self.serializer)?;
        self.byte_count = self
            .byte_count
            .checked_add(amount)
            .ok_or(CountError::TooBig)?;

        // TODO: is it quicker to check during end instead?
        match self.size_behavior {
            StructSizeBehavior::Two if self.byte_count > STRUCT_SIZE_TWO_MAX => {
                Err(CountError::TooBig.into())
            }
            StructSizeBehavior::DoubleTwo if self.byte_count > STRUCT_SIZE_DOUBLE_TWO_MAX => {
                Err(CountError::TooBig.into())
            }
            _ => Ok(()),
        }
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(match self.size_behavior {
            StructSizeBehavior::None => self.byte_count,
            StructSizeBehavior::Two => self.byte_count + 2,
            StructSizeBehavior::DoubleTwo => self.byte_count + 4,
        })
    }
}

impl<'ser> SerializeTupleStruct for AccountingStructSerializer<'ser> {
    type Ok = u32;
    type Error = CountError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeTuple::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

/// Stand-in code for types of serialization that will never happen
/// because the types are unspecified.
pub enum Unimplemented {}

impl SerializeMap for Unimplemented {
    type Ok = u32;
    type Error = CountError;
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

impl SerializeTupleVariant for Unimplemented {
    type Ok = u32;
    type Error = CountError;
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

impl SerializeStructVariant for Unimplemented {
    type Ok = u32;
    type Error = CountError;
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

pub fn size_for<T: Serialize>(t: &T) -> Result<u32, CountError> {
    let mut counter = SizeCounterSerializer { in_stat: false };
    t.serialize(&mut counter)
}
