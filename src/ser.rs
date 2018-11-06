//! Serialization related code.
use byteorder::{LittleEndian, WriteBytesExt};
pub use common::*;
use failure::{Compat, Fail};
pub use serde::ser::Serialize;
use serde::ser::{self, *};
use std::error::Error;
use std::fmt;
use std::io::{self, Cursor, Seek, SeekFrom, Write};

/// A custom serialization error.
#[derive(Debug)]
pub struct SerializeError(String);

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SerializeError {
    fn description(&self) -> &str {
        &self.0
    }
}

#[derive(Fail, Debug)]
pub enum SerFail {
    #[fail(display = "IO Error: {}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "Serialize Error: {}", _0)]
    SerializeError(#[cause] SerializeError),
    #[fail(display = "String was too long by {} bytes", _0)]
    StringTooLong(usize),
    #[fail(display = "Byte buffer was too long by {} bytes", _0)]
    BytesTooLong(usize),
    #[fail(display = "Sequence too long")]
    SeqTooLong,
    #[fail(display = "Total size was bigger than u64")]
    TooBig,
    #[fail(display = "Type {} is unspecified in 9p", _0)]
    UnspecifiedType(&'static str),
}

impl From<io::Error> for SerFail {
    fn from(src: io::Error) -> Self {
        SerFail::Io(src)
    }
}

/// A wrapper for a `SerFail` that has it implement `std::error::Error`.
/// This is necessary because of the blanket impl of `Fail` for `std::error::Error`,
/// and because `Serializer` requires the error type to implement `std::error::Error`.
pub struct SerError(pub Compat<SerFail>);

impl fmt::Display for SerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl fmt::Debug for SerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Error for SerError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        Error::cause(&self.0)
    }
}

impl ser::Error for SerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerFail::SerializeError(SerializeError(format!("{}", msg))).into()
    }
}

impl<T> From<T> for SerError
where
    T: Into<SerFail>,
{
    fn from(t: T) -> Self {
        SerError(t.into().compat())
    }
}

/// A serializer that works with any type that implements `Write` and `Seek`.
#[derive(Debug)]
pub struct WriteSerializer<W: Write + Seek> {
    pub writer: W,
    in_stat: bool, // if in a rstat/twstat message, double-prefix the size
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

    fn seek_fwd(&mut self, mut amount: usize) -> io::Result<u64> {
        if amount > u32::max_value() as usize {
            self.writer
                .seek(SeekFrom::Current(u32::max_value() as i64))?;
            amount -= u32::max_value() as usize
        }

        self.writer.seek(SeekFrom::Current(amount as i64))
    }

    fn seek_back(&mut self, mut amount: usize) -> io::Result<u64> {
        if amount > u32::max_value() as usize {
            self.writer
                .seek(SeekFrom::Current(-(u32::max_value() as i64)))?;
            amount -= u32::max_value() as usize
        }

        self.writer.seek(SeekFrom::Current(-(amount as i64)))
    }
}

// TODO: serializing non-9p types should not be a panic
impl<'ser, W: 'ser + Write + Seek> Serializer for &'ser mut WriteSerializer<W> {
    type Ok = usize;
    type Error = SerError;

    type SerializeSeq = CountingSequenceSerializer<'ser, W>;
    type SerializeTuple = AccountingStructSerializer<'ser, W>;
    type SerializeTupleStruct = AccountingStructSerializer<'ser, W>;
    type SerializeTupleVariant = AccountingStructSerializer<'ser, W>;
    type SerializeMap = Unimplemented;
    type SerializeStruct = AccountingStructSerializer<'ser, W>;
    type SerializeStructVariant = AccountingStructSerializer<'ser, W>;

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
        Err(SerFail::UnspecifiedType("f32").into())
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(SerFail::UnspecifiedType("f64").into())
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerFail::UnspecifiedType("char").into())
    }
    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        const STR_MAX: usize = u16::max_value() as usize;
        let len = s.len();

        if len > STR_MAX {
            return Err(SerFail::StringTooLong(len - STR_MAX).into())
        }

        let len = len as u16;
        self.writer.write_u16::<LittleEndian>(len)?;
        self.writer.write(s.as_bytes())?;

        Ok(len as usize + 2)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        const BYTE_MAX: usize = u32::max_value() as usize;
        let len = v.len();

        if len > BYTE_MAX {
            return Err(SerFail::BytesTooLong(len - BYTE_MAX).into());
        }

        let len = len as u32;
        self.writer.write_u32::<LittleEndian>(len as u32)?;
        self.writer.write_all(v)?;
        Ok(len as usize + 4)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerFail::UnspecifiedType("none").into())
    }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(SerFail::UnspecifiedType("some").into())
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerFail::UnspecifiedType("unit").into())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerFail::UnspecifiedType("unit struct").into())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerFail::UnspecifiedType("unit variant").into())
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
        Err(SerFail::UnspecifiedType("newtype variant").into())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
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
            running_total: 0,
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
            running_total: 0,
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
        Err(SerFail::UnspecifiedType("tuple variant").into())
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerFail::UnspecifiedType("map").into())
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
            running_total: 0,
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
        Err(SerFail::UnspecifiedType("struct_variant").into())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

/// A sequence serializer that counts how many items it gets and then prefixes
/// with the 2-byte count.
#[derive(Debug)]
pub struct CountingSequenceSerializer<'ser, W: 'ser + Write + Seek> {
    serializer: &'ser mut WriteSerializer<W>,
    current_count: u16,
    byte_count: usize,
}

