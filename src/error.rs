//! Error handling in the dynamic serialization and deserialization.
//!
//! In the [`serde`], serialization and deserialization errors are generic and are bounded by
//! [`serde::ser::Error`] and [`serde::de::Error`] respectively. Serializers and deserializers
//! have their own errors. In order to achieve type erasure, this crate provides a universal
//! [`Error`] that satifies both `serde::ser::Error` and `serde::de::Error`.

use core::{error, fmt};

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// [`Result`] type alias with error type [`SerializerError`].
pub type SerializerResult<T> = Result<T, SerializerError>;

/// [`Result`] type alias with error type [`DeserializerError`].
pub type DeserializerResult<T> = Result<T, DeserializerError>;

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

/// A universal implementation of the [`serde::ser::Error`] and [`serde::de::Error`].
///
/// This error represents various kinds of errors that might occur during dynamic serialization and
/// deserialization. And it can be returned by:
///
/// - [`&mut dyn Serializer`](crate::ser::Serializer) as [`serde::Serializer`], and
/// - [`&mut dyn Deserializer`](crate::de::Deserializer) as [`serde::Deserializer`].
#[repr(transparent)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// Converts the error into an arbitrary [`serde::ser::Error`].
    #[cold]
    #[must_use]
    pub fn into_ser_error<E: serde::ser::Error>(self) -> E {
        match *self.0 {
            ErrorKind::SerializerError(error) => E::custom(error),
            ErrorKind::DeserializerError(error) => E::custom(error),
            ErrorKind::Other(error) => E::custom(error),
            error => E::custom(error),
        }
    }

    /// Converts the error into an arbitrary [`serde::de::Error`].
    #[cold]
    #[must_use]
    pub fn into_de_error<E: serde::de::Error>(self) -> E {
        match *self.0 {
            ErrorKind::SerializerError(error) => E::custom(error),
            ErrorKind::DeserializerError(error) => E::custom(error),
            ErrorKind::InvalidType(unexp, exp) => E::invalid_type(unexp.borrow(), &exp.as_str()),
            ErrorKind::InvalidValue(unexp, exp) => E::invalid_value(unexp.borrow(), &exp.as_str()),
            ErrorKind::InvalidLength(len, exp) => E::invalid_length(len, &exp.as_str()),
            ErrorKind::UnknownVariant(variant, expected) => {
                E::unknown_variant(variant.as_str(), expected)
            }
            ErrorKind::UnknownField(field, expected) => E::unknown_field(field.as_str(), expected),
            ErrorKind::MissingField(field) => E::missing_field(field),
            ErrorKind::DuplicateField(field) => E::duplicate_field(field),
            ErrorKind::Other(error) => E::custom(error),
        }
    }
}

impl From<SerializerError> for Error {
    #[cold]
    #[inline(never)]
    fn from(error: SerializerError) -> Self {
        Error(Box::new(ErrorKind::SerializerError(error)))
    }
}

impl From<DeserializerError> for Error {
    #[cold]
    #[inline(never)]
    fn from(error: DeserializerError) -> Self {
        Error(Box::new(ErrorKind::DeserializerError(error)))
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.0.as_ref() {
            ErrorKind::SerializerError(error) => Some(error),
            ErrorKind::DeserializerError(error) => Some(error),
            _ => None,
        }
    }
}

impl serde::ser::Error for Error {
    #[cold]
    #[inline(never)]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(Box::new(ErrorKind::Other(msg.to_string())))
    }
}

impl serde::de::Error for Error {
    #[cold]
    #[inline(never)]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(Box::new(ErrorKind::Other(msg.to_string())))
    }

    #[cold]
    #[inline(never)]
    fn invalid_type(unexp: serde::de::Unexpected, exp: &dyn serde::de::Expected) -> Self {
        Error(Box::new(ErrorKind::InvalidType(
            Unexpected::from(unexp),
            exp.to_string(),
        )))
    }

    #[cold]
    #[inline(never)]
    fn invalid_value(unexp: serde::de::Unexpected, exp: &dyn serde::de::Expected) -> Self {
        Error(Box::new(ErrorKind::InvalidValue(
            Unexpected::from(unexp),
            exp.to_string(),
        )))
    }

    #[cold]
    #[inline(never)]
    fn invalid_length(len: usize, exp: &dyn serde::de::Expected) -> Self {
        Error(Box::new(ErrorKind::InvalidLength(len, exp.to_string())))
    }

    #[cold]
    #[inline(never)]
    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Error(Box::new(ErrorKind::UnknownVariant(
            String::from(variant),
            expected,
        )))
    }

    #[cold]
    #[inline(never)]
    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Error(Box::new(ErrorKind::UnknownField(
            String::from(field),
            expected,
        )))
    }

    #[cold]
    #[inline(never)]
    fn missing_field(field: &'static str) -> Self {
        Error(Box::new(ErrorKind::MissingField(field)))
    }

    #[cold]
    #[inline(never)]
    fn duplicate_field(field: &'static str) -> Self {
        Error(Box::new(ErrorKind::DuplicateField(field)))
    }
}

