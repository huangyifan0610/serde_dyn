//! # Dynamic Serialization.
//!
//! This module provides traits and implementations for dynamic serialization, allowing
//! serialization of trait objects without knowing the concrete type at compile time.
//!
//! - [`Serialize`]: the dyn-compatible version of [`serde::Serialize`].
//! - [`Serializer`]: the dyn-compatible version of [`serde::Serializer`].

use core::{error, fmt, mem};

#[cfg(any(feature = "std", feature = "alloc"))]
use core::num::NonZeroUsize;
#[cfg(any(feature = "std", feature = "alloc"))]
use core::ptr::NonNull;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::{String, ToString};

/// A data structure that can be serialized with dynamic [`Serializer`].
///
/// This trait mirrors the functionality of [`serde::Serialize`] but is dyn-compatible.
///
/// The trait objects `dyn Serialize` also implement [`serde::Serialize`], which means they can
/// be serialized just like common data structures.
///
/// # Implementation
///
/// Always perfer to implement [`serde::Serialize`] rather than [`Serialize`], because data types
/// that implement [`serde::Serialize`] automatically derive this trait thanks to the blanket
/// implementation.
///
/// # Examples
///
/// The following example shows how to serialize a heterogeneous array:
///
/// ```
/// # use serde_dyn::Serialize;
/// let heterogeneous = [
///     &[true, false] as &dyn Serialize,
///     &100_u8 as &dyn Serialize,
///     &"Hello, world" as &dyn Serialize,
///     &3.14_f32 as &dyn Serialize,
/// ];
///
/// let json = serde_json::to_string(&heterogeneous).unwrap();
/// assert_eq!(json, r#"[[true,false],100,"Hello, world",3.14]"#);
/// ```
#[diagnostic::on_unimplemented(
    message = "the trait bound `{Self}: serde::Serialize` is not satisfied",
    note = "the trait `serde_dyn::Serialize` is not implement for `{Self}`",
    note = "because the trait `serde::Serialize` is not implemented for `{Self}`"
)]
pub trait Serialize {
    /// Serialize this value into the given [`Serializer`].
    ///
    /// Also see [`serde::Serialize::serialize`].
    fn dyn_serialize(&self, serializer: &mut dyn Serializer) -> SerializeResult<()>;
}

/// Automatically derives [`Serialize`] for `T` which implements [`serde::Serialize`].
impl<T: ?Sized + serde::Serialize> Serialize for T {
    fn dyn_serialize(&self, serializer: &mut dyn Serializer) -> SerializeResult<()> {
        // Just forward to `serde::Serialize::serialize` because `&mut dyn Serializer` also
        // implements `serde::Serializer`.
        self.serialize(serializer)
    }
}

impl serde::Serialize for dyn Serialize + '_ {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // We can serialize `dyn Serialize` with `dyn Serializer`.
        let mut serializer = InplaceSerializer::Serializer(serializer);
        let result = self.dyn_serialize(&mut serializer);
        match serializer {
            // `InplaceSerializer::Ok(_)` implies `result` is `Ok(_)`.
            InplaceSerializer::Ok(ok) => Ok(ok),
            // `InplaceSerializer::Err(_)` implies `result` is `Err(SerializerError::Error)`.
            InplaceSerializer::Error(error) => Err(error),
            _ => match result {
                Ok(_) => unreachable!("`result` is `Ok(_)` if and only if `serializer` is `Ok(_)`"),
                Err(error) => Err(error.into_ser_error()),
            },
        }
    }
}

/// A data format that can dynamically serialize the data structure supported by [`serde`].
///
/// This trait mirrors the functionality of [`serde::Serializer`] but is dyn-compatible. The
/// mirrored functions always start with "dyn_", for example, `dyn_serialize_bool` in `Serializer`
/// vs. `serialize_bool` in `serde::Serializer`. But these functions are not supported:
///
/// - [`serde::Serializer::collect_seq`].
/// - [`serde::Serializer::collect_map`].
///
/// # Implementation
///
/// Alhrough its legal to implement this trait directly, it's more recommended to use the provided
/// function `<dyn Serializer>::new` to automatically convert [`serde::Serializer`] into dynamic
/// [`Serializer`].
///
/// # Examples
///
/// The following example shows how to create dynamic serializer.
///
/// ```
/// # use std::collections::HashMap;
/// # use serde_dyn::Serializer;
/// let mut json = Vec::new();
/// let writer = std::io::Cursor::new(&mut json);
/// let mut json_serializer = serde_json::Serializer::new(writer);
/// let mut json_serializer = <dyn Serializer>::new(&mut json_serializer);
///
/// let mut cbor = Vec::new();
/// let writer = std::io::Cursor::new(&mut cbor);
/// let writer = serde_cbor::ser::IoWrite::new(writer);
/// let mut cbor_serializer = serde_cbor::Serializer::new(writer);
/// let mut cbor_serializer = <dyn Serializer>::new(&mut cbor_serializer);
///
/// let mut map: HashMap<&'static str, &mut dyn Serializer> = HashMap::with_capacity(2);
/// map.insert("json", &mut json_serializer);
/// map.insert("cbor", &mut cbor_serializer);
/// for serializer in map.values_mut() {
///     serializer.dyn_serialize_str("Hello, world!");
/// }
/// assert_eq!(json, b"\"Hello, world!\"");
/// assert_eq!(cbor, b"mHello, world!");
/// ```
///
/// It's also posiblie to treat `dyn Serializer` as common [`serde::Serializer`].
///
/// ```
/// # use serde_dyn::Serializer;
/// let mut json = Vec::new();
/// let writer = std::io::Cursor::new(&mut json);
/// let mut serializer = serde_json::Serializer::new(writer);
/// let mut serializer = <dyn Serializer>::new(&mut serializer);
/// let serializer: &mut dyn Serializer = &mut serializer;
///
/// let data = ["Hello", "World"];
/// serde::Serialize::serialize(&data, serializer).unwrap();
/// assert_eq!(json, b"[\"Hello\",\"World\"]");
/// ```
#[diagnostic::on_unimplemented(
    note = "perfer `<dyn Serializer>::new` instead of manually implementation"
)]
pub trait Serializer {
    /// Serialize a `bool` value.
    ///
    /// Also see [`serde::Serializer::serialize_bool`].
    fn dyn_serialize_bool(&mut self, v: bool) -> SerializerResult<()>;