impl<'ser, W: 'ser + Write + Seek> SerializeSeq for CountingSequenceSerializer<'ser, W> {
    type Ok = usize;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.current_count = self
            .current_count
            .checked_add(1)
            .ok_or(SerFail::SeqTooLong)?;
        self.byte_count = self
            .byte_count
            .checked_add(value.serialize(&mut *self.serializer)?)
            .ok_or(SerFail::TooBig)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // TODO: this can be converted safely (multiple seeks (make a helper func))
        self.serializer.seek_back(self.byte_count)?;
        self.serializer.seek_back(2)?;
        self.current_count.serialize(&mut *self.serializer)?;
        self.serializer.seek_fwd(self.byte_count)?;
        Ok(self.byte_count + 2)
    }
}

/// There are two special cases for nested structs in 9p messages, and they're both
/// Stat.
#[derive(Debug)]
enum StructSizeBehavior {
    /// A stat is being serialized to be sent for a directory read.
    /// Prefix its two-bytes size.
    Two,
    /// A stat is being serialized for a stat-related message (twstat or rstat).
    /// Its binary representation is prefixed with yet another two-byte size.
    DoubleTwo,
    /// This struct is not a Stat. Ignore the above.
    None,
}

/// A struct serializer that counts the byte size of everything serialized so far.
#[derive(Debug)]
pub struct AccountingStructSerializer<'ser, W: 'ser + Write + Seek> {
    serializer: &'ser mut WriteSerializer<W>,
    running_total: usize,
    size_behavior: StructSizeBehavior,
}

impl<'ser, W: 'ser + Write + Seek> SerializeStruct for AccountingStructSerializer<'ser, W> {
    type Ok = usize;
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

impl<'ser, W: 'ser + Write + Seek> SerializeTuple for AccountingStructSerializer<'ser, W> {
    type Ok = usize;
    type Error = SerError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.running_total = self
            .running_total
            .checked_add(value.serialize(&mut *self.serializer)?)
            .ok_or(SerFail::TooBig)?;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // TODO: this can be converted safely (multiple seeks (make a helper func))
        Ok(match self.size_behavior {
            StructSizeBehavior::None => self.running_total,
            StructSizeBehavior::Two => {
                self.serializer.seek_back(self.running_total)?;
                self.serializer.seek_back(2)?;
                (self.running_total as u16).serialize(&mut *self.serializer)?;
                self.serializer.seek_fwd(self.running_total)?;
                self.running_total + 2
            }
            StructSizeBehavior::DoubleTwo => {
                self.serializer.seek_back(self.running_total)?;
                self.serializer.seek_back(4)?;
                (self.running_total as u16 + 2).serialize(&mut *self.serializer)?;
                (self.running_total as u16 + 0).serialize(&mut *self.serializer)?;
                self.serializer.seek_fwd(self.running_total)?;
                self.running_total + 4
            }
        })
    }
}

impl<'ser, W: 'ser + Write + Seek> SerializeTupleStruct for AccountingStructSerializer<'ser, W> {
    type Ok = usize;
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

impl<'ser, W: 'ser + Write + Seek> SerializeTupleVariant for AccountingStructSerializer<'ser, W> {
    type Ok = usize;
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

impl<'ser, W: 'ser + Write + Seek> SerializeStructVariant for AccountingStructSerializer<'ser, W> {
    type Ok = usize;
    type Error = SerError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

/// Stand-in code for types of serialization that will never happen.
pub enum Unimplemented {}

impl SerializeMap for Unimplemented {
    type Ok = usize;
    type Error = SerError;
    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }
    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// TODO: this is mainly used for converting stat calls.
// It should take a ref to the writer, which would be the existing buffer on the dir.
/// Serialize the given object into a new vec buffer and return the result.
pub fn into_bytes<T: Serialize>(t: &T) -> Vec<u8> {
    let mut ser = WriteSerializer {
        writer: Cursor::new(Vec::new()),
        in_stat: false,
    };

    t.serialize(&mut ser).unwrap();

    ser.into_writer().into_inner()
}

/// Serializes the given item into the given type that implements write and seek.
/// This is typically a file or a buffer.
///
/// Returns the number of bytes written.
pub fn into_write_seeker<T: Serialize, W: Write + Seek>(
    t: &T,
    writer: W,
) -> Result<usize, SerError> {
    let mut ser = WriteSerializer {
        writer,
        in_stat: false,
    };

    t.serialize(&mut ser)
}

// TODO: document the error conditions better.
/// Serializes the given item into the given buffer.
///
/// Returns the number of bytes written.
///
/// Can fail if the buffer isn't big enough, among other reasons.
pub fn into_buf<T: Serialize, B: AsMut<[u8]>>(t: &T, mut buf: B) -> Result<usize, SerError> {
    let writer = Cursor::new(buf.as_mut());
    into_write_seeker(t, writer)
}

/// Serializes the given item into the given Vec.
///
/// Returns the number of bytes written.
///
/// Will typically not fail because of space issues.
pub fn into_vec<T: Serialize, V: AsMut<Vec<u8>>>(
    t: &T,
    mut vec: V,
) -> Result<usize, SerError> {
    let writer = Cursor::new(vec.as_mut());
    into_write_seeker(t, writer)
}
