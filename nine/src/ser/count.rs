
use super::common::{self, *};
use serde::ser::*;

#[derive(Debug)]
struct SizeCounterSerializer {
    in_stat: bool, // if in a rstat/twstat message, double-prefix the size
}

type Unimplemented = common::Unimplemented<u32, SerError>;
impl<'ser> Serializer for &'ser mut SizeCounterSerializer {
    type Ok = u32;
    type Error = SerError; // TODO overflow

    type SerializeSeq = CountingSequenceSerializer<'ser>;
    type SerializeTuple = AccountingStructSerializer<'ser>;
    type SerializeTupleStruct = AccountingStructSerializer<'ser>;
    type SerializeTupleVariant = Unimplemented;
    type SerializeMap = Unimplemented;
    type SerializeStruct = AccountingStructSerializer<'ser>;
    type SerializeStructVariant = Unimplemented;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(1)
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(1)
    }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(2)
    }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(4)
    }
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(8)
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(1)
    }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(2)
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(4)
    }
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(8)
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("f32"))
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("f64"))
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("char"))
    }
    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        if s.len() > u16::MAX as usize {
            return Err(SerError::StringTooLong);
        }
        let len = s.len() as u16;

        Ok(len as u32 + 2)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if v.len() > BYTES_LEN_MAX as usize {
            return Err(SerError::BytesTooLong);
        }
        Ok(v.len() as u32 + 4)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("none"))
    }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(SerError::UnspecifiedType("some"))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("unit"))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("unit struct"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("unit variant"))
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
        Err(SerError::UnspecifiedType("newtype variant"))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            if len > u16::MAX as usize {
                return Err(SerError::SeqTooLong);
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
        Err(SerError::UnspecifiedType("tuple variant").into())
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerError::UnspecifiedType("map").into())
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
        Err(SerError::UnspecifiedType("struct_variant"))
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
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.current_count = self
            .current_count
            .checked_add(1)
            .ok_or(SerError::SeqTooLong)?;
        let amt = value.serialize(&mut *self.serializer)?;
        self.byte_count = self.byte_count.checked_add(amt).ok_or(SerError::TooBig)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.byte_count.checked_add(2).ok_or(SerError::TooBig)
    }
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
    type Error = SerError;
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
    type Error = SerError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let amount = value.serialize(&mut *self.serializer)?;
        self.byte_count = self
            .byte_count
            .checked_add(amount)
            .ok_or(SerError::TooBig)?;

        // TODO: is it quicker to check during end instead?
        match self.size_behavior {
            StructSizeBehavior::Two if self.byte_count > STRUCT_SIZE_TWO_MAX => {
                Err(SerError::TooBig.into())
            }
            StructSizeBehavior::DoubleTwo if self.byte_count > STRUCT_SIZE_DOUBLE_TWO_MAX => {
                Err(SerError::TooBig.into())
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
    type Error = SerError;
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

//region Unimplemented
/// Stand-in code for types of serialization that will never happen
/// because the types are unspecified.
enum Unimplemented {}

impl SerializeMap for Unimplemented {
    type Ok = u32;
    type Error = SerError;
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
    type Error = SerError;
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
    type Error = SerError;
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

pub fn size_for<T: Serialize>(t: &T) -> Result<u32, SerError> {
    let mut counter = SizeCounterSerializer { in_stat: false };
    t.serialize(&mut counter)
}