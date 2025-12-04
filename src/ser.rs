//! # Dynamic Serialization.
//!
//! This module provides traits and implementations for dynamic serialization, allowing
//! serialization of trait objects without knowing the concrete type at compile time.
//!
//! - [`Serialize`]: the dyn-compatible version of [`serde::Serialize`].
//! - [`Serializer`]: the dyn-compatible version of [`serde::Serializer`].

use core::{fmt, mem};

use serde::ser::SerializeMap as _;
use serde::ser::SerializeSeq as _;
use serde::ser::SerializeStruct as _;
use serde::ser::SerializeStructVariant as _;
use serde::ser::SerializeTuple as _;
use serde::ser::SerializeTupleStruct as _;
use serde::ser::SerializeTupleVariant as _;

use crate::error::{Error, SerializerError, SerializerResult};

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
/// # use serde::Serialize as _;
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
    fn dyn_serialize(&self, serializer: &mut dyn Serializer) -> Result<(), Error>;
}

/// Automatically derives [`Serialize`] for `T` which implements [`serde::Serialize`].
impl<T: ?Sized + serde::Serialize> Serialize for T {
    fn dyn_serialize(&self, serializer: &mut dyn Serializer) -> Result<(), Error> {
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
            // The `unwrap_err` never panics because `result` is `Ok(_)` if and only if
            // the `serializer` is `Ok(_)`.
            _ => Err(result.unwrap_err().into_ser_error()),
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
    type Error = Error;
    type SerializeSeq = &'a mut dyn SerializeSeq;
    type SerializeTuple = &'a mut dyn SerializeTuple;
    type SerializeTupleStruct = &'a mut dyn SerializeTupleStruct;
    type SerializeTupleVariant = &'a mut dyn SerializeTupleVariant;
    type SerializeMap = &'a mut dyn SerializeMap;
    type SerializeStruct = &'a mut dyn SerializeStruct;
    type SerializeStructVariant = &'a mut dyn SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.dyn_serialize_bool(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.dyn_serialize_i8(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.dyn_serialize_i16(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.dyn_serialize_i32(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.dyn_serialize_i64(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        self.dyn_serialize_i128(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.dyn_serialize_u8(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.dyn_serialize_u16(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.dyn_serialize_u32(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.dyn_serialize_u64(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        self.dyn_serialize_u128(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.dyn_serialize_f32(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.dyn_serialize_f64(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.dyn_serialize_char(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.dyn_serialize_str(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.dyn_serialize_bytes(v).map_err(Error::from)
    }

    #[inline]
    fn serialize_none(self) -> Result<(), Error> {
        self.dyn_serialize_none().map_err(Error::from)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_some(&value).map_err(Error::from)
    }

    #[inline]
    fn serialize_unit(self) -> Result<(), Error> {
        self.dyn_serialize_unit().map_err(Error::from)
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Error> {
        self.dyn_serialize_unit_struct(name).map_err(Error::from)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.dyn_serialize_unit_variant(name, variant_index, variant)
            .map_err(Error::from)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_newtype_struct(name, &value)
            .map_err(Error::from)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_newtype_variant(name, variant_index, variant, &value)
            .map_err(Error::from)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        self.dyn_serialize_seq(len).map_err(Error::from)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.dyn_serialize_tuple(len).map_err(Error::from)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.dyn_serialize_tuple_struct(name, len)
            .map_err(Error::from)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        self.dyn_serialize_tuple_variant(name, variant_index, variant, len)
            .map_err(Error::from)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        self.dyn_serialize_map(len).map_err(Error::from)
    }

    #[inline]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        self.dyn_serialize_struct(name, len).map_err(Error::from)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        self.dyn_serialize_struct_variant(name, variant_index, variant, len)
            .map_err(Error::from)
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + fmt::Display,
    {
        self.dyn_collect_str(&value).map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_element(&value).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_element(&value).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(&value).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(&value).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_key(&key).map_err(Error::from)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_value(&value).map_err(Error::from)
    }

    #[inline]
    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Error>
    where
        K: ?Sized + serde::Serialize,
        V: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_entry(&key, &value).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(key, &value).map_err(Error::from)
    }

    #[inline]
    fn skip_field(&mut self, key: &'static str) -> Result<(), Error> {
        self.dyn_skip_field(key).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
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
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.dyn_serialize_field(key, &value).map_err(Error::from)
    }

    #[inline]
    fn skip_field(&mut self, key: &'static str) -> Result<(), Error> {
        self.dyn_skip_field(key).map_err(Error::from)
    }

    #[inline]
    fn end(self) -> Result<(), Error> {
        self.dyn_end().map_err(Error::from)
    }
}

/// An implementation of the trait [`Serializer`], which serves as a bridge between dynamic and
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
    fn emplace<T, F>(&mut self, f: F, r: Result<T, S::Error>) -> SerializerResult<()>
    where
        F: FnOnce(T) -> Self,
    {
        match r {
            #[allow(clippy::unit_arg)]
            Ok(ok) => Ok(*self = f(ok)),
            Err(err) => {
                *self = InplaceSerializer::Error(err);
                Err(SerializerError::Error)
            }
        }
    }

    fn emplace_error(&mut self, value: Result<(), S::Error>) -> SerializerResult<()> {
        match value {
            Ok(ok) => Ok(ok),
            Err(err) => {
                *self = InplaceSerializer::Error(err);
                Err(SerializerError::Error)
            }
        }
    }
}