    /// Serialize an `i8` value.
    ///
    /// Also see [`serde::Serializer::serialize_i8`].
    fn dyn_serialize_i8(&mut self, v: i8) -> SerializerResult<()>;

    /// Serialize an `i16` value.
    ///
    /// Also see [`serde::Serializer::serialize_i16`].
    fn dyn_serialize_i16(&mut self, v: i16) -> SerializerResult<()>;

    /// Serialize an `i32` value.
    ///
    /// Also see [`serde::Serializer::serialize_i32`].
    fn dyn_serialize_i32(&mut self, v: i32) -> SerializerResult<()>;

    /// Serialize an `i64` value.
    ///
    /// Also see [`serde::Serializer::serialize_i64`].
    fn dyn_serialize_i64(&mut self, v: i64) -> SerializerResult<()>;

    /// Serialize an `i128` value.
    ///
    /// Also see [`serde::Serializer::serialize_i128`].
    fn dyn_serialize_i128(&mut self, v: i128) -> SerializerResult<()>;

    /// Serialize an `u8` value.
    ///
    /// Also see [`serde::Serializer::serialize_u8`].
    fn dyn_serialize_u8(&mut self, v: u8) -> SerializerResult<()>;

    /// Serialize an `u16` value.
    ///
    /// Also see [`serde::Serializer::serialize_u16`].
    fn dyn_serialize_u16(&mut self, v: u16) -> SerializerResult<()>;

    /// Serialize an `u32` value.
    ///
    /// Also see [`serde::Serializer::serialize_u32`].
    fn dyn_serialize_u32(&mut self, v: u32) -> SerializerResult<()>;

    /// Serialize an `u64` value.
    ///
    /// Also see [`serde::Serializer::serialize_u64`].
    fn dyn_serialize_u64(&mut self, v: u64) -> SerializerResult<()>;

    /// Serialize an `u128` value.
    ///
    /// Also see [`serde::Serializer::serialize_u128`].
    fn dyn_serialize_u128(&mut self, v: u128) -> SerializerResult<()>;

    /// Serialize an `f32` value.
    ///
    /// Also see [`serde::Serializer::serialize_f32`].
    fn dyn_serialize_f32(&mut self, v: f32) -> SerializerResult<()>;

    /// Serialize an `f64` value.
    ///
    /// Also see [`serde::Serializer::serialize_f64`].
    fn dyn_serialize_f64(&mut self, v: f64) -> SerializerResult<()>;

    /// Serialize a character.
    ///
    /// Also see [`serde::Serializer::serialize_char`].
    fn dyn_serialize_char(&mut self, v: char) -> SerializerResult<()>;

    /// Serialize a string.
    ///
    /// Also see [`serde::Serializer::serialize_str`].
    fn dyn_serialize_str(&mut self, v: &str) -> SerializerResult<()>;

    /// Serialize a chunk of raw byte data.
    ///
    /// Also see [`serde::Serializer::serialize_bytes`].
    fn dyn_serialize_bytes(&mut self, v: &[u8]) -> SerializerResult<()>;

    /// Serialize a [`None`] value.
    ///
    /// Also see [`serde::Serializer::serialize_none`].
    fn dyn_serialize_none(&mut self) -> SerializerResult<()>;

    /// Serialize a [`Some`]\(`v`\) value.
    ///
    /// Also see [`serde::Serializer::serialize_some`].
    fn dyn_serialize_some(&mut self, value: &dyn Serialize) -> SerializerResult<()>;

    /// Serialize a `()` value.
    ///
    /// Also see [`serde::Serializer::serialize_unit`].
    fn dyn_serialize_unit(&mut self) -> SerializerResult<()>;