/// The internal data of the wrapper struct `Error`.
#[derive(Debug)]
enum ErrorKind {
    /// The serializer is in the wrong status.
    SerializerError(SerializerError),

    /// The deserializer is in the wrong status.
    DeserializerError(DeserializerError),

    /// An error returned by [`serde::de::Error::invalid_type`].
    InvalidType(Unexpected, String),

    /// An error returned by [`serde::de::Error::invalid_value`].
    InvalidValue(Unexpected, String),

    /// An error returned by [`serde::de::Error::invalid_length`].
    InvalidLength(usize, String),

    /// An error returned by [`serde::de::Error::unknown_variant`].
    UnknownVariant(String, &'static [&'static str]),

    /// An error returned by [`serde::de::Error::unknown_field`].
    UnknownField(String, &'static [&'static str]),

    /// An error returned by [`serde::de::Error::missing_field`].
    MissingField(&'static str),

    /// An error returned by [`serde::de::Error::duplicate_field`].
    DuplicateField(&'static str),

    /// A custom error that is returned by [`serde::ser::Error::custom`] and
    /// [`serde::de::Error::custom`].
    Other(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct OneOf(&'static [&'static str]);

        impl fmt::Display for OneOf {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
                    [] => f.write_str(""),
                    [x] => write!(f, "`{x}`"),
                    [x, y] => write!(f, "`{x}` or `{y}`"),
                    [x, ys @ ..] => {
                        write!(f, "one of `{x}`")?;
                        for y in ys {
                            write!(f, ", `{y}`")?;
                        }
                        Ok(())
                    }
                }
            }
        }

        match self {
            ErrorKind::SerializerError(error) => error.fmt(f),
            ErrorKind::DeserializerError(error) => error.fmt(f),
            ErrorKind::InvalidType(unexp, exp) => {
                write!(f, "invalid type: {unexp}, expected {exp}")
            }
            ErrorKind::InvalidValue(unexp, exp) => {
                write!(f, "invalid value: {unexp}, expected {exp}")
            }
            ErrorKind::InvalidLength(len, exp) => {
                write!(f, "invalid length: {len}, expected {exp}")
            }
            ErrorKind::UnknownVariant(variant, expected) => {
                if expected.is_empty() {
                    write!(f, "unknown variant `{variant}`, there are no variants")
                } else {
                    let expected = OneOf(expected);
                    write!(f, "unknown variant `{variant}`, expected {expected}")
                }
            }
            ErrorKind::UnknownField(field, expected) => {
                if expected.is_empty() {
                    write!(f, "unknown field `{field}`, there are no fields")
                } else {
                    let expected = OneOf(expected);
                    write!(f, "unknown field `{field}`, expected {expected}")
                }
            }
            ErrorKind::MissingField(field) => write!(f, "missing field `{field}`"),
            ErrorKind::DuplicateField(field) => write!(f, "duplicate field `{field}`"),
            ErrorKind::Other(error) => f.write_str(error),
        }
    }
}

/// An owned version of the enum [`serde::de::Unexpected`].
#[derive(Debug)]
enum Unexpected {
    /// The input contained a boolean value that was not expected.
    Bool(bool),

    /// The input contained an unsigned integer `u8`, `u16`, `u32` or `u64` that
    /// was not expected.
    Unsigned(u64),

    /// The input contained a signed integer `i8`, `i16`, `i32` or `i64` that
    /// was not expected.
    Signed(i64),

    /// The input contained a floating point `f32` or `f64` that was not
    /// expected.
    Float(f64),

    /// The input contained a `char` that was not expected.
    Char(char),

    /// The input contained a `&str` or `String` that was not expected.
    Str(String),

    /// The input contained a `&[u8]` or `Vec<u8>` that was not expected.
    Bytes(Vec<u8>),

    /// The input contained a unit `()` that was not expected.
    Unit,

    /// The input contained an `Option<T>` that was not expected.
    Option,

    /// The input contained a newtype struct that was not expected.
    NewtypeStruct,

    /// The input contained a sequence that was not expected.
    Seq,

    /// The input contained a map that was not expected.
    Map,

    /// The input contained an enum that was not expected.
    Enum,

    /// The input contained a unit variant that was not expected.
    UnitVariant,

    /// The input contained a newtype variant that was not expected.
    NewtypeVariant,

    /// The input contained a tuple variant that was not expected.
    TupleVariant,