impl<S: serde::Serializer> Serializer for InplaceSerializer<S> {
    fn dyn_serialize_bool(&mut self, v: bool) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_bool(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i8(&mut self, v: i8) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_i8(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i16(&mut self, v: i16) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_i16(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i32(&mut self, v: i32) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_i32(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i64(&mut self, v: i64) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_i64(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_i128(&mut self, v: i128) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_i128(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u8(&mut self, v: u8) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_u8(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u16(&mut self, v: u16) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_u16(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u32(&mut self, v: u32) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_u32(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u64(&mut self, v: u64) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_u64(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_u128(&mut self, v: u128) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_u128(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_f32(&mut self, v: f32) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_f32(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_f64(&mut self, v: f64) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_f64(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_char(&mut self, v: char) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_char(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_str(&mut self, v: &str) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_str(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_bytes(&mut self, v: &[u8]) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_bytes(v))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_none(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_none())
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_some(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_some(value))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_unit(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.serialize_unit())
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_unit_struct(&mut self, name: &'static str) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::Ok,
                serializer.serialize_unit_struct(name),
            )
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::Ok,
                serializer.serialize_unit_variant(name, variant_index, variant),
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::Ok,
                serializer.serialize_newtype_struct(name, value),
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::Ok,
                serializer.serialize_newtype_variant(name, variant_index, variant, value),
            )
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_seq(&mut self, len: Option<usize>) -> SerializerResult<&mut dyn SerializeSeq> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeSeq,
                serializer.serialize_seq(len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_tuple(&mut self, len: usize) -> SerializerResult<&mut dyn SerializeTuple> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeTuple,
                serializer.serialize_tuple(len),
            )?;
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeTupleStruct,
                serializer.serialize_tuple_struct(name, len),
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeTupleVariant,
                serializer.serialize_tuple_variant(name, variant_index, variant, len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_serialize_map(&mut self, len: Option<usize>) -> SerializerResult<&mut dyn SerializeMap> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeMap,
                serializer.serialize_map(len),
            )?;
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeStruct,
                serializer.serialize_struct(name, len),
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
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(
                InplaceSerializer::SerializeStructVariant,
                serializer.serialize_struct_variant(name, variant_index, variant, len),
            )?;
            Ok(self)
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_collect_str(&mut self, value: &dyn fmt::Display) -> SerializerResult<()> {
        if let InplaceSerializer::Serializer(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.collect_str(value))
        } else {
            Err(SerializerError::Serializer)
        }
    }

    fn dyn_is_human_readable(&self) -> bool {
        if let InplaceSerializer::Serializer(serializer) = self {
            serializer.is_human_readable()
        } else {
            true
        }
    }
}

impl<S: serde::Serializer> SerializeSeq for InplaceSerializer<S> {
    fn dyn_serialize_element(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeSeq(serializer) = self {
            let result = serializer.serialize_element(value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeSeq)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeSeq(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
        } else {
            Err(SerializerError::SerializeSeq)
        }
    }
}

impl<S: serde::Serializer> SerializeTuple for InplaceSerializer<S> {
    fn dyn_serialize_element(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeTuple(serializer) = self {
            let result = serializer.serialize_element(value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeTuple)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeTuple(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
        } else {
            Err(SerializerError::SerializeTuple)
        }
    }
}

impl<S: serde::Serializer> SerializeTupleStruct for InplaceSerializer<S> {
    fn dyn_serialize_field(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeTupleStruct(serializer) = self {
            let result = serializer.serialize_field(value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeTupleStruct)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeTupleStruct(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
        } else {
            Err(SerializerError::SerializeTupleStruct)
        }
    }
}

impl<S: serde::Serializer> SerializeTupleVariant for InplaceSerializer<S> {
    fn dyn_serialize_field(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeTupleVariant(serializer) = self {
            let result = serializer.serialize_field(value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeTupleVariant)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeTupleVariant(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
        } else {
            Err(SerializerError::SerializeTupleVariant)
        }
    }
}

impl<S: serde::Serializer> SerializeMap for InplaceSerializer<S> {
    fn dyn_serialize_key(&mut self, key: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeMap(serializer) = self {
            let result = serializer.serialize_key(key);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeMap)
        }
    }

    fn dyn_serialize_value(&mut self, value: &dyn Serialize) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeMap(serializer) = self {
            let result = serializer.serialize_value(value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeMap)
        }
    }

    fn dyn_serialize_entry(
        &mut self,
        key: &dyn Serialize,
        value: &dyn Serialize,
    ) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeMap(serializer) = self {
            let result = serializer.serialize_entry(key, value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeMap)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeMap(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
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
        if let InplaceSerializer::SerializeStruct(serializer) = self {
            let result = serializer.serialize_field(key, value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeStruct)
        }
    }

    fn dyn_skip_field(&mut self, key: &'static str) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeStruct(serializer) = self {
            let result = serializer.skip_field(key);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeStruct)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeStruct(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
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
        if let InplaceSerializer::SerializeStructVariant(serializer) = self {
            let result = serializer.serialize_field(key, value);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeStructVariant)
        }
    }

    fn dyn_skip_field(&mut self, key: &'static str) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeStructVariant(serializer) = self {
            let result = serializer.skip_field(key);
            self.emplace_error(result)
        } else {
            Err(SerializerError::SerializeStructVariant)
        }
    }

    fn dyn_end(&mut self) -> SerializerResult<()> {
        if let InplaceSerializer::SerializeStructVariant(serializer) = mem::take(self) {
            self.emplace(InplaceSerializer::Ok, serializer.end())
        } else {
            Err(SerializerError::SerializeStructVariant)
        }
    }
}