    /// Serialize a unit struct like `struct Unit` or `PhantomData<T>`.
    ///
    /// Also see [`serde::Serializer::serialize_unit_struct`].
    fn dyn_serialize_unit_struct(&mut self, name: &'static str) -> SerializerResult<()>;

    /// Serialize a unit variant like `E::A` in `enum E { A, B }`.
    ///
    /// The `name` is the name of the enum, the `variant_index` is the index of this variant within
    /// the enum, and the `variant` is the name of the variant.
    ///
    /// Also see [`serde::Serializer::serialize_unit_variant`].
    fn dyn_serialize_unit_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> SerializerResult<()>;

    /// Serialize a newtype struct like `struct Millimeters(u8)`.
    ///
    /// Also see [`serde::Serializer::serialize_newtype_struct`].
    fn dyn_serialize_newtype_struct(
        &mut self,
        name: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()>;

    /// Serialize a newtype variant like `E::N` in `enum E { N(u8) }`.
    ///
    /// The `name` is the name of the enum, the `variant_index` is the index of this variant within
    /// the enum, and the `variant` is the name of the variant. The `value` is the data contained
    /// within this newtype variant.
    ///
    /// Also see [`serde::Serializer::serialize_newtype_variant`].
    fn dyn_serialize_newtype_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()>;

    /// Begin to serialize a variably sized sequence. This call must be followed by zero or more
    /// calls to `dyn_serialize_element`, then a call to `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_seq`].
    fn dyn_serialize_seq(&mut self, len: Option<usize>) -> SerializerResult<&mut dyn SerializeSeq>;

    /// Begin to serialize a statically sized sequence whose length will be known at deserialization
    /// time without looking at the serialized data. This call must be followed by zero or more
    /// calls to `dyn_serialize_element`, then a call to `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_tuple`].
    fn dyn_serialize_tuple(&mut self, len: usize) -> SerializerResult<&mut dyn SerializeTuple>;

    /// Begin to serialize a tuple struct like `struct Rgb(u8, u8, u8)`. This call must be followed
    /// by zero or more calls to `dyn_serialize_field`, then a call to `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_tuple_struct`].
    fn dyn_serialize_tuple_struct(
        &mut self,
        name: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeTupleStruct>;

    /// Begin to serialize a tuple variant like `E::T` in `enum E { T(u8, u8) }`. This call must be
    /// followed by zero or more calls to `dyn_serialize_field`, then a call to `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_tuple_variant`].
    fn dyn_serialize_tuple_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeTupleVariant>;

    /// Begin to serialize a map. This call must be followed by zero or more calls to
    /// `dyn_serialize_key` and `dyn_serialize_value`, then a call to `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_map`].
    fn dyn_serialize_map(&mut self, len: Option<usize>) -> SerializerResult<&mut dyn SerializeMap>;

    /// Begin to serialize a struct like `struct Rgb { r: u8, g: u8, b: u8 }`. This call must be
    /// followed by zero or more calls to `dyn_serialize_field`, then a call to `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_struct`].
    fn dyn_serialize_struct(
        &mut self,
        name: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeStruct>;

    /// Begin to serialize a struct variant like `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.
    /// This call must be followed by zero or more calls to `dyn_serialize_field`, then a call to
    /// `dyn_end`.
    ///
    /// Also see [`serde::Serializer::serialize_struct_variant`].
    fn dyn_serialize_struct_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeStructVariant>;

    /// Serialize a string produced by an implementation of `Display`.
    ///
    /// Also see [`serde::Serializer::collect_str`]
    fn dyn_collect_str(&mut self, value: &dyn fmt::Display) -> SerializerResult<()>;

    /// Determine whether `Serialize` implementations should serialize in human-readable form.
    ///
    /// Also see [`serde::Serializer::is_human_readable`]
    fn dyn_is_human_readable(&self) -> bool;
}

impl dyn Serializer {
    /// Returns dynamic [`Serializer`].
    ///
    /// Note that the returned [`InplaceSerializer`] is not an implementation of `Serializer`.
    /// Use a mutable reference instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_dyn::Serializer;
    /// let mut json = Vec::new();
    /// let writer = std::io::Cursor::new(&mut json);
    /// let mut serializer = serde_json::Serializer::new(writer);
    /// let mut serializer = <dyn Serializer>::new(&mut serializer);
    /// let serializer: &mut dyn Serializer = &mut serializer;
    /// # let _ = serializer;
    /// ```
    #[inline]
    #[must_use]
    pub const fn new<S: serde::Serializer>(serializer: S) -> InplaceSerializer<S> {
        InplaceSerializer::Serializer(serializer)
    }
}

impl<'a> serde::Serializer for &'a mut (dyn Serializer + '_) {
    type Ok = ();
    type Error = SerializeError;
    type SerializeSeq = &'a mut dyn SerializeSeq;
    type SerializeTuple = &'a mut dyn SerializeTuple;
    type SerializeTupleStruct = &'a mut dyn SerializeTupleStruct;
    type SerializeTupleVariant = &'a mut dyn SerializeTupleVariant;
    type SerializeMap = &'a mut dyn SerializeMap;
    type SerializeStruct = &'a mut dyn SerializeStruct;
    type SerializeStructVariant = &'a mut dyn SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, v: bool) -> SerializeResult<()> {
        self.dyn_serialize_bool(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> SerializeResult<()> {
        self.dyn_serialize_i8(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> SerializeResult<()> {
        self.dyn_serialize_i16(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> SerializeResult<()> {
        self.dyn_serialize_i32(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> SerializeResult<()> {
        self.dyn_serialize_i64(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> SerializeResult<()> {
        self.dyn_serialize_i128(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> SerializeResult<()> {
        self.dyn_serialize_u8(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> SerializeResult<()> {
        self.dyn_serialize_u16(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> SerializeResult<()> {
        self.dyn_serialize_u32(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> SerializeResult<()> {
        self.dyn_serialize_u64(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> SerializeResult<()> {
        self.dyn_serialize_u128(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> SerializeResult<()> {
        self.dyn_serialize_f32(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> SerializeResult<()> {
        self.dyn_serialize_f64(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_char(self, v: char) -> SerializeResult<()> {
        self.dyn_serialize_char(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> SerializeResult<()> {
        self.dyn_serialize_str(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> SerializeResult<()> {
        self.dyn_serialize_bytes(v).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_none(self) -> SerializeResult<()> {
        self.dyn_serialize_none().map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_some(&value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_unit(self) -> SerializeResult<()> {
        self.dyn_serialize_unit().map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> SerializeResult<()> {
        self.dyn_serialize_unit_struct(name)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> SerializeResult<()> {
        self.dyn_serialize_unit_variant(name, variant_index, variant)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_newtype_struct(name, &value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_newtype_variant(name, variant_index, variant, &value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> SerializeResult<Self::SerializeSeq> {
        self.dyn_serialize_seq(len).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> SerializeResult<Self::SerializeTuple> {
        self.dyn_serialize_tuple(len).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> SerializeResult<Self::SerializeTupleStruct> {
        self.dyn_serialize_tuple_struct(name, len)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerializeResult<Self::SerializeTupleVariant> {
        self.dyn_serialize_tuple_variant(name, variant_index, variant, len)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> SerializeResult<Self::SerializeMap> {
        self.dyn_serialize_map(len).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> SerializeResult<Self::SerializeStruct> {
        self.dyn_serialize_struct(name, len)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerializeResult<Self::SerializeStructVariant> {
        self.dyn_serialize_struct_variant(name, variant_index, variant, len)
            .map_err(SerializeError::from)
    }

    fn collect_seq<I>(self, iter: I) -> SerializeResult<()>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: serde::Serialize,
    {
        let iter = iter.into_iter();
        let len = match iter.size_hint() {
            (lo, Some(hi)) if lo == hi => Some(lo),
            _ => None,
        };
        let serializer = self.dyn_serialize_seq(len)?;
        for item in iter {
            serializer.dyn_serialize_element(&item)?;
        }
        serializer.dyn_end().map_err(SerializeError::from)
    }

    fn collect_map<K, V, I>(self, iter: I) -> SerializeResult<()>
    where
        K: serde::Serialize,
        V: serde::Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let len = match iter.size_hint() {
            (lo, Some(hi)) if lo == hi => Some(lo),
            _ => None,
        };
        let serializer = self.dyn_serialize_map(len)?;
        for (key, value) in iter {
            serializer.dyn_serialize_entry(&key, &value)?;
        }
        serializer.dyn_end().map_err(SerializeError::from)
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + fmt::Display,
    {
        self.dyn_collect_str(&value).map_err(SerializeError::from)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.dyn_is_human_readable()
    }
}

/// Returned from [`Serializer::dyn_serialize_seq`].
///
/// Also see [`serde::ser::SerializeSeq`].
pub trait SerializeSeq {
    /// Serialize a sequence element.
    ///
    /// Also see [`serde::ser::SerializeSeq::serialize_element`].
    fn dyn_serialize_element(&mut self, value: &dyn Serialize) -> SerializerResult<()>;

    /// Finish serializing the sequence.
    ///
    /// Also see [`serde::ser::SerializeSeq::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeSeq for &mut (dyn SerializeSeq + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_element(&value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Returned from [`Serializer::dyn_serialize_tuple`].
///
/// Also see [`serde::ser::SerializeTuple`].
pub trait SerializeTuple {
    /// Serialize a tuple element.
    ///
    /// Also see [`serde::ser::SerializeTuple::serialize_element`].
    fn dyn_serialize_element(&mut self, value: &dyn Serialize) -> SerializerResult<()>;

    /// Finish serializing the tuple.
    ///
    /// Also see [`serde::ser::SerializeTuple::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeTuple for &mut (dyn SerializeTuple + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_element(&value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Returned from [`Serializer::dyn_serialize_tuple_struct`].
///
/// Also see [`serde::ser::SerializeTupleStruct`].
pub trait SerializeTupleStruct {
    /// Serialize a tuple struct field.
    ///
    /// Also see [`serde::ser::SerializeTupleStruct::serialize_field`].
    fn dyn_serialize_field(&mut self, value: &dyn Serialize) -> SerializerResult<()>;

    /// Finish serializing the tuple struct.
    ///
    /// Also see [`serde::ser::SerializeTupleStruct::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeTupleStruct for &mut (dyn SerializeTupleStruct + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(&value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Returned from [`Serializer::dyn_serialize_tuple_variant`].
///
/// Also see [`serde::ser::SerializeTupleVariant`].
pub trait SerializeTupleVariant {
    /// Serialize a tuple variant field.
    ///
    /// Also see [`serde::ser::SerializeTupleVariant::serialize_field`].
    fn dyn_serialize_field(&mut self, value: &dyn Serialize) -> SerializerResult<()>;

    /// Finish serializing the tuple variant.
    ///
    /// Also see [`serde::ser::SerializeTupleVariant::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeTupleVariant for &mut (dyn SerializeTupleVariant + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(&value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Returned from [`Serializer::dyn_serialize_map`].
///
/// Also see [`serde::ser::SerializeMap`].
pub trait SerializeMap {
    /// Serialize a map key.
    ///
    /// Also see [`serde::ser::SerializeMap::serialize_key`].
    fn dyn_serialize_key(&mut self, key: &dyn Serialize) -> SerializerResult<()>;

    /// Serialize a map value.
    ///
    /// Also see [`serde::ser::SerializeMap::serialize_value`].
    fn dyn_serialize_value(&mut self, value: &dyn Serialize) -> SerializerResult<()>;

    /// Serialize a map entry consisting of a key and a value.
    ///
    /// Also see [`serde::ser::SerializeMap::serialize_entry`].
    fn dyn_serialize_entry(
        &mut self,
        key: &dyn Serialize,
        value: &dyn Serialize,
    ) -> SerializerResult<()>;

    /// Finish serializing the map.
    ///
    /// Also see [`serde::ser::SerializeMap::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeMap for &mut (dyn SerializeMap + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_key(&key).map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_value(&value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> SerializeResult<()>
    where
        K: ?Sized + serde::Serialize,
        V: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_entry(&key, &value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Returned from [`Serializer::dyn_serialize_struct`].
///
/// Also see [`serde::ser::SerializeStruct`].
pub trait SerializeStruct {
    /// Serialize a struct field.
    ///
    /// Also see [`serde::ser::SerializeStruct::serialize_field`].
    fn dyn_serialize_field(
        &mut self,
        key: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()>;

    /// Indicate that a struct field has been skipped.
    ///
    /// Also see [`serde::ser::SerializeStruct::skip_field`].
    fn dyn_skip_field(&mut self, key: &'static str) -> SerializerResult<()>;

    /// Finish serializing the struct.
    ///
    /// Also see [`serde::ser::SerializeStruct::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeStruct for &mut (dyn SerializeStruct + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(key, &value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn skip_field(&mut self, key: &'static str) -> SerializeResult<()> {
        self.dyn_skip_field(key).map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Returned from [`Serializer::dyn_serialize_struct_variant`].
///
/// Also see [`serde::ser::SerializeStructVariant`].
pub trait SerializeStructVariant {
    /// Serialize a struct variant field.
    ///
    /// Also see [`serde::ser::SerializeStructVariant::serialize_field`].
    fn dyn_serialize_field(
        &mut self,
        key: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()>;

    /// Indicate that a struct variant field has been skipped.
    ///
    /// Also see [`serde::ser::SerializeStructVariant::skip_field`].
    fn dyn_skip_field(&mut self, key: &'static str) -> SerializerResult<()>;

    /// Finish serializing the struct variant.
    ///
    /// Also see [`serde::ser::SerializeStructVariant::end`].
    fn dyn_end(&mut self) -> SerializerResult<()>;
}

impl serde::ser::SerializeStructVariant for &mut (dyn SerializeStructVariant + '_) {
    type Ok = ();
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SerializeResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(key, &value)
            .map_err(SerializeError::from)
    }

    #[inline]
    fn skip_field(&mut self, key: &'static str) -> SerializeResult<()> {
        self.dyn_skip_field(key).map_err(SerializeError::from)
    }

    #[inline]
    fn end(self) -> SerializeResult<()> {
        self.dyn_end().map_err(SerializeError::from)
    }
}

/// Aliased to [`Result`] type with error type [`SerializerError`].
pub type SerializerResult<T> = Result<T, SerializerError>;

/// An error that is returned by methods defined in [`Serializer`].
///
/// This error just represents that the `Serializer` is in the wrong status. It doesn't implement
/// [`serde::ser::Error`] and tells nothing about why the serialization fails.
///
/// [`Serializer`]: crate::ser::Serializer
#[derive(Clone, Copy, Debug, Default)]
pub enum SerializerError {
    /// An error occurred during serialization.
    #[default]
    Error,
    /// The serializer is not ready.
    Serializer,
    /// The serializer is not ready to serialize the sequence.
    SerializeSeq,
    /// The serializer is not ready to serialize the tuple.
    SerializeTuple,
    /// The serializer is not ready to serialize the tuple struct.
    SerializeTupleStruct,
    /// The serializer is not ready to serialize the tuple variant.
    SerializeTupleVariant,
    /// The serializer is not ready to serialize the map.
    SerializeMap,
    /// The serializer is not ready to serialize the struct.
    SerializeStruct,
    /// The serializer is not ready to serialize the struct variant.
    SerializeStructVariant,
}

impl fmt::Display for SerializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SerializerError::Error => "an error occurred during the serialization",
            SerializerError::Serializer => "the serializer is not ready",
            SerializerError::SerializeSeq => {
                "the serializer is not ready to serialize the sequence"
            }
            SerializerError::SerializeTuple => "the serializer is not ready to serialize the tuple",
            SerializerError::SerializeTupleStruct => {
                "the serializer is not ready to serialize the tuple struct"
            }
            SerializerError::SerializeTupleVariant => {
                "the serializer is not ready to serialize the tuple variant"
            }
            SerializerError::SerializeMap => "the serializer is not ready to serialize the map",
            SerializerError::SerializeStruct => {
                "the serializer is not ready to serialize the struct"
            }
            SerializerError::SerializeStructVariant => {
                "the serializer is not ready to serialize the struct variant"
            }
        })
    }
}

impl error::Error for SerializerError {}

/// Aliased to [`Result`] type with error type [`SerializeError`].
pub type SerializeResult<T> = Result<T, SerializeError>;

/// A universal implementation of the [`serde::ser::Error`] trait.
///
/// This error tells why the dynamic serialization failed and is returned by
/// [`&mut dyn Serializer`](crate::ser::Serializer) as [`serde::Serializer`].
#[repr(transparent)]
#[cfg_attr(not(any(feature = "std", feature = "alloc")), derive(Clone, Copy))]
pub struct SerializeError(
    #[cfg(any(feature = "std", feature = "alloc"))] NonNull<String>,
    #[cfg(not(any(feature = "std", feature = "alloc")))] SerializerError,
);

impl SerializeError {
    #[cold]
    #[must_use]
    fn into_ser_error<E: serde::ser::Error>(self) -> E {
        #[cfg(any(feature = "std", feature = "alloc"))]
        match self.into_string() {
            Err(error) => E::custom(error),
            Ok(error) => E::custom(error),
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            E::custom(self.0)
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl SerializeError {
    const fn encode(error: SerializerError) -> NonZeroUsize {
        const ALIGN: usize = mem::align_of::<String>();
        NonZeroUsize::new(((error as usize) << ALIGN.trailing_zeros()) | ALIGN.strict_sub(1))
            .unwrap()
    }

    const fn decode(error: NonZeroUsize) -> Option<SerializerError> {
        const SERIALIZE_ERROR_ERROR: NonZeroUsize = SerializeError::encode(SerializerError::Error);
        const SERIALIZE_ERROR_SERIALIZER: NonZeroUsize =
            SerializeError::encode(SerializerError::Serializer);
        const SERIALIZE_ERROR_SERIALIZE_SEQ: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeSeq);
        const SERIALIZE_ERROR_SERIALIZE_TUPLE: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeTuple);
        const SERIALIZE_ERROR_SERIALIZE_TUPLE_STRUCT: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeTupleStruct);
        const SERIALIZE_ERROR_SERIALIZE_TUPLE_VARIANT: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeTupleVariant);
        const SERIALIZE_ERROR_SERIALIZE_MAP: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeMap);
        const SERIALIZE_ERROR_SERIALIZE_STRUCT: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeStruct);
        const SERIALIZE_ERROR_SERIALIZE_STRUCT_VARIANT: NonZeroUsize =
            SerializeError::encode(SerializerError::SerializeStructVariant);

        match error {
            SERIALIZE_ERROR_ERROR => Some(SerializerError::Error),
            SERIALIZE_ERROR_SERIALIZER => Some(SerializerError::Serializer),
            SERIALIZE_ERROR_SERIALIZE_SEQ => Some(SerializerError::SerializeSeq),
            SERIALIZE_ERROR_SERIALIZE_TUPLE => Some(SerializerError::SerializeTuple),
            SERIALIZE_ERROR_SERIALIZE_TUPLE_STRUCT => Some(SerializerError::SerializeTupleStruct),
            SERIALIZE_ERROR_SERIALIZE_TUPLE_VARIANT => Some(SerializerError::SerializeTupleVariant),
            SERIALIZE_ERROR_SERIALIZE_MAP => Some(SerializerError::SerializeMap),
            SERIALIZE_ERROR_SERIALIZE_STRUCT => Some(SerializerError::SerializeStruct),
            SERIALIZE_ERROR_SERIALIZE_STRUCT_VARIANT => {
                Some(SerializerError::SerializeStructVariant)
            }
            _ => None,
        }
    }

    fn into_string(self) -> SerializerResult<String> {
        let this = mem::ManuallyDrop::new(self);

        match SerializeError::decode(this.0.expose_provenance()) {
            Some(error) => Err(error),
            // TODO: Replace `Box::from_raw` with `Box::from_non_null` once it's stablized.
            // SAFETY: We have handled the `SerializerError` case.
            None => Ok(*unsafe { Box::from_raw(this.0.as_ptr()) }),
        }
    }

    fn as_string(&self) -> SerializerResult<&String> {
        match SerializeError::decode(self.0.expose_provenance()) {
            Some(error) => Err(error),
            // SAFETY: We have handled the `SerializerError` case.
            None => Ok(unsafe { self.0.as_ref() }),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Drop for SerializeError {
    fn drop(&mut self) {
        const REPLACEMENT: SerializeError = SerializeError(NonNull::without_provenance(
            SerializeError::encode(SerializerError::Error),
        ));

        let _ = mem::replace(self, REPLACEMENT).into_string();
    }
}

impl From<SerializerError> for SerializeError {
    #[cold]
    #[inline(never)]
    fn from(value: SerializerError) -> Self {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            SerializeError(NonNull::without_provenance(SerializeError::encode(value)))
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            SerializeError(value)
        }
    }
}

impl serde::ser::Error for SerializeError {
    #[cold]
    #[inline(never)]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            // TODO: Replace `Box::into_raw` with `Box::into_non_null` once it's stablized.
            SerializeError(
                NonNull::new(Box::into_raw(Box::new(msg.to_string())))
                    .expect("`Box::into_raw` never returns null pointer"),
            )
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let _ = msg;
            SerializeError(SerializerError::Error)
        }
    }
}

impl fmt::Debug for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        match self.as_string() {
            Ok(error) => f.write_str(error),
            Err(error) => error.fmt(f),
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.0.fmt(f)
        }
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        match self.as_string() {
            Ok(error) => f.write_str(error),
            Err(error) => error.fmt(f),
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.0.fmt(f)
        }
    }
}

impl error::Error for SerializeError {}

/// An implementation of the [`Serializer`] trait, which serves as a bridge between dynamic and
/// static serialization.
///
/// This enum is returned by `<dyn Serializer>::new`.
#[derive(Debug, Default)]
pub enum InplaceSerializer<S: serde::Serializer> {
    /// The serializer is not ready.
    #[default]
    None,
    /// The serialization has done successfully.
    Ok(S::Ok),
    /// The serialization has done unsuccessfully.
    Error(S::Error),
    /// The serializer is ready.
    Serializer(S),
    /// The serializer is ready to serialize a sequence.
    SerializeSeq(S::SerializeSeq),
    /// The serializer is ready to serialize a tuple.
    SerializeTuple(S::SerializeTuple),
    /// The serializer is ready to serialize a tuple struct.
    SerializeTupleStruct(S::SerializeTupleStruct),
    /// The serializer is ready to serialize a tuple variant.
    SerializeTupleVariant(S::SerializeTupleVariant),
    /// The serializer is ready to serialize a map.
    SerializeMap(S::SerializeMap),
    /// The serializer is ready to serialize a struct.
    SerializeStruct(S::SerializeStruct),
    /// The serializer is ready to serialize a struct variant.
    SerializeStructVariant(S::SerializeStructVariant),
}

impl<S: serde::Serializer> InplaceSerializer<S> {
    fn write_error(&mut self, error: S::Error) -> SerializerError {
        *self = InplaceSerializer::Error(error);
        SerializerError::Error
    }

    fn write_with<T, F>(&mut self, f: F, r: Result<T, S::Error>) -> SerializerResult<()>
    where
        F: FnOnce(T) -> Self,
    {
        match r {
            #[allow(clippy::unit_arg)]
            Ok(ok) => Ok(*self = f(ok)),
            Err(error) => Err(self.write_error(error)),
        }
    }
}

impl<S: serde::Serializer> Serializer for InplaceSerializer<S> {
    fn dyn_serialize_bool(&mut self, v: bool) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_bool(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i8(&mut self, v: i8) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_i8(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i16(&mut self, v: i16) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_i16(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i32(&mut self, v: i32) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_i32(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i64(&mut self, v: i64) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_i64(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i128(&mut self, v: i128) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_i128(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u8(&mut self, v: u8) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_u8(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u16(&mut self, v: u16) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_u16(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u32(&mut self, v: u32) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_u32(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u64(&mut self, v: u64) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_u64(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u128(&mut self, v: u128) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_u128(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_f32(&mut self, v: f32) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_f32(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_f64(&mut self, v: f64) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_f64(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_char(&mut self, v: char) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_char(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_str(&mut self, v: &str) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_str(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_bytes(&mut self, v: &[u8]) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_bytes(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_none(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_none())
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_some(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_some(value))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_unit(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_unit())
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_unit_struct(&mut self, name: &'static str) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.serialize_unit_struct(name))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_unit_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::Ok,
                ser.serialize_unit_variant(name, variant_index, variant),
            )
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_newtype_struct(
        &mut self,
        name: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::Ok,
                ser.serialize_newtype_struct(name, value),
            )
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_newtype_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::Ok,
                ser.serialize_newtype_variant(name, variant_index, variant, value),
            )
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_seq(&mut self, len: Option<usize>) -> SerializerResult<&mut dyn SerializeSeq> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::SerializeSeq, ser.serialize_seq(len))?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_tuple(&mut self, len: usize) -> SerializerResult<&mut dyn SerializeTuple> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::SerializeTuple, ser.serialize_tuple(len))?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_tuple_struct(
        &mut self,
        name: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeTupleStruct> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::SerializeTupleStruct,
                ser.serialize_tuple_struct(name, len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_tuple_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeTupleVariant> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::SerializeTupleVariant,
                ser.serialize_tuple_variant(name, variant_index, variant, len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_map(&mut self, len: Option<usize>) -> SerializerResult<&mut dyn SerializeMap> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::SerializeMap, ser.serialize_map(len))?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_struct(
        &mut self,
        name: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeStruct> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::SerializeStruct,
                ser.serialize_struct(name, len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_struct_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerializerResult<&mut dyn SerializeStructVariant> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(
                InplaceSerializer::SerializeStructVariant,
                ser.serialize_struct_variant(name, variant_index, variant, len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_collect_str(&mut self, value: &dyn fmt::Display) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.collect_str(value))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_is_human_readable(&self) -> bool {
        if let InplaceSerializer::Serializer(ser) = self {
            ser.is_human_readable()
        } else {
            true
        }
    }
}

impl<S: serde::Serializer> SerializeSeq for InplaceSerializer<S> {
    fn dyn_serialize_element(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        use serde::ser::SerializeSeq;

        if let InplaceSerializer::SerializeSeq(ser) = self {
            ser.serialize_element(value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeSeq)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeSeq;

        if let InplaceSerializer::SerializeSeq(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeSeq)
        }
    }
}

impl<S: serde::Serializer> SerializeTuple for InplaceSerializer<S> {
    fn dyn_serialize_element(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        use serde::ser::SerializeTuple;

        if let InplaceSerializer::SerializeTuple(ser) = self {
            ser.serialize_element(value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeTuple)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeTuple;

        if let InplaceSerializer::SerializeTuple(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeTuple)
        }
    }
}

impl<S: serde::Serializer> SerializeTupleStruct for InplaceSerializer<S> {
    fn dyn_serialize_field(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        use serde::ser::SerializeTupleStruct;

        if let InplaceSerializer::SerializeTupleStruct(ser) = self {
            ser.serialize_field(value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeTupleStruct)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeTupleStruct;

        if let InplaceSerializer::SerializeTupleStruct(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeTupleStruct)
        }
    }
}

impl<S: serde::Serializer> SerializeTupleVariant for InplaceSerializer<S> {
    fn dyn_serialize_field(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        use serde::ser::SerializeTupleVariant;

        if let InplaceSerializer::SerializeTupleVariant(ser) = self {
            ser.serialize_field(value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeTupleVariant)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeTupleVariant;

        if let InplaceSerializer::SerializeTupleVariant(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeTupleVariant)
        }
    }
}

impl<S: serde::Serializer> SerializeMap for InplaceSerializer<S> {
    fn dyn_serialize_key(&mut self, key: &dyn Serialize) -> SerializerResult<()> {
        use serde::ser::SerializeMap;

        if let InplaceSerializer::SerializeMap(ser) = self {
            ser.serialize_key(key)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeMap)
        }
    }

    fn dyn_serialize_value(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        use serde::ser::SerializeMap;

        if let InplaceSerializer::SerializeMap(ser) = self {
            ser.serialize_value(value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeMap)
        }
    }

    fn dyn_serialize_entry(
        &mut self,
        key: &dyn Serialize,
        value: &dyn Serialize,
    ) -> SerializerResult<()> {
        use serde::ser::SerializeMap;

        if let InplaceSerializer::SerializeMap(ser) = self {
            ser.serialize_entry(key, value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeMap)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeMap;

        if let InplaceSerializer::SerializeMap(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeMap)
        }
    }
}

impl<S: serde::Serializer> SerializeStruct for InplaceSerializer<S> {
    fn dyn_serialize_field(
        &mut self,
        key: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()> {
        use serde::ser::SerializeStruct;

        if let InplaceSerializer::SerializeStruct(ser) = self {
            ser.serialize_field(key, value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeStruct)
        }
    }

    fn dyn_skip_field(&mut self, key: &'static str) -> SerializerResult<()> {
        use serde::ser::SerializeStruct;

        if let InplaceSerializer::SerializeStruct(ser) = self {
            ser.skip_field(key).map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeStruct)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeStruct;

        if let InplaceSerializer::SerializeStruct(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeStruct)
        }
    }
}

impl<S: serde::Serializer> SerializeStructVariant for InplaceSerializer<S> {
    fn dyn_serialize_field(
        &mut self,
        key: &'static str,
        value: &dyn Serialize,
    ) -> SerializerResult<()> {
        use serde::ser::SerializeStructVariant;

        if let InplaceSerializer::SerializeStructVariant(ser) = self {
            ser.serialize_field(key, value)
                .map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeStructVariant)
        }
    }

    fn dyn_skip_field(&mut self, key: &'static str) -> SerializerResult<()> {
        use serde::ser::SerializeStructVariant;

        if let InplaceSerializer::SerializeStructVariant(ser) = self {
            ser.skip_field(key).map_err(|error| self.write_error(error))
        } else {
            Err(SerializerError::SerializeStructVariant)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        use serde::ser::SerializeStructVariant;

        if let InplaceSerializer::SerializeStructVariant(ser) = mem::take(self) {
            self.write_with(InplaceSerializer::Ok, ser.end())
        } else {
            Err(SerializerError::SerializeStructVariant)
        }
    }
}
