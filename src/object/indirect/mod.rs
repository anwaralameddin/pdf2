pub(crate) mod id;
pub(crate) mod object;
pub(crate) mod reference;
pub(crate) mod stream;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::stream::OwnedStream;
use self::stream::Stream;
use super::direct::DirectValue;
use super::BorrowedBuffer;
use crate::object::direct::OwnedDirectValue;
use crate::object::indirect::reference::Reference;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

#[derive(Debug, PartialEq)]
pub(crate) enum IndirectValue<'buffer> {
    Stream(Stream<'buffer>),
    Direct(DirectValue<'buffer>),
}

#[derive(Debug, PartialEq)]
pub(crate) enum OwnedIndirectValue {
    Stream(OwnedStream),
    Direct(OwnedDirectValue),
}

impl Display for IndirectValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Stream(stream) => write!(f, "{}", stream),
            Self::Direct(direct) => write!(f, "{}", direct),
        }
    }
}

impl Display for OwnedIndirectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&IndirectValue::from(self), f)
    }
}

impl<'buffer> Parser<'buffer> for IndirectValue<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        Stream::parse_suppress_recoverable(buffer)
            .or_else(|| DirectValue::parse_suppress_recoverable(buffer))
            .unwrap_or_else(|| {
                Err(ParseRecoverable {
                    buffer,
                    object: stringify!(IndirectValue),
                    code: ParseErrorCode::NotFoundUnion,
                }
                .into())
            })
    }
}

impl Parser<'_> for OwnedIndirectValue {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        IndirectValue::parse(buffer).map(|(buffer, value)| (buffer, value.to_owned_buffer()))
    }
}

mod process {
    use ::std::collections::HashMap;

    use super::*;

    impl IndirectValue<'_> {
        pub(crate) fn lookup<'a>(
            &self,
            _parsed_objects: &'a HashMap<Reference, DirectValue>,
        ) -> Option<&'a DirectValue> {
            todo!("Implement lookup and report unfound references")
            // REFERENCE: [7.3.9 Null object, p33]
            // TODO indirect references to non-existent objects should resolve to null
        }
    }

    impl OwnedIndirectValue {
        pub(crate) fn lookup<'a>(
            &self,
            _parsed_objects: &'a HashMap<Reference, OwnedDirectValue>,
        ) -> Option<&'a OwnedDirectValue> {
            todo!("Implement lookup and report unfound references")
            // REFERENCE: [7.3.9 Null object, p33]
            // TODO indirect references to non-existent objects should resolve to null
        }
    }
}

mod convert {

    use super::*;
    use crate::impl_from;
    use crate::impl_from_ref;
    use crate::object::direct::array::Array;
    use crate::object::direct::array::OwnedArray;
    use crate::object::direct::boolean::Boolean;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::name::OwnedName;
    use crate::object::direct::null::Null;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::numeric::Real;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::object::direct::string::OwnedHexadecimal;
    use crate::object::direct::string::OwnedLiteral;
    use crate::object::direct::string::OwnedString;
    use crate::object::direct::string::String_;
    use crate::object::BorrowedBuffer;

    impl BorrowedBuffer for IndirectValue<'_> {
        type OwnedBuffer = OwnedIndirectValue;

        fn to_owned_buffer(self) -> Self::OwnedBuffer {
            match self {
                IndirectValue::Stream(v) => OwnedIndirectValue::Stream(v.to_owned_buffer()),
                IndirectValue::Direct(v) => OwnedIndirectValue::Direct(v.to_owned_buffer()),
            }
        }
    }

    impl<'buffer> From<&'buffer OwnedIndirectValue> for IndirectValue<'buffer> {
        fn from(value: &'buffer OwnedIndirectValue) -> Self {
            match value {
                OwnedIndirectValue::Stream(v) => IndirectValue::Stream(v.into()),
                OwnedIndirectValue::Direct(v) => IndirectValue::Direct(v.into()),
            }
        }
    }

    impl_from_ref!('buffer, Stream<'buffer>, Stream, IndirectValue<'buffer>);
    impl_from_ref!('buffer, DirectValue<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Reference, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Array<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Boolean, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, bool, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Dictionary<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Name<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Null, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Integer, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, u64, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Real, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, f64, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Numeric, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Hexadecimal<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Literal<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, String_<'buffer>, Direct, IndirectValue<'buffer>);

    impl_from!(OwnedStream, Stream, OwnedIndirectValue);
    impl_from!(OwnedDirectValue, Direct, OwnedIndirectValue);
    impl_from!(Reference, Direct, OwnedIndirectValue);
    impl_from!(OwnedArray, Direct, OwnedIndirectValue);
    impl_from!(Boolean, Direct, OwnedIndirectValue);
    impl_from!(bool, Direct, OwnedIndirectValue);
    impl_from!(OwnedDictionary, Direct, OwnedIndirectValue);
    impl_from!(OwnedName, Direct, OwnedIndirectValue);
    impl_from!(Null, Direct, OwnedIndirectValue);
    impl_from!(Integer, Direct, OwnedIndirectValue);
    impl_from!(u64, Direct, OwnedIndirectValue);
    impl_from!(Real, Direct, OwnedIndirectValue);
    impl_from!(f64, Direct, OwnedIndirectValue);
    impl_from!(Numeric, Direct, OwnedIndirectValue);
    impl_from!(OwnedHexadecimal, Direct, OwnedIndirectValue);
    impl_from!(OwnedLiteral, Direct, OwnedIndirectValue);
    impl_from!(OwnedString, Direct, OwnedIndirectValue);

    impl IndirectValue<'_> {
        pub(crate) fn as_stream(&self) -> Option<&Stream> {
            if let Self::Stream(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_direct(&self) -> Option<&DirectValue> {
            if let Self::Direct(v) = self {
                Some(v)
            } else {
                None
            }
        }
    }

    impl OwnedIndirectValue {
        pub(crate) fn as_stream(&self) -> Option<&OwnedStream> {
            if let Self::Stream(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_direct(&self) -> Option<&OwnedDirectValue> {
            if let Self::Direct(v) = self {
                Some(v)
            } else {
                None
            }
        }
    }
}

// TODO Add tests
