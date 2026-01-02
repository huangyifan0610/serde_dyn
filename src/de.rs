//! # Dynamic Deserialization
//!
//! This module provides traits and implementations for dynamic deserialization, allowing
//! deserialization without knowing concrete types at compile time.

use core::{error, fmt, mem};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::{String, ToString};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// A data format that can dynamically deserialize the data structure supported by [`serde`].
///
/// This trait mirrors the functionality of [`serde::Deserializer`] but is dyn-compatible.
/// The mirrored functions always start with "dyn_", for example, `dyn_deserialize_any` in
/// `Deserializer` vs. `deserialize_any` in `serde::Deserializer`.
///
/// # Implementation
///
/// Alhrough its legal to implement this trait directly, it's more recommended to use the provided
/// function `<dyn Deserializer>::new` to automatically convert [`serde::Deserializer`] into dynamic
/// [`Deserializer`].
///
/// # Examples
///
/// The following example shows how to create dynamic deserializer, and use `&mut dyn Deserializer`
/// just like `serde::Deserializer`.
///
/// ```
/// # use serde_dyn::Deserializer;
/// let mut deserializer = serde_json::Deserializer::from_str("\"Hello, world!\"");
/// let mut deserializer = <dyn Deserializer>::new(&mut deserializer);
/// let deserializer: &mut dyn Deserializer = &mut deserializer;
///
/// let value = <String as serde::Deserialize>::deserialize(deserializer).unwrap();
/// assert_eq!(value, "Hello, world!");
/// ```    
#[diagnostic::on_unimplemented(
    note = "perfer `<dyn Deserializer>::new` instead of manually implementation"
)]
pub trait Deserializer<'de> {
    /// Require the `Deserializer` to figure out how to drive the visitor based
    /// on what data type is in the input.
    ///
    /// Also see [`serde::Deserializer::deserialize_any`].  
    fn dyn_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a `bool` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_bool`].  
    fn dyn_deserialize_bool(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `i8` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_i8`].  
    fn dyn_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `i16` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_i16`].  
    fn dyn_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `i32` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_i32`].  
    fn dyn_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `i64` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_i64`].  
    fn dyn_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `i128` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_i128`].  
    fn dyn_deserialize_128(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `u8` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_u8`].  
    fn dyn_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `u16` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_u16`].  
    fn dyn_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `u32` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_u32`].  
    fn dyn_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `u64` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_u64`].  
    fn dyn_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `u128` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_u128`].
    fn dyn_deserialize_u128(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `f32` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_f32`].
    fn dyn_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an `f64` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_f64`].
    fn dyn_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a `char` value.
    ///
    /// Also see [`serde::Deserializer::deserialize_char`].
    fn dyn_deserialize_char(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a string value and does
    /// not benefit from taking ownership of buffered data owned by the `Deserializer`.
    ///
    /// Also see [`serde::Deserializer::deserialize_str`].
    fn dyn_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a string value and would
    /// benefit from taking ownership of buffered data owned by the `Deserializer`.
    ///
    /// Also see [`serde::Deserializer::deserialize_string`].
    fn dyn_deserialize_string(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a byte array and does not
    /// benefit from taking ownership of buffered data owned by the `Deserializer`.
    ///
    /// Also see [`serde::Deserializer::deserialize_bytes`].
    fn dyn_deserialize_bytes(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a byte array and would
    /// benefit from taking ownership of buffered data owned by the `Deserializer`.
    ///
    /// Also see [`serde::Deserializer::deserialize_byte_buf`].
    fn dyn_deserialize_byte_buf(
        &mut self,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an optional value.
    ///
    /// Also see [`serde::Deserializer::deserialize_option`].
    fn dyn_deserialize_option(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a unit value.
    ///
    /// Also see [`serde::Deserializer::deserialize_unit`].
    fn dyn_deserialize_unit(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a unit struct with a
    /// particular name.
    ///
    /// Also see [`serde::Deserializer::deserialize_unit_struct`].
    fn dyn_deserialize_unit_struct(
        &mut self,
        name: &'static str,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a newtype struct with a
    /// particular name.
    ///
    /// Also see [`serde::Deserializer::deserialize_newtype_struct`].
    fn dyn_deserialize_newtype_struct(
        &mut self,
        name: &'static str,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a sequence of values.
    ///
    /// Also see [`serde::Deserializer::deserialize_seq`].
    fn dyn_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a sequence of values and
    /// knows how many values there are without looking at the serialized data.
    ///
    /// Also see [`serde::Deserializer::deserialize_tuple`].
    fn dyn_deserialize_tuple(
        &mut self,
        len: usize,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a tuple struct with a
    /// particular name and number of fields.
    ///
    /// Also see [`serde::Deserializer::deserialize_tuple_struct`].  
    fn dyn_deserialize_tuple_struct(
        &mut self,
        name: &'static str,
        len: usize,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a map of key-value pairs.
    ///
    /// Also see [`serde::Deserializer::deserialize_map`].  
    fn dyn_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting a struct with a particular
    /// name and fields.
    ///
    /// Also see [`serde::Deserializer::deserialize_struct`].  
    fn dyn_deserialize_struct(
        &mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting an enum value with a
    /// particular name and possible variants.
    ///
    /// Also see [`serde::Deserializer::deserialize_enum`].
    fn dyn_deserialize_enum(
        &mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type is expecting the name of a struct
    /// field or the discriminant of an enum variant.
    ///
    /// Also see [`serde::Deserializer::deserialize_identifier`].  
    fn dyn_deserialize_identifier(
        &mut self,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Hint that the `Deserialize` type needs to deserialize a value whose type
    /// doesn't matter because it is ignored.
    ///
    /// Also see [`serde::Deserializer::deserialize_ignored_any`].
    fn dyn_deserialize_ignored_any(
        &mut self,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Determine whether `Deserialize` implementations should expect to
    /// deserialize their human-readable form.
    ///
    /// Also see [`serde::Deserializer::is_human_readable`].
    fn dyn_is_human_readable(&self) -> bool;
}

impl<'de> dyn Deserializer<'de> {
    /// Returns dynamic [`Deserializer`].
    ///
    /// Note that the returned [`InplaceDeserializer`] is not an implementation of `Deserializer`.
    /// Use a mutable reference instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_dyn::Deserializer;
    /// let mut deserializer = serde_json::Deserializer::from_str("{\"x\":1,\"y\":2}");
    /// let mut deserializer = <dyn Deserializer>::new(&mut deserializer);
    /// let deserializer: &mut dyn Deserializer = &mut deserializer;
    /// # let _ = deserializer;
    /// ```
    #[inline]
    #[must_use]
    pub const fn new<D: serde::Deserializer<'de>>(deserializer: D) -> InplaceDeserializer<'de, D> {
        InplaceDeserializer::Deserializer(deserializer)
    }
}

