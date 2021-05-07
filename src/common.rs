//! `common` contains types useful across multiple protocol versions.
//! You shouldn't need to use it unless you're making your own.

use serde::{
    de::{Deserializer, Visitor},
    ser::Serializer,
};
use std::convert::AsRef;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::marker::PhantomData;

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

/// A serializer function that serializes any byte slice like object.
pub fn serialize_bytes<T, S>(t: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_bytes(t.as_ref())
}

pub trait MessageTypeId {
    fn msg_type_id(&self) -> u8;
}

pub trait ConstMessageTypeId {
    const MSG_TYPE_ID: u8;
}

impl<T: ConstMessageTypeId> MessageTypeId for T {
    fn msg_type_id(&self) -> u8 {
        Self::MSG_TYPE_ID
    }
}

/// Helper for message_type_ids.
/// Implements the MessageTypeId trait for the given type.
macro_rules! message_type_id {
    ($mtype:ty, $id:expr) => {
        
    };
}

/// Allows you to write message type IDs all at once, similar
/// to how they'd be written in in Fcall.h.
macro_rules! message_type_ids {
    {$($mtype:ident = $id:expr),*} => {
        $(
            impl $crate::common::ConstMessageTypeId for $mtype {
                const MSG_TYPE_ID: u8 = $id;
            }
        )*
    }
}

pub trait Taggable {
    type Tagged;
    fn tag(self, tag: u16) -> Self::Tagged;
}

///Allows messages to be declared while
macro_rules! messages {
    { 
        $(
    $(#[$structmeta:meta])*
    $name:ident {
        $(
        $(#[$fieldmeta:meta])*
        $field:ident: $type:ty,
        )*
    }
    )* } => {
        pub mod tagged {
            use super::*;
            $(
            $(#[$structmeta])*
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct $name {
                pub tag: u16,
                $(
                $(#[$fieldmeta])*
                pub $field: $type,
                )*
            }
            )*
        }

        pub mod untagged {
            use super::*;
            $(
            $(#[$structmeta])*
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct $name {
                $(
                $(#[$fieldmeta])*
                pub $field: $type,
                )*
            }

            impl $crate::common::Taggable for $name {
                type Tagged = super::tagged::$name;

                fn tag(self, tag: u16) -> Self::Tagged {
                    Self::Tagged {
                        tag,
                        $(
                            $field: self.$field,
                        )*
                    }
                }
            }
            )*
        }
    }
}