//! Dynamic serialization and deserialization based on [`serde`].
//!
//! This crate provides traits and implementations that allow for dynamic serialization and
//! deserialization through trait objects, enabling serialization and deserialization without
//! knowing the concrete types at compile time.
//!
//! The main components include [`Serialize`], [`Serializer`] and [`Deserializer`].

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod de;
pub mod error;
pub mod ser;

// re-exports
pub use crate::de::Deserializer;
pub use crate::ser::{Serialize, Serializer};
