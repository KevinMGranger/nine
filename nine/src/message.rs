//! `message` contains traits and macros used to help define messages.

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

/// Allows you to write message type IDs all at once, similar
/// to how they'd be written in in Fcall.h.
#[macro_export]
macro_rules! message_type_ids {
    {$($mtype:ident = $id:expr),*} => {
        $(
            impl $crate::message::ConstMessageTypeId for $mtype {
                const MSG_TYPE_ID: u8 = $id;
            }
        )*
    }
}

pub trait Taggable {
    type Tagged;
    fn tag(self, tag: u16) -> Self::Tagged;
}

/// Allows messages to be declared in tagged and untagged form.
/// Will also impl `Taggable` for each untagged type.
#[macro_export]
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

            impl $crate::message::Taggable for $name {
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