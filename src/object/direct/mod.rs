pub(crate) mod array;
pub(crate) mod boolean;
pub(crate) mod dictionary;
pub(crate) mod name;
pub(crate) mod null;
pub(crate) mod numeric;
pub(crate) mod string;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use array::Array;

use self::array::OwnedArray;
use self::boolean::Boolean;
use self::dictionary::OwnedDictionary;
use self::name::Name;
use self::name::OwnedName;
use self::null::Null;
use self::numeric::Numeric;
use self::string::OwnedString;
use self::string::String_;
use super::BorrowedBuffer;
use crate::object::indirect::reference::Reference;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

/// REFERENCE:
/// - [7.3 Objects, p24]
/// - [7.3.8 Stream objects, p31]
/// Streams are always indirect objects. While `Reference` is not an object, it
/// can substitute for one in some contexts, and it is convenient to treat it as
/// such. Hence, the `DirectValue` enum includes it along with direct objects.
#[derive(Debug, PartialEq, Clone)] // TODO (TEMP) Add Copy
pub enum DirectValue<'buffer> {
    Reference(Reference),
    Array(Array<'buffer>),
    Boolean(Boolean),
    Dictionary(&'buffer OwnedDictionary), // TODO (TEMP) Replace with Dictionary
    Name(Name<'buffer>),
    Null(Null),
    Numeric(Numeric),
    String(String_<'buffer>),
    // Stream(Stream),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OwnedDirectValue {
    Reference(Reference),
    Array(OwnedArray),
    Boolean(Boolean),
    Dictionary(OwnedDictionary),
    Name(OwnedName),
    Null(Null),
    Numeric(Numeric),
    String(OwnedString),
    // Stream(Stream),
}

impl Display for DirectValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Reference(reference) => write!(f, "{}", reference),
            Self::Array(array) => write!(f, "{}", array),
            Self::Boolean(boolean) => write!(f, "{}", boolean),
            Self::Dictionary(dictionary) => write!(f, "{}", dictionary),
            Self::Name(name) => write!(f, "{}", name),
            Self::Null(null) => write!(f, "{}", null),
            Self::Numeric(numeric) => write!(f, "{}", numeric),
            Self::String(string) => write!(f, "{}", string),
        }
    }
}

impl Display for OwnedDirectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&DirectValue::from(self), f)
    }
}

impl<'buffer> Parser<'buffer> for DirectValue<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        Reference::parse_suppress_recoverable(buffer)
            .or_else(|| Null::parse_suppress_recoverable(buffer))
            .or_else(|| Boolean::parse_suppress_recoverable(buffer))
            .or_else(|| Numeric::parse_suppress_recoverable(buffer))
            .or_else(|| Name::parse_suppress_recoverable(buffer))
            .or_else(|| String_::parse_suppress_recoverable(buffer))
            .or_else(|| Array::parse_suppress_recoverable(buffer))
            // .or_else(|| OwnedDictionary::parse_suppress_recoverable(buffer)) // TODO (TEMP) Replace with dictionary.parse_suppress_recoverable(buffer)
            .unwrap_or_else(|| {
                Err(ParseRecoverable {
                    buffer,
                    object: stringify!(DirectValue),
                    code: ParseErrorCode::NotFoundUnion,
                }
                .into())
            })
    }
}

impl Parser<'_> for OwnedDirectValue {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        DirectValue::parse(buffer).map(|(buffer, owned)| (buffer, owned.to_owned_buffer()))
    }
}

mod process {
    use ::std::collections::HashMap;

    use super::*;

    impl DirectValue<'_> {
        pub(crate) fn lookup<'a>(
            &self,
            _parsed_objects: &'a HashMap<Reference, DirectValue>,
        ) -> Option<&'a DirectValue> {
            todo!("Implement lookup and report unfound references")
            // REFERENCE: [7.3.9 Null object, p33]
            // TODO Indirect references to non-existent objects should resolve to null
        }
    }

    impl OwnedDirectValue {
        pub(crate) fn lookup<'a>(
            &self,
            _parsed_objects: &'a HashMap<Reference, OwnedDirectValue>,
        ) -> Option<&'a OwnedDirectValue> {
            todo!("Implement lookup and report unfound references")
            // REFERENCE: [7.3.9 Null object, p33]
            // TODO Indirect references to non-existent objects should resolve to null
        }
    }
}

mod convert {

    use self::numeric::Integer;
    use self::numeric::Real;
    use self::string::OwnedHexadecimal;
    use self::string::OwnedLiteral;
    use super::*;
    use crate::impl_from;
    use crate::impl_from_ref;
    use crate::object::direct::array::OwnedArray;
    use crate::object::direct::boolean::Boolean;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::name::OwnedName;
    use crate::object::direct::null::Null;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::object::direct::string::OwnedString;
    use crate::object::BorrowedBuffer;

    impl BorrowedBuffer for DirectValue<'_> {
        type OwnedBuffer = OwnedDirectValue;

