//! `common` contains types useful across multiple protocol versions.

use serde::{
    de::{Deserialize, Deserializer, Visitor}, ser::{Serialize, Serializer},
};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::ops::Deref;
use std::convert::AsRef;

/// A CowStr allows hard-coded strings to be used in places.
pub type CowStr = Cow<'static, str>;

/// Data wraps a Vec<u8> so that a deserializer's deserialize_byte_buf
/// is used instead of treating it like any other sequence.
#[derive(Debug, PartialEq, Eq)]
pub struct Data(pub Vec<u8>);

impl Deref for Data {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for Data {
    fn from(v: Vec<u8>) -> Data {
        Data(v.into())
    }
}

impl From<Data> for Vec<u8> {
    fn from(d: Data) -> Self {
        d.0
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GimmeBytesVisitor;

        impl<'de> Visitor<'de> for GimmeBytesVisitor {
            type Value = Data;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "some sorta bytes")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Data(v.into()))
            }

            fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
                Ok(Data(v.into()))
            }

            fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Ok(Data(v))
            }
        }

        deserializer.deserialize_byte_buf(GimmeBytesVisitor)
    }
}

impl Serialize for Data {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

/// Each message type has an associate type ID.
pub trait MessageTypeId {
    const MSG_TYPE_ID: u8;

    fn msg_type_id(&self) -> u8 {
        Self::MSG_TYPE_ID
    }
}

/// Helper for message_type_ids.
/// Implements the MessageTypeId trait for the given type.
macro_rules! message_type_id {
    ($mtype:ty, $id:expr) => {
        impl $crate::common::MessageTypeId for $mtype {
            const MSG_TYPE_ID: u8 = $id;
        }
    };
}

/// Allows you to write message type IDs all at once, similar
/// to how they'd be written in in Fcall.h.
macro_rules! message_type_ids {
    {$($mtype:ty = $id:expr),*} => {
        $(
            message_type_id!($mtype, $id);
        )*
    }
}