impl<'de> serde::Deserializer<'de> for &mut (dyn Deserializer<'de> + '_) {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_any(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_bool(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_i8(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_i16(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_i32(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_i64(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_128(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_u8(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_u16(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_u32(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_u64(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_u128(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_f32(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_f64(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_char(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_str(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_string(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_bytes(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_byte_buf(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_option(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_unit(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_unit_struct(name, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_newtype_struct(name, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_seq(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_tuple(len, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_tuple_struct(name, len, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_map(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_struct(name, fields, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_enum(name, variants, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_identifier(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_deserialize_ignored_any(&mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.dyn_is_human_readable()
    }
}

/// The dyn-compatible version of the [`serde::de::DeserializeSeed`] trait.
///
/// We cannot perform type-erasure on [`serde::Deserialize`] since it's stateless. But luckly,
/// `DeserializeSeed` could receive our custom data - "seed". In the `dyn_deserialize` procedure,
/// the type-erased seed brings in the concrete seed, and then takes out the deserialized value.
///
/// Also see [`serde::de::DeserializeSeed`].
pub trait DeserializeSeed<'de> {
    /// Performs stateful deserialization with given dynamic deserializer.
    ///
    /// Also see [`serde::de::DeserializeSeed::deserialize`].
    fn dyn_deserialize(
        &mut self,
        deserializer: &mut dyn Deserializer<'de>,
    ) -> DeserializeResult<()>;
}

impl<'de> serde::de::DeserializeSeed<'de> for &mut (dyn DeserializeSeed<'de> + '_) {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut deserializer = InplaceDeserializer::Deserializer(deserializer);
        let result = self.dyn_deserialize(&mut deserializer);
        match deserializer {
            InplaceDeserializer::Error(error) => Err(error),
            _ => result.map_err(DeserializeError::into_de_error),
        }
    }
}

/// The dyn-compatible version of the [`serde::de::Visitor`] trait.
///
/// This trait represents a visitor that walks through a deserializer.
pub trait Visitor<'de> {
    /// Format a message stating what data this `Visitor` expects to receive.
    ///
    /// Also see [`serde::de::Visitor::expecting`].
    fn dyn_expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// The input contains a boolean.
    ///
    /// Also see [`serde::de::Visitor::visit_bool`].
    fn dyn_visit_bool(&mut self, v: bool) -> DeserializeResult<()>;

    /// The input contains an `i8`.
    ///
    /// Also see [`serde::de::Visitor::visit_i8`].
    fn dyn_visit_i8(&mut self, v: i8) -> DeserializeResult<()>;

    /// The input contains an `i16`.
    ///
    /// Also see [`serde::de::Visitor::visit_i16`].
    fn dyn_visit_i16(&mut self, v: i16) -> DeserializeResult<()>;

    /// The input contains an `i32`.
    ///
    /// Also see [`serde::de::Visitor::visit_i32`].
    fn dyn_visit_i32(&mut self, v: i32) -> DeserializeResult<()>;

    /// The input contains an `i64`.
    ///
    /// Also see [`serde::de::Visitor::visit_i64`].
    fn dyn_visit_i64(&mut self, v: i64) -> DeserializeResult<()>;

    /// The input contains an `i128`.
    ///
    /// Also see [`serde::de::Visitor::visit_i128`].
    fn dyn_visit_i128(&mut self, v: i128) -> DeserializeResult<()>;

    /// The input contains an `u8`.
    ///
    /// Also see [`serde::de::Visitor::visit_u8`].
    fn dyn_visit_u8(&mut self, v: u8) -> DeserializeResult<()>;

    /// The input contains an `u16`.
    ///
    /// Also see [`serde::de::Visitor::visit_u16`].
    fn dyn_visit_u16(&mut self, v: u16) -> DeserializeResult<()>;

    /// The input contains an `u32`.
    ///
    /// Also see [`serde::de::Visitor::visit_u32`].
    fn dyn_visit_u32(&mut self, v: u32) -> DeserializeResult<()>;

    /// The input contains an `u64`.
    ///
    /// Also see [`serde::de::Visitor::visit_u64`].
    fn dyn_visit_u64(&mut self, v: u64) -> DeserializeResult<()>;

    /// The input contains an `u128`.
    ///
    /// Also see [`serde::de::Visitor::visit_u128`].
    fn dyn_visit_u128(&mut self, v: u128) -> DeserializeResult<()>;

    /// The input contains an `f32`.
    ///
    /// Also see [`serde::de::Visitor::visit_f32`].
    fn dyn_visit_f32(&mut self, v: f32) -> DeserializeResult<()>;

    /// The input contains an `f64`.
    ///
    /// Also see [`serde::de::Visitor::visit_f64`].
    fn dyn_visit_f64(&mut self, v: f64) -> DeserializeResult<()>;

    /// The input contains a `char`.
    ///
    /// Also see [`serde::de::Visitor::visit_char`].
    fn dyn_visit_char(&mut self, v: char) -> DeserializeResult<()>;

    /// The input contains a string. The lifetime of the string is ephemeral and
    /// it may be destroyed after this method returns.
    ///
    /// Also see [`serde::de::Visitor::visit_str`].
    fn dyn_visit_str(&mut self, v: &str) -> DeserializeResult<()>;

    /// The input contains a string that lives at least as long as the
    /// `Deserializer`.
    ///
    /// Also see [`serde::de::Visitor::visit_borrowed_str`].
    fn dyn_visit_borrowed_str(&mut self, v: &'de str) -> DeserializeResult<()>;

    /// The input contains a string and ownership of the string is being given
    /// to the `Visitor`.
    ///
    /// Also see [`serde::de::Visitor::visit_string`].
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn dyn_visit_string(&mut self, v: String) -> DeserializeResult<()>;

    /// The input contains a byte array. The lifetime of the byte array is
    /// ephemeral and it may be destroyed after this method returns.
    ///
    /// Also see [`serde::de::Visitor::visit_bytes`].
    fn dyn_visit_bytes(&mut self, v: &[u8]) -> DeserializeResult<()>;

    /// The input contains a byte array that lives at least as long as the
    /// `Deserializer`.
    ///
    /// Also see [`serde::de::Visitor::visit_borrowed_bytes`].
    fn dyn_visit_borrowed_bytes(&mut self, v: &'de [u8]) -> DeserializeResult<()>;

    /// The input contains a byte array and ownership of the byte array is being
    /// given to the `Visitor`.
    ///
    /// Also see [`serde::de::Visitor::visit_byte_buf`].
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn dyn_visit_byte_buf(&mut self, v: Vec<u8>) -> DeserializeResult<()>;

    /// The input contains an optional that is absent.
    ///
    /// Also see [`serde::de::Visitor::visit_none`].
    fn dyn_visit_none(&mut self) -> DeserializeResult<()>;

    /// The input contains an optional that is present.
    ///
    /// Also see [`serde::de::Visitor::visit_some`].
    fn dyn_visit_some(&mut self, deserializer: &mut dyn Deserializer<'de>)
    -> DeserializeResult<()>;

    /// The input contains a unit `()`.
    ///
    /// Also see [`serde::de::Visitor::visit_unit`].
    fn dyn_visit_unit(&mut self) -> DeserializeResult<()>;

    /// The input contains a newtype struct.
    ///
    /// The content of the newtype struct may be read from the given
    /// `Deserializer`.
    ///
    /// Also see [`serde::de::Visitor::visit_newtype_struct`].
    fn dyn_visit_newtype_struct(
        &mut self,
        deserializer: &mut dyn Deserializer<'de>,
    ) -> DeserializeResult<()>;

    /// The input contains a sequence of elements.
    ///
    /// Also see [`serde::de::Visitor::visit_seq`].
    fn dyn_visit_seq(&mut self, seq: &mut dyn SeqAccess<'de>) -> DeserializeResult<()>;

    /// The input contains a key-value map.
    ///
    /// Also see [`serde::de::Visitor::visit_map`].
    fn dyn_visit_map(&mut self, map: &mut dyn MapAccess<'de>) -> DeserializeResult<()>;

    /// The input contains an enum.
    ///
    /// Also see [`serde::de::Visitor::visit_enum`].
    fn dyn_visit_enum(&mut self, data: &mut dyn EnumAccess<'de>) -> DeserializeResult<()>;
}

impl<'de> serde::de::Visitor<'de> for &mut (dyn Visitor<'de> + '_) {
    type Value = ();

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dyn_expecting(formatter)
    }

    fn visit_bool<E>(self, v: bool) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_bool(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_i8<E>(self, v: i8) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_i8(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_i16<E>(self, v: i16) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_i16(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_i32<E>(self, v: i32) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_i32(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_i64<E>(self, v: i64) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_i64(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_i128<E>(self, v: i128) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_i128(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_u8<E>(self, v: u8) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_u8(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_u16<E>(self, v: u16) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_u16(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_u32<E>(self, v: u32) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_u32(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_u64<E>(self, v: u64) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_u64(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_u128<E>(self, v: u128) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_u128(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_f32<E>(self, v: f32) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_f32(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_f64<E>(self, v: f64) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_f64(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_char<E>(self, v: char) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_char(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_str<E>(self, v: &str) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_str(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_borrowed_str(v)
            .map_err(DeserializeError::into_de_error)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, v: String) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_string(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_bytes(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_borrowed_bytes(v)
            .map_err(DeserializeError::into_de_error)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_byte_buf(v)
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_none<E>(self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_none()
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut deserializer = InplaceDeserializer::Deserializer(deserializer);
        let result = self.dyn_visit_some(&mut deserializer);
        if let InplaceDeserializer::Error(error) = deserializer {
            Err(error)
        } else {
            result.map_err(DeserializeError::into_de_error)
        }
    }

    fn visit_unit<E>(self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.dyn_visit_unit()
            .map_err(DeserializeError::into_de_error)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut deserializer = InplaceDeserializer::Deserializer(deserializer);
        let result = self.dyn_visit_newtype_struct(&mut deserializer);
        if let InplaceDeserializer::Error(error) = deserializer {
            Err(error)
        } else {
            result.map_err(DeserializeError::into_de_error)
        }
    }

    fn visit_seq<A>(self, seq: A) -> Result<(), A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut seq = InplaceSeqAccess::SeqAccess(seq);
        let result = self.dyn_visit_seq(&mut seq);
        if let InplaceSeqAccess::Error(error) = seq {
            Err(error)
        } else {
            result.map_err(DeserializeError::into_de_error)
        }
    }

    fn visit_map<A>(self, map: A) -> Result<(), A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut map = InplaceMapAccess::MapAccess(map);
        let result = self.dyn_visit_map(&mut map);
        if let InplaceMapAccess::Error(error) = map {
            Err(error)
        } else {
            result.map_err(DeserializeError::into_de_error)
        }
    }

    fn visit_enum<A>(self, data: A) -> Result<(), A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let mut data = InplaceEnumAccess::EnumAccess(data);
        let result = self.dyn_visit_enum(&mut data);
        if let InplaceEnumAccess::Error(error) = data {
            Err(error)
        } else {
            result.map_err(DeserializeError::into_de_error)
        }
    }
}

/// The dyn-compatible version of trait [`serde::de::SeqAccess`].
///
/// Provides a `Visitor` access to each element of a sequence in the input.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation,
/// which deserializes each item in a sequence.
pub trait SeqAccess<'de> {
    /// This returns `Ok(Some(_))` for the next value in the sequence, or
    /// `Ok(None)` if there are no more remaining items.
    ///
    /// Also see [`serde::de::SeqAccess::next_element_seed`].
    fn dyn_next_element(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<Option<()>>;

    /// Returns the number of elements remaining in the sequence, if known.
    fn dyn_size_hint(&self) -> Option<usize>;
}

impl<'de> serde::de::SeqAccess<'de> for &mut (dyn SeqAccess<'de> + '_) {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> DeserializeResult<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let mut seed = InplaceDeserializeSeed::DeserializeSeed(seed);
        let result = self.dyn_next_element(&mut seed);
        if let InplaceDeserializeSeed::Value(value) = seed {
            debug_assert!(matches!(result, Ok(Some(()))));
            Ok(Some(value))
        } else if let Ok(None) = result {
            Ok(None)
        } else {
            Err(seed.expect_err(result))
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.dyn_size_hint()
    }
}

/// The dyn-compatible version of trait [`serde::de::MapAccess`].
///
/// Provides a `Visitor` access to each entry of a map in the input.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation.
pub trait MapAccess<'de> {
    /// This returns `Ok(Some(_))` for the next key in the map, or `Ok(None)`
    /// if there are no more remaining entries.
    ///
    /// Also see [`serde::de::MapAccess::next_key_seed`].
    fn dyn_next_key(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<Option<()>>;

    /// This returns a `Ok(_)` for the next value in the map.
    ///
    /// Also see [`serde::de::MapAccess::next_value_seed`].
    fn dyn_next_value(&mut self, seed: &mut dyn DeserializeSeed<'de>) -> DeserializerResult<()>;

    /// This returns `Ok(Some((_, _)))` for the next key-value pair in
    /// the map, or `Ok(None)` if there are no more remaining items.
    ///
    /// Also see [`serde::de::MapAccess::next_entry_seed`].
    fn dyn_next_entry(
        &mut self,
        kseed: &mut dyn DeserializeSeed<'de>,
        vseed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<Option<((), ())>>;

    /// Returns the number of entries remaining in the map, if known.
    ///
    /// Also see [`serde::de::MapAccess::size_hint`].
    fn dyn_size_hint(&self) -> Option<usize>;
}

impl<'de> serde::de::MapAccess<'de> for &mut (dyn MapAccess<'de> + '_) {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> DeserializeResult<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let mut seed = InplaceDeserializeSeed::DeserializeSeed(seed);
        let result = self.dyn_next_key(&mut seed);
        if let InplaceDeserializeSeed::Value(key) = seed {
            debug_assert!(matches!(result, Ok(Some(()))));
            Ok(Some(key))
        } else if let Ok(None) = result {
            Ok(None)
        } else {
            Err(seed.expect_err(result))
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let mut seed = InplaceDeserializeSeed::DeserializeSeed(seed);
        let result = self.dyn_next_value(&mut seed);
        match seed {
            InplaceDeserializeSeed::Value(value) => Ok(value),
            seed => Err(seed.expect_err(result)),
        }
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> DeserializeResult<Option<(K::Value, V::Value)>>
    where
        K: serde::de::DeserializeSeed<'de>,
        V: serde::de::DeserializeSeed<'de>,
    {
        let mut kseed = InplaceDeserializeSeed::DeserializeSeed(kseed);
        let mut vseed = InplaceDeserializeSeed::DeserializeSeed(vseed);
        let result = self.dyn_next_entry(&mut kseed, &mut vseed);
        if let InplaceDeserializeSeed::Value(key) = kseed
            && let InplaceDeserializeSeed::Value(value) = vseed
        {
            debug_assert!(matches!(result, Ok(Some(((), ())))));
            Ok(Some((key, value)))
        } else if let Ok(None) = result {
            Ok(None)
        } else {
            Err(vseed.expect_err(result))
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.dyn_size_hint()
    }
}

/// The dyn-compatible version of trait [`serde::de::EnumAccess`].
///
/// Provides a `Visitor` access to the data of an enum in the input.
///
/// `EnumAccess` is created by the `Deserializer` and passed to the
/// `Visitor` in order to identify which variant of an enum to deserialize.
pub trait EnumAccess<'de> {
    /// `variant` is called to identify which variant to deserialize.
    ///
    /// Also see [`serde::de::EnumAccess::variant_seed`].
    fn dyn_variant_seed(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<((), &mut dyn VariantAccess<'de>)>;
}

impl<'a, 'de: 'a> serde::de::EnumAccess<'de> for &'a mut (dyn EnumAccess<'de> + '_) {
    type Error = DeserializeError;
    type Variant = &'a mut dyn VariantAccess<'de>;

    fn variant_seed<V>(self, seed: V) -> DeserializeResult<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let mut seed = InplaceDeserializeSeed::DeserializeSeed(seed);
        let result = self.dyn_variant_seed(&mut seed);
        match result {
            Ok(((), variant)) => match seed {
                InplaceDeserializeSeed::Value(value) => Ok((value, variant)),
                // This is unreachable because `result` is `Ok(_)` if and only if `seed` is `Value`.
                _ => unreachable!(),
            },
            Err(error) => Err(DeserializeError::from(error)),
        }
    }
}

/// The dyn-compatible version of trait [`serde::de::VariantAccess`].
///
/// `VariantAccess` is a visitor that is created by the `Deserializer` and passed
/// to the `Deserialize` to deserialize the content of a particular enum variant.
pub trait VariantAccess<'de> {
    /// Called when deserializing a variant with no values.
    ///
    /// Also see [`serde::de::VariantAccess::unit_variant`].
    fn dyn_unit_variant(&mut self) -> DeserializerResult<()>;

    /// Called when deserializing a variant with a single value.
    ///
    /// Also see [`serde::de::VariantAccess::newtype_variant_seed`].
    fn dyn_newtype_variant(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<()>;

    /// Called when deserializing a tuple-like variant.
    ///
    /// Also see [`serde::de::VariantAccess::tuple_variant`].
    fn dyn_tuple_variant(
        &mut self,
        len: usize,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;

    /// Called when deserializing a struct-like variant.
    ///
    /// The `fields` are the names of the fields of the struct variant.
    ///
    /// Also see [`serde::de::VariantAccess::struct_variant`].
    fn dyn_struct_variant(
        &mut self,
        fields: &'static [&'static str],
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()>;
}

impl<'de> serde::de::VariantAccess<'de> for &mut (dyn VariantAccess<'de> + '_) {
    type Error = DeserializeError;

    fn unit_variant(self) -> DeserializeResult<()> {
        self.dyn_unit_variant().map_err(DeserializeError::from)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> DeserializeResult<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let mut seed = InplaceDeserializeSeed::DeserializeSeed(seed);
        let result = self.dyn_newtype_variant(&mut seed);
        match seed {
            InplaceDeserializeSeed::Value(value) => Ok(value),
            seed => Err(seed.expect_err(result)),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_tuple_variant(len, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> DeserializeResult<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut visitor = InplaceVisitor::Visitor(visitor);
        let result = self.dyn_struct_variant(fields, &mut visitor);
        match visitor {
            InplaceVisitor::Value(value) => Ok(value),
            visitor => Err(visitor.expect_err(result)),
        }
    }
}

/// [`Result`] type alias with error type [`DeserializeError`].
pub type DeserializeResult<T> = Result<T, DeserializeError>;

/// A universal implementation of the [`serde::de::Error`] trait.
///
/// This error tells why the dynamic deserialization failed and is returned by
/// [`&mut dyn Deserializer`](crate::de::Deserializer) as [`serde::Deserializer`].
pub enum DeserializeError {
    /// The deserializer is in the wrong status.
    DeserializerError(DeserializerError),

    /// A custom error that is returned by [`serde::de::Error::custom`].
    #[cfg(any(feature = "std", feature = "alloc"))]
    Other(Box<str>),
}

impl DeserializeError {
    /// Converts the error into an arbitrary [`serde::de::Error`].
    #[cold]
    #[must_use]
    pub fn into_de_error<E: serde::de::Error>(self) -> E {
        match self {
            DeserializeError::DeserializerError(error) => E::custom(error),
            #[cfg(any(feature = "std", feature = "alloc"))]
            DeserializeError::Other(error) => E::custom(error),
        }
    }
}

impl From<DeserializerError> for DeserializeError {
    #[cold]
    #[inline(never)]
    fn from(error: DeserializerError) -> Self {
        DeserializeError::DeserializerError(error)
    }
}

impl fmt::Debug for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeError::DeserializerError(error) => error.fmt(f),
            #[cfg(any(feature = "std", feature = "alloc"))]
            DeserializeError::Other(error) => f.write_str(error),
        }
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeError::DeserializerError(error) => error.fmt(f),
            #[cfg(any(feature = "std", feature = "alloc"))]
            DeserializeError::Other(error) => f.write_str(error),
        }
    }
}

impl error::Error for DeserializeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DeserializeError::DeserializerError(error) => Some(error),
            #[cfg(any(feature = "std", feature = "alloc"))]
            DeserializeError::Other(_) => None,
        }
    }
}

impl serde::de::Error for DeserializeError {
    #[cfg(any(feature = "std", feature = "alloc"))]
    #[cold]
    #[inline(never)]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializeError::Other(msg.to_string().into_boxed_str())
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn custom<T: fmt::Display>(_: T) -> Self {
        DeserializeError::DeserializerError(DeserializerError::Error)
    }
}

/// [`Result`] type alias with error type [`DeserializerError`].
pub type DeserializerResult<T> = Result<T, DeserializerError>;

/// An error that is returned by methods defined in [`Deserializer`].
///
/// This error just represents that the `Deserializer` is in the wrong status. It doesn't implement
/// [`serde::de::Error`] and tells nothing about why the deserialization fails.
///
/// [`Deserializer`]: crate::de::Deserializer
#[derive(Clone, Copy, Debug, Default)]
pub enum DeserializerError {
    /// An error occurred during the deserialization.
    #[default]
    Error,
    /// The deserializer is not ready.
    Deserializer,
    /// The deserialize-seed is not ready.
    DeserializeSeed,
    /// The visitor is not ready.
    Visitor,
    /// The sequence access is not ready.
    SeqAccess,
    /// The map access is not ready.
    MapAccess,
    /// The enum access is not ready.
    EnumAccess,
    /// The enum variant access is not ready.
    VariantAccess,
}

impl fmt::Display for DeserializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DeserializerError::Error => "an error occurred during the deserialization",
            DeserializerError::Deserializer => "the deserializer is not ready",
            DeserializerError::DeserializeSeed => "the deserialize-seed is not ready",
            DeserializerError::Visitor => "the visitor is not ready",
            DeserializerError::SeqAccess => "the sequence access is not ready",
            DeserializerError::MapAccess => "the map access is not ready",
            DeserializerError::EnumAccess => "the enum access is not ready",
            DeserializerError::VariantAccess => "the enum variant access is not ready",
        })
    }
}

impl error::Error for DeserializerError {}

/// An implementation of the [`Deserializer`] trait.
///
/// Also see [`serde::de::Deserializer`].
#[derive(Debug, Default)]
pub enum InplaceDeserializer<'de, D: serde::Deserializer<'de>> {
    /// The deserializer is not ready.
    #[default]
    None,
    /// The deserialization has done unsuccessfully.
    Error(D::Error),
    /// The deserializer is ready.
    Deserializer(D),
}

impl<'de, D: serde::Deserializer<'de>> InplaceDeserializer<'de, D> {
    fn write_error(&mut self, error: D::Error) -> DeserializerError {
        *self = InplaceDeserializer::Error(error);
        DeserializerError::Error
    }
}

impl<'de, D: serde::Deserializer<'de>> Deserializer<'de> for InplaceDeserializer<'de, D> {
    fn dyn_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_any(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_bool(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_bool(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_i8(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_i16(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_i32(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_i64(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_128(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_i128(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_u8(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_u16(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_u32(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_u64(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_u128(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_u128(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_f32(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_f64(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_char(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_char(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_str(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_string(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_string(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_bytes(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_bytes(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_byte_buf(
        &mut self,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_byte_buf(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_option(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_option(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_unit(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_unit(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_unit_struct(
        &mut self,
        name: &'static str,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_unit_struct(name, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_newtype_struct(
        &mut self,
        name: &'static str,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_newtype_struct(name, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_seq(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_tuple(
        &mut self,
        len: usize,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_tuple(len, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_tuple_struct(
        &mut self,
        name: &'static str,
        len: usize,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_tuple_struct(name, len, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_map(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_struct(
        &mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_struct(name, fields, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_enum(
        &mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_enum(name, variants, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_identifier(
        &mut self,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_identifier(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_deserialize_ignored_any(
        &mut self,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        if let InplaceDeserializer::Deserializer(de) = mem::take(self) {
            de.deserialize_ignored_any(visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::Deserializer)
        }
    }

    fn dyn_is_human_readable(&self) -> bool {
        if let InplaceDeserializer::Deserializer(de) = self {
            de.is_human_readable()
        } else {
            true
        }
    }
}

/// An implementation of the [`DeserializeSeed`] trait.
///
/// Also see [`serde::de::DeserializeSeed`].
#[derive(Debug, Default)]
pub enum InplaceDeserializeSeed<'de, T: serde::de::DeserializeSeed<'de>> {
    /// The deserialize-seed is not ready.
    #[default]
    None,
    /// The deserialization has done successfully.
    Value(T::Value),
    /// The deserialize-seed is ready.
    DeserializeSeed(T),
}

impl<'de, T: serde::de::DeserializeSeed<'de>> InplaceDeserializeSeed<'de, T> {
    fn expect_err<U>(&self, result: DeserializerResult<U>) -> DeserializeError {
        debug_assert!(matches!(
            self,
            InplaceDeserializeSeed::None | InplaceDeserializeSeed::DeserializeSeed(_)
        ));

        match result {
            Ok(_) => unreachable!(),
            Err(error) => DeserializeError::DeserializerError(error),
        }
    }
}

impl<'de, T: serde::de::DeserializeSeed<'de>> DeserializeSeed<'de>
    for InplaceDeserializeSeed<'de, T>
{
    fn dyn_deserialize(
        &mut self,
        deserializer: &mut dyn Deserializer<'de>,
    ) -> DeserializeResult<()> {
        if let InplaceDeserializeSeed::DeserializeSeed(seed) = mem::take(self) {
            seed.deserialize(deserializer)
                .map(|value| *self = InplaceDeserializeSeed::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::DeserializeSeed))
        }
    }
}

/// An implementation of the [`Visitor`] trait.
///
/// Also see [`serde::de::Visitor`].
#[derive(Debug, Default)]
pub enum InplaceVisitor<'de, V: serde::de::Visitor<'de>> {
    /// The deserialize-seed is not ready.
    #[default]
    None,
    /// The deserialization has done successfully.
    Value(V::Value),
    /// The deserialize-seed is ready.
    Visitor(V),
}

impl<'de, V: serde::de::Visitor<'de>> InplaceVisitor<'de, V> {
    fn expect_err(&self, result: DeserializerResult<()>) -> DeserializeError {
        debug_assert!(matches!(
            self,
            InplaceVisitor::None | InplaceVisitor::Visitor(_)
        ));

        match result {
            Ok(_) => unreachable!(),
            Err(error) => DeserializeError::DeserializerError(error),
        }
    }
}

impl<'de, V: serde::de::Visitor<'de>> Visitor<'de> for InplaceVisitor<'de, V> {
    fn dyn_expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let InplaceVisitor::Visitor(visitor) = self {
            visitor.expecting(formatter)
        } else {
            formatter.write_str("nothing (the visitor is not ready)")
        }
    }

    fn dyn_visit_bool(&mut self, v: bool) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_bool(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_i8(&mut self, v: i8) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_i8(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_i16(&mut self, v: i16) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_i16(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_i32(&mut self, v: i32) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_i32(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_i64(&mut self, v: i64) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_i64(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_i128(&mut self, v: i128) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_i128(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_u8(&mut self, v: u8) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_u8(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_u16(&mut self, v: u16) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_u16(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_u32(&mut self, v: u32) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_u32(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_u64(&mut self, v: u64) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_u64(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_u128(&mut self, v: u128) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_u128(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_f32(&mut self, v: f32) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_f32(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_f64(&mut self, v: f64) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_f64(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_char(&mut self, v: char) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_char(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_str(&mut self, v: &str) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_str(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_borrowed_str(&mut self, v: &'de str) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_borrowed_str(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn dyn_visit_string(&mut self, v: String) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_string(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_bytes(&mut self, v: &[u8]) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_bytes(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_borrowed_bytes(&mut self, v: &'de [u8]) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_borrowed_bytes(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn dyn_visit_byte_buf(&mut self, v: Vec<u8>) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_byte_buf(v)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_none(&mut self) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_none()
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_some(
        &mut self,
        deserializer: &mut dyn Deserializer<'de>,
    ) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_some(deserializer)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_unit(&mut self) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_unit()
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_newtype_struct(
        &mut self,
        deserializer: &mut dyn Deserializer<'de>,
    ) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_newtype_struct(deserializer)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_seq(&mut self, seq: &mut dyn SeqAccess<'de>) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_seq(seq)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_map(&mut self, map: &mut dyn MapAccess<'de>) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_map(map)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }

    fn dyn_visit_enum(&mut self, data: &mut dyn EnumAccess<'de>) -> DeserializeResult<()> {
        if let InplaceVisitor::Visitor(visitor) = mem::take(self) {
            visitor
                .visit_enum(data)
                .map(|value| *self = InplaceVisitor::Value(value))
        } else {
            Err(DeserializeError::from(DeserializerError::Visitor))
        }
    }
}

/// An implementation of the [`SeqAccess`] trait.
///
/// Also see [`serde::de::SeqAccess`].
#[derive(Debug)]
pub enum InplaceSeqAccess<'de, A: serde::de::SeqAccess<'de>> {
    /// The deserialization has done unsuccessfully.
    Error(A::Error),
    /// The sequence access is ready.
    SeqAccess(A),
}

impl<'de, A: serde::de::SeqAccess<'de>> InplaceSeqAccess<'de, A> {
    fn write_error(&mut self, error: A::Error) -> DeserializerError {
        *self = InplaceSeqAccess::Error(error);
        DeserializerError::Error
    }
}

impl<'de, A: serde::de::SeqAccess<'de>> SeqAccess<'de> for InplaceSeqAccess<'de, A> {
    fn dyn_next_element(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<Option<()>> {
        if let InplaceSeqAccess::SeqAccess(access) = self {
            access
                .next_element_seed(seed)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::SeqAccess)
        }
    }

    fn dyn_size_hint(&self) -> Option<usize> {
        match self {
            InplaceSeqAccess::Error(_) => None,
            InplaceSeqAccess::SeqAccess(access) => access.size_hint(),
        }
    }
}

/// An implementation of the [`MapAccess`] trait.
///
/// Also see [`serde::de::MapAccess`].
#[derive(Debug)]
pub enum InplaceMapAccess<'de, A: serde::de::MapAccess<'de>> {
    /// The deserialization has done unsuccessfully.
    Error(A::Error),
    /// The map access is ready.
    MapAccess(A),
}

impl<'de, A: serde::de::MapAccess<'de>> InplaceMapAccess<'de, A> {
    fn write_error(&mut self, error: A::Error) -> DeserializerError {
        *self = InplaceMapAccess::Error(error);
        DeserializerError::Error
    }
}

impl<'de, A: serde::de::MapAccess<'de>> MapAccess<'de> for InplaceMapAccess<'de, A> {
    fn dyn_next_key(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<Option<()>> {
        if let InplaceMapAccess::MapAccess(access) = self {
            access
                .next_key_seed(seed)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::MapAccess)
        }
    }

    fn dyn_next_value(&mut self, seed: &mut dyn DeserializeSeed<'de>) -> DeserializerResult<()> {
        if let InplaceMapAccess::MapAccess(access) = self {
            access
                .next_value_seed(seed)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::MapAccess)
        }
    }

    fn dyn_next_entry(
        &mut self,
        kseed: &mut dyn DeserializeSeed<'de>,
        vseed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<Option<((), ())>> {
        if let InplaceMapAccess::MapAccess(access) = self {
            access
                .next_entry_seed(kseed, vseed)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::MapAccess)
        }
    }

    fn dyn_size_hint(&self) -> Option<usize> {
        match self {
            InplaceMapAccess::Error(_) => None,
            InplaceMapAccess::MapAccess(access) => access.size_hint(),
        }
    }
}

/// An implementation of the [`EnumAccess`] and [`VariantAccess`] traits.
///
/// Also see [`serde::de::EnumAccess`] and [`serde::de::VariantAccess`].
#[derive(Debug, Default)]
pub enum InplaceEnumAccess<'de, A: serde::de::EnumAccess<'de>> {
    /// The enum access is not ready.
    #[default]
    None,
    /// The deserialization has done unsuccessfully.
    Error(A::Error),
    /// The enum access is ready.
    EnumAccess(A),
    /// The enum variant access is ready.
    VariantAccess(A::Variant),
}

impl<'de, A: serde::de::EnumAccess<'de>> InplaceEnumAccess<'de, A> {
    fn write_error(&mut self, error: A::Error) -> DeserializerError {
        *self = InplaceEnumAccess::Error(error);
        DeserializerError::Error
    }
}

impl<'de, A: serde::de::EnumAccess<'de>> EnumAccess<'de> for InplaceEnumAccess<'de, A> {
    fn dyn_variant_seed(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<((), &mut dyn VariantAccess<'de>)> {
        if let InplaceEnumAccess::EnumAccess(access) = mem::take(self) {
            match access.variant_seed(seed) {
                Ok(((), variant)) => {
                    *self = InplaceEnumAccess::VariantAccess(variant);
                    Ok(((), self))
                }
                Err(error) => Err(self.write_error(error)),
            }
        } else {
            Err(DeserializerError::EnumAccess)
        }
    }
}

impl<'de, A: serde::de::EnumAccess<'de>> VariantAccess<'de> for InplaceEnumAccess<'de, A> {
    fn dyn_unit_variant(&mut self) -> DeserializerResult<()> {
        use serde::de::VariantAccess;

        if let InplaceEnumAccess::VariantAccess(access) = mem::take(self) {
            access
                .unit_variant()
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::VariantAccess)
        }
    }

    fn dyn_newtype_variant(
        &mut self,
        seed: &mut dyn DeserializeSeed<'de>,
    ) -> DeserializerResult<()> {
        use serde::de::VariantAccess;

        if let InplaceEnumAccess::VariantAccess(access) = mem::take(self) {
            access
                .newtype_variant_seed(seed)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::VariantAccess)
        }
    }

    fn dyn_tuple_variant(
        &mut self,
        len: usize,
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        use serde::de::VariantAccess;

        if let InplaceEnumAccess::VariantAccess(access) = mem::take(self) {
            access
                .tuple_variant(len, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::VariantAccess)
        }
    }

    fn dyn_struct_variant(
        &mut self,
        fields: &'static [&'static str],
        visitor: &mut dyn Visitor<'de>,
    ) -> DeserializerResult<()> {
        use serde::de::VariantAccess;

        if let InplaceEnumAccess::VariantAccess(access) = mem::take(self) {
            access
                .struct_variant(fields, visitor)
                .map_err(|error| self.write_error(error))
        } else {
            Err(DeserializerError::VariantAccess)
        }
    }
}
