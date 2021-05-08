use super::common::{self, *};
use byteorder::{WriteBytesExt, LittleEndian};
use serde::ser::*;
use std::io::{self, Seek, SeekFrom, Write};

/// Serializes the given item into the given type that implements write and seek.
/// This is typically a file or a buffer (`io::Cursor<Vec<u8>>`).
///
/// Returns the number of bytes written.
pub fn into_write_seeker<T: Serialize, W: Write + Seek>(t: &T, writer: W) -> Result<u32, SerErrorWithIo> {
    let mut ser = WriteSerializer {
        writer,
        in_stat: false,
    };

    t.serialize(&mut ser)
}

/// A serializer that works with any type that implements `Write` and `Seek`.
#[derive(Debug)]
pub struct WriteSerializer<W: Write + Seek> {
    pub writer: W,
    pub(crate) in_stat: bool, // if in a rstat/twstat message, double-prefix the size
}

impl<W: Write + Seek> WriteSerializer<W> {
    /// Create a serializer from the given Writer.
    pub fn new(writer: W) -> WriteSerializer<W> {
        WriteSerializer {
            writer,
            in_stat: false,
        }
    }
    /// Consume the serializer, giving back the writer.
    pub fn into_writer(self) -> W {
        self.writer
    }

    /// Seek the writer forward the given amount.
    fn seek_fwd(&mut self, amount: u32) -> io::Result<u64> {
        self.writer.seek(SeekFrom::Current(amount as i64))
    }

    /// Seek the writer backward the given amount.
    fn seek_back(&mut self, amount: u32) -> io::Result<u64> {
        self.writer.seek(SeekFrom::Current(-(amount as i64)))
    }
}

type Unimplemented = common::Unimplemented<u32, SerErrorWithIo>;
impl<'ser, W: 'ser + Write + Seek> Serializer for &'ser mut WriteSerializer<W> {
    type Ok = u32;
    type Error = SerErrorWithIo;

    type SerializeSeq = CountingSequenceSerializer<'ser, W>;
    type SerializeTuple = AccountingStructSerializer<'ser, W>;
    type SerializeTupleStruct = AccountingStructSerializer<'ser, W>;
    type SerializeTupleVariant = Unimplemented;
    type SerializeMap = Unimplemented;
    type SerializeStruct = AccountingStructSerializer<'ser, W>;
    type SerializeStructVariant = Unimplemented;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(v as u8)?;
        Ok(1)
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_i8(v)?;
        Ok(1)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_i16::<LittleEndian>(v)?;
        Ok(2)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_i32::<LittleEndian>(v)?;
        Ok(4)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_i64::<LittleEndian>(v)?;
        Ok(8)
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(v)?;
        Ok(1)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u16::<LittleEndian>(v)?;
        Ok(2)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u32::<LittleEndian>(v)?;
        Ok(4)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u64::<LittleEndian>(v)?;
        Ok(8)
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
    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        if s.len() > u16::MAX as usize {
            return Err(SerError::StringTooLong.into());
        }
        let len = s.len() as u16;
        self.writer.write_u16::<LittleEndian>(len)?;
        self.writer.write(s.as_bytes())?;

        Ok(len as u32 + 2)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if v.len() > BYTES_LEN_MAX as usize {
            return Err(SerError::BytesTooLong.into());
        }
        self.writer.write_u32::<LittleEndian>(v.len() as u32)?;
        self.writer.write_all(v)?;
        Ok(v.len() as u32 + 4)
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

        self.seek_fwd(2)?;
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
                self.seek_fwd(4)?;
                StructSizeBehavior::DoubleTwo
            } else {
                self.seek_fwd(2)?;
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
        Err(SerError::UnspecifiedType("struct_variant").into())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

/// A sequence serializer that counts how many items it
/// gets and then prefixes with the 2-byte count.
#[derive(Debug)]
pub struct CountingSequenceSerializer<'ser, W: 'ser + Write + Seek> {
    serializer: &'ser mut WriteSerializer<W>,
    current_count: u16,
    byte_count: u32,
}

impl<'ser, W: 'ser + Write + Seek> SerializeSeq for CountingSequenceSerializer<'ser, W> {
    type Ok = u32;
    type Error = SerErrorWithIo;

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
        self.serializer.seek_back(self.byte_count)?;
        self.serializer.seek_back(2)?;
        self.current_count.serialize(&mut *self.serializer)?;
        self.serializer.seek_fwd(self.byte_count)?;
        self.byte_count
            .checked_add(2)
            .ok_or(SerError::TooBig.into())
    }
}

/// A struct serializer that counts the byte size of everything serialized so far.
#[derive(Debug)]
pub struct AccountingStructSerializer<'ser, W: 'ser + Write + Seek> {
    serializer: &'ser mut WriteSerializer<W>,
    byte_count: u32,
    size_behavior: StructSizeBehavior,
}

impl<'ser, W: 'ser + Write + Seek> SerializeStruct for AccountingStructSerializer<'ser, W> {
    type Ok = u32;
    type Error = SerErrorWithIo;
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
impl<'ser, W: 'ser + Write + Seek> SerializeTuple for AccountingStructSerializer<'ser, W> {
    type Ok = u32;
    type Error = SerErrorWithIo;
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
            StructSizeBehavior::Two => {
                self.serializer.seek_back(self.byte_count)?;
                self.serializer.seek_back(2)?;
                (self.byte_count as u16).serialize(&mut *self.serializer)?;
                self.serializer.seek_fwd(self.byte_count)?;
                self.byte_count + 2
            }
            StructSizeBehavior::DoubleTwo => {
                self.serializer.seek_back(self.byte_count)?;
                self.serializer.seek_back(4)?;
                (self.byte_count as u16 + 2).serialize(&mut *self.serializer)?;
                (self.byte_count as u16 + 0).serialize(&mut *self.serializer)?;
                self.serializer.seek_fwd(self.byte_count)?;
                self.byte_count + 4
            }
        })
    }
}

impl<'ser, W: 'ser + Write + Seek> SerializeTupleStruct for AccountingStructSerializer<'ser, W> {
    type Ok = u32;
    type Error = SerErrorWithIo;
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