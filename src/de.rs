//! Deserializers and deserializer convenience functions.

pub use serde::de::Deserialize;

mod common;
pub use common::*;

mod read;
pub use read::*;