use super::common::{self, *};
use super::count::size_for;
use bytes::{BufMut, BytesMut};
use serde::ser::*;

pub fn into_new_bytes<T: Serialize>(t: &T) -> Result<BytesMut, SerError> {
    let ser = BytesSerializer {
        buf: BytesMut::with_capacity(size_for(t)? as usize),
        in_stat: false,
    };
    Ok(t.serialize(ser)?.buf)
}

pub fn into_existing_bytes<T: Serialize>(t: &T, buf: &mut BytesMut) -> Result<(), SerError> {
    buf.reserve(size_for(t)? as usize);
    let mut ser = BytesSerializer {
        buf: buf.split_off(buf.len()),
        in_stat: false,
    };
    ser = t.serialize(ser)?;
    buf.unsplit(ser.buf);
    Ok(())
}

#[derive(Debug)]
struct BytesSerializer {
    buf: BytesMut,
    in_stat: bool, // if in a rstat/twstat message, double-prefix the size
}

type Unimplemented = common::Unimplemented<BytesSerializer, SerError>;
impl Serializer for BytesSerializer {
    type Ok = BytesSerializer;
    type Error = SerError;

    type SerializeSeq = CountingSequenceSerializer;
    type SerializeTuple = AccountingStructSerializer;
    type SerializeTupleStruct = AccountingStructSerializer;
    type SerializeTupleVariant = Unimplemented;
    type SerializeMap = Unimplemented;
    type SerializeStruct = AccountingStructSerializer;
    type SerializeStructVariant = Unimplemented;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u8(v as u8);
        Ok(self)
    }
    fn serialize_i8(mut self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.buf.put_i8(v);
        Ok(self)
    }
    fn serialize_i16(mut self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.buf.put_i16_le(v);
        Ok(self)
    }
    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.buf.put_i32_le(v);
        Ok(self)
    }
    fn serialize_i64(mut self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.buf.put_i64_le(v);
        Ok(self)
    }
    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u8(v);
        Ok(self)
    }
    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u16_le(v);
        Ok(self)
    }
    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u32_le(v);
        Ok(self)
    }
    fn serialize_u64(mut self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u64_le(v);
        Ok(self)
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("f32").into())
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("f64").into())
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("char").into())
    }
    fn serialize_str(mut self, s: &str) -> Result<Self::Ok, Self::Error> {
        if s.len() > u16::MAX as usize {
            return Err(SerError::StringTooLong.into());
        }
        let len = s.len() as u16;
        self.buf.put_u16_le(len);
        self.buf.put(s.as_bytes());

        Ok(self)
    }
    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if v.len() > BYTES_LEN_MAX as usize {
            return Err(SerError::BytesTooLong.into());
        }
        self.buf.put_u32_le(v.len() as u32);
        self.buf.put(v);
        Ok(self)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("none").into())
    }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(SerError::UnspecifiedType("some").into())
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("unit").into())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("unit struct").into())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerError::UnspecifiedType("unit variant").into())
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
        Err(SerError::UnspecifiedType("newtype variant").into())
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            if len > u16::MAX as usize {
                return Err(SerError::SeqTooLong.into());
            }
        }

        Ok(CountingSequenceSerializer::new(self))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(AccountingStructSerializer::new(
            self,
            StructSizeBehavior::None,
        ))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(AccountingStructSerializer::new(
            self,
            StructSizeBehavior::None,
        ))
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
        mut self,
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

        Ok(AccountingStructSerializer::new(self, size_behavior))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerError::UnspecifiedType("struct_variant").into())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

/// A sequence serializer that counts how many items it
/// gets and then prefixes with the 2-byte count.
#[derive(Debug)]
pub struct CountingSequenceSerializer {
    current_count: u16,
    before: BytesMut,
    count_bytes: BytesMut,
    current: BytesSerializer,
}

impl CountingSequenceSerializer {
    fn new(mut ser: BytesSerializer) -> Self {
        let before = ser.buf.split();
        let count_bytes = ser.buf.split_to(2);
        CountingSequenceSerializer {
            current_count: 0,
            current: ser,
            count_bytes,
            before,
        }
    }
}

impl SerializeSeq for CountingSequenceSerializer {
    type Ok = BytesSerializer;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.current_count = self
            .current_count
            .checked_add(1)
            .ok_or(SerError::SeqTooLong)?;
        //TODO: do we need to keep track of in_struct ourselves here?
        self.current = value.serialize(self.current)?;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.count_bytes.put_u16_le(self.current_count);
        let mut buf = self.before;
        buf.unsplit(self.count_bytes);
        buf.unsplit(self.current.buf);

        Ok(BytesSerializer {
            buf,
            ..self.current
        })
    }
}

/// A struct serializer that counts the byte size of everything serialized so far.
#[derive(Debug)]
pub struct AccountingStructSerializer {
    before: BytesMut,
    size_behavior: StructSizeBehavior,
    size_bytes: BytesMut,
    current: BytesSerializer,
}

impl AccountingStructSerializer {
    fn new(mut ser: BytesSerializer, size_behavior: StructSizeBehavior) -> Self {
        let before = ser.buf.split();
        let size_bytes = ser.buf.split_to(size_behavior.offset());
        Self {
            before,
            size_behavior,
            size_bytes,
            current: ser,
        }
    }
}

impl SerializeStruct for AccountingStructSerializer {
    type Ok = BytesSerializer;
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

// size[4] Rstat tag[2] stat[n]

impl SerializeTuple for AccountingStructSerializer {
    type Ok = BytesSerializer;
    type Error = SerError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let in_stat = self.current.in_stat;

        let returned_current = value.serialize(self.current)?;

        self.current = BytesSerializer {
            in_stat,
            ..returned_current
        };

        Ok(())
    }
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        // TODO max size checking (or in element?)
        match self.size_behavior {
            StructSizeBehavior::None => {}
            StructSizeBehavior::Two => {
                let byte_count = self.current.buf.len();
                self.size_bytes.put_u16_le(byte_count as u16);
            }
            StructSizeBehavior::DoubleTwo => {
                let byte_count = self.current.buf.len();
                self.size_bytes.put_u16_le(byte_count as u16 + 2);
                self.size_bytes.put_u16_le(byte_count as u16);
            }
        }

        let mut buf = self.before;
        buf.unsplit(self.size_bytes);
        buf.unsplit(self.current.buf);

        Ok(BytesSerializer {
            buf,
            ..self.current
        })
    }
}

impl SerializeTupleStruct for AccountingStructSerializer {
    type Ok = BytesSerializer;
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