        fn to_owned_buffer(self) -> Self::OwnedBuffer {
            match self {
                DirectValue::Reference(reference) => Self::OwnedBuffer::Reference(reference),
                DirectValue::Array(array) => Self::OwnedBuffer::Array(array.to_owned_buffer()),
                DirectValue::Boolean(boolean) => Self::OwnedBuffer::Boolean(boolean),
                DirectValue::Dictionary(dictionary) => {
                    Self::OwnedBuffer::Dictionary(dictionary.clone())
                } // TODO (TEMP) Replace with dictionary.to_owned_buffer()
                DirectValue::Name(name) => Self::OwnedBuffer::Name(name.to_owned_buffer()),
                DirectValue::Null(null) => Self::OwnedBuffer::Null(null),
                DirectValue::Numeric(numeric) => Self::OwnedBuffer::Numeric(numeric),
                DirectValue::String(string) => Self::OwnedBuffer::String(string.to_owned_buffer()),
            }
        }
    }

    impl<'buffer> From<&'buffer OwnedDirectValue> for DirectValue<'buffer> {
        fn from(value: &'buffer OwnedDirectValue) -> Self {
            match &value {
                OwnedDirectValue::Reference(reference) => Self::Reference(*reference),
                OwnedDirectValue::Array(array) => Self::Array(array.into()),
                OwnedDirectValue::Boolean(boolean) => Self::Boolean(*boolean),
                OwnedDirectValue::Dictionary(dictionary) => Self::Dictionary(dictionary),
                OwnedDirectValue::Name(owned_name) => Self::Name(owned_name.into()),
                OwnedDirectValue::Null(null) => Self::Null(*null),
                OwnedDirectValue::Numeric(numeric) => Self::Numeric(*numeric),
                OwnedDirectValue::String(owned_string) => Self::String(owned_string.into()),
            }
        }
    }

    impl_from_ref!('buffer, Reference, Reference, DirectValue<'buffer>);
    impl_from_ref!('buffer, Array<'buffer>, Array, DirectValue<'buffer>);
    impl_from_ref!('buffer, Boolean, Boolean, DirectValue<'buffer>);
    impl_from_ref!('buffer, bool, Boolean, DirectValue<'buffer>);
    impl_from_ref!('buffer, &'buffer OwnedDictionary, Dictionary, DirectValue<'buffer>);
    impl_from_ref!('buffer, Name<'buffer>, Name, DirectValue<'buffer>);
    impl_from_ref!('buffer, Null, Null, DirectValue<'buffer>);
    impl_from_ref!('buffer, Integer, Numeric, DirectValue<'buffer>);
    impl_from_ref!('buffer, u64, Numeric, DirectValue<'buffer>);
    impl_from_ref!('buffer, Real, Numeric, DirectValue<'buffer>);
    impl_from_ref!('buffer, f64, Numeric, DirectValue<'buffer>);
    impl_from_ref!('buffer, Numeric, Numeric, DirectValue<'buffer>);
    impl_from_ref!('buffer, Hexadecimal<'buffer>, String, DirectValue<'buffer>);
    impl_from_ref!('buffer, Literal<'buffer>, String, DirectValue<'buffer>);
    impl_from_ref!('buffer, String_<'buffer>, String, DirectValue<'buffer>);

    impl_from!(Reference, Reference, OwnedDirectValue);
    impl_from!(OwnedArray, Array, OwnedDirectValue);
    impl_from!(Boolean, Boolean, OwnedDirectValue);
    impl_from!(bool, Boolean, OwnedDirectValue);
    impl_from!(OwnedDictionary, Dictionary, OwnedDirectValue);
    impl_from!(OwnedName, Name, OwnedDirectValue);
    impl_from!(Null, Null, OwnedDirectValue);
    impl_from!(Integer, Numeric, OwnedDirectValue);
    impl_from!(u64, Numeric, OwnedDirectValue);
    impl_from!(Real, Numeric, OwnedDirectValue);
    impl_from!(f64, Numeric, OwnedDirectValue);
    impl_from!(Numeric, Numeric, OwnedDirectValue);
    impl_from!(OwnedHexadecimal, String, OwnedDirectValue);
    impl_from!(OwnedLiteral, String, OwnedDirectValue);
    impl_from!(OwnedString, String, OwnedDirectValue);

    impl DirectValue<'_> {
        pub(crate) fn as_reference(&self) -> Option<&Reference> {
            if let Self::Reference(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_array(&self) -> Option<&Array> {
            if let Self::Array(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_boolean(&self) -> Option<&Boolean> {
            if let Self::Boolean(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_dictionary(&self) -> Option<&OwnedDictionary> {
            if let Self::Dictionary(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_name(&self) -> Option<&Name> {
            if let Self::Name(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_null(&self) -> Option<&Null> {
            if let Self::Null(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_numeric(&self) -> Option<&Numeric> {
            if let Self::Numeric(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_string(&self) -> Option<&String_> {
            if let Self::String(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_u64(&self) -> Option<u64> {
            self.as_numeric()
                .and_then(Numeric::as_integer)
                .and_then(Integer::as_u64)
        }

        pub(crate) fn as_usize(&self) -> Option<usize> {
            self.as_numeric()
                .and_then(Numeric::as_integer)
                .and_then(Integer::as_usize)
        }
    }

    impl OwnedDirectValue {
        pub(crate) fn as_reference(&self) -> Option<&Reference> {
            if let Self::Reference(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_array(&self) -> Option<&OwnedArray> {
            if let Self::Array(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_boolean(&self) -> Option<&Boolean> {
            if let Self::Boolean(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_dictionary(&self) -> Option<&OwnedDictionary> {
            if let Self::Dictionary(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_name(&self) -> Option<&OwnedName> {
            if let Self::Name(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_null(&self) -> Option<&Null> {
            if let Self::Null(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_numeric(&self) -> Option<&Numeric> {
            if let Self::Numeric(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_string(&self) -> Option<&OwnedString> {
            if let Self::String(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_u64(&self) -> Option<u64> {
            self.as_numeric()
                .and_then(Numeric::as_integer)
                .and_then(Integer::as_u64)
        }

        pub(crate) fn as_usize(&self) -> Option<usize> {
            self.as_numeric()
                .and_then(Numeric::as_integer)
                .and_then(Integer::as_usize)
        }
    }
}

// TODO Add tests