    /// The input contained a struct variant that was not expected.
    StructVariant,

    /// A message stating what uncategorized thing the input contained that was
    /// not expected.
    Other(String),
}

impl Unexpected {
    /// Returns the borrowed version - `serde::de::Unexpected`.
    fn borrow(&self) -> serde::de::Unexpected<'_> {
        match self {
            Unexpected::Bool(v) => serde::de::Unexpected::Bool(*v),
            Unexpected::Unsigned(v) => serde::de::Unexpected::Unsigned(*v),
            Unexpected::Signed(v) => serde::de::Unexpected::Signed(*v),
            Unexpected::Float(v) => serde::de::Unexpected::Float(*v),
            Unexpected::Char(v) => serde::de::Unexpected::Char(*v),
            Unexpected::Str(v) => serde::de::Unexpected::Str(v),
            Unexpected::Bytes(v) => serde::de::Unexpected::Bytes(v),
            Unexpected::Unit => serde::de::Unexpected::Unit,
            Unexpected::Option => serde::de::Unexpected::Option,
            Unexpected::NewtypeStruct => serde::de::Unexpected::NewtypeStruct,
            Unexpected::Seq => serde::de::Unexpected::Seq,
            Unexpected::Map => serde::de::Unexpected::Map,
            Unexpected::Enum => serde::de::Unexpected::Enum,
            Unexpected::UnitVariant => serde::de::Unexpected::UnitVariant,
            Unexpected::NewtypeVariant => serde::de::Unexpected::NewtypeVariant,
            Unexpected::TupleVariant => serde::de::Unexpected::TupleVariant,
            Unexpected::StructVariant => serde::de::Unexpected::StructVariant,
            Unexpected::Other(v) => serde::de::Unexpected::Other(v),
        }
    }
}

impl From<serde::de::Unexpected<'_>> for Unexpected {
    fn from(value: serde::de::Unexpected<'_>) -> Self {
        match value {
            serde::de::Unexpected::Bool(v) => Unexpected::Bool(v),
            serde::de::Unexpected::Unsigned(v) => Unexpected::Unsigned(v),
            serde::de::Unexpected::Signed(v) => Unexpected::Signed(v),
            serde::de::Unexpected::Float(v) => Unexpected::Float(v),
            serde::de::Unexpected::Char(v) => Unexpected::Char(v),
            serde::de::Unexpected::Str(v) => Unexpected::Str(String::from(v)),
            serde::de::Unexpected::Bytes(v) => Unexpected::Bytes(Vec::from(v)),
            serde::de::Unexpected::Unit => Unexpected::Unit,
            serde::de::Unexpected::Option => Unexpected::Option,
            serde::de::Unexpected::NewtypeStruct => Unexpected::NewtypeStruct,
            serde::de::Unexpected::Seq => Unexpected::Seq,
            serde::de::Unexpected::Map => Unexpected::Map,
            serde::de::Unexpected::Enum => Unexpected::Enum,
            serde::de::Unexpected::UnitVariant => Unexpected::UnitVariant,
            serde::de::Unexpected::NewtypeVariant => Unexpected::NewtypeVariant,
            serde::de::Unexpected::TupleVariant => Unexpected::TupleVariant,
            serde::de::Unexpected::StructVariant => Unexpected::StructVariant,
            serde::de::Unexpected::Other(v) => Unexpected::Other(String::from(v)),
        }
    }
}

impl fmt::Display for Unexpected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unexpected::Bool(v) => write!(formatter, "boolean `{v}`"),
            Unexpected::Unsigned(v) => write!(formatter, "integer `{v}`"),
            Unexpected::Signed(v) => write!(formatter, "integer `{v}`"),
            Unexpected::Float(v) => write!(formatter, "floating point `{v}`"),
            Unexpected::Char(v) => write!(formatter, "character `{v}`"),
            Unexpected::Str(v) => write!(formatter, "string {v:?}"),
            Unexpected::Bytes(_) => formatter.write_str("byte array"),
            Unexpected::Unit => formatter.write_str("unit value"),
            Unexpected::Option => formatter.write_str("optional value"),
            Unexpected::NewtypeStruct => formatter.write_str("newtype struct"),
            Unexpected::Seq => formatter.write_str("sequence"),
            Unexpected::Map => formatter.write_str("map"),
            Unexpected::Enum => formatter.write_str("enum"),
            Unexpected::UnitVariant => formatter.write_str("unit variant"),
            Unexpected::NewtypeVariant => formatter.write_str("newtype variant"),
            Unexpected::TupleVariant => formatter.write_str("tuple variant"),
            Unexpected::StructVariant => formatter.write_str("struct variant"),
            Unexpected::Other(v) => formatter.write_str(v),
        }
    }
}
