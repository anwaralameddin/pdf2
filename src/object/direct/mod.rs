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

use self::array::Array;
use self::boolean::Boolean;
use self::dictionary::Dictionary;
use self::name::Name;
use self::null::Null;
use self::numeric::Numeric;
use self::string::String_;
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
#[derive(Debug, PartialEq, Clone)]
pub enum DirectValue {
    Reference(Reference),
    Array(Array),
    Boolean(Boolean),
    Dictionary(Dictionary),
    Name(Name),
    Null(Null),
    Numeric(Numeric),
    String(String_),
    // Stream(Stream),
}

impl Display for DirectValue {
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

impl Parser<'_> for DirectValue {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Reference::parse_suppress_recoverable(buffer)
            .or_else(|| Null::parse_suppress_recoverable(buffer))
            .or_else(|| Boolean::parse_suppress_recoverable(buffer))
            .or_else(|| Numeric::parse_suppress_recoverable(buffer))
            .or_else(|| Name::parse_suppress_recoverable(buffer))
            .or_else(|| String_::parse_suppress_recoverable(buffer))
            .or_else(|| Array::parse_suppress_recoverable(buffer))
            .or_else(|| Dictionary::parse_suppress_recoverable(buffer))
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

mod process {
    use ::std::collections::HashMap;

    use super::*;

    impl DirectValue {
        pub(crate) fn lookup<'a>(
            &self,
            _parsed_objects: &'a HashMap<Reference, DirectValue>,
        ) -> Option<&'a DirectValue> {
            todo!("Implement lookup and report unfound references")
            // REFERENCE: [7.3.9 Null object, p33]
            // TODO Indirect references to non-existent objects should resolve to null
        }
    }
}

mod convert {

    use self::numeric::Integer;
    use self::numeric::Real;
    use self::string::Hexadecimal;
    use self::string::Literal;
    use super::*;
    use crate::impl_from;
    use crate::object::direct::array::Array;
    use crate::object::direct::boolean::Boolean;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::null::Null;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::string::String_;

    impl_from!(Reference, Reference, DirectValue);
    impl_from!(Array, Array, DirectValue);
    impl_from!(Boolean, Boolean, DirectValue);
    impl_from!(bool, Boolean, DirectValue);
    impl_from!(Dictionary, Dictionary, DirectValue);
    impl_from!(Name, Name, DirectValue);
    impl_from!(Null, Null, DirectValue);
    impl_from!(Integer, Numeric, DirectValue);
    impl_from!(u64, Numeric, DirectValue);
    impl_from!(Real, Numeric, DirectValue);
    impl_from!(f64, Numeric, DirectValue);
    impl_from!(Numeric, Numeric, DirectValue);
    impl_from!(Hexadecimal, String, DirectValue);
    impl_from!(Literal, String, DirectValue);
    impl_from!(String_, String, DirectValue);

    impl DirectValue {
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

        pub(crate) fn as_dictionary(&self) -> Option<&Dictionary> {
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
}

// TODO Add tests
