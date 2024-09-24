pub(crate) mod id;
pub(crate) mod object;
pub(crate) mod reference;
pub(crate) mod stream;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::stream::OwnedStream;
use crate::object::direct::OwnedDirectValue;
use crate::object::indirect::reference::Reference;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

#[derive(Debug, PartialEq)]
pub(crate) enum OwnedIndirectValue {
    Stream(OwnedStream),
    Direct(OwnedDirectValue),
}

impl Display for OwnedIndirectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            OwnedIndirectValue::Stream(stream) => write!(f, "{}", stream),
            OwnedIndirectValue::Direct(direct) => write!(f, "{}", direct),
        }
    }
}

impl Parser<'_> for OwnedIndirectValue {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        OwnedStream::parse_suppress_recoverable(buffer)
            .or_else(|| OwnedDirectValue::parse_suppress_recoverable(buffer))
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

mod process {
    use ::std::collections::HashMap;

    use super::*;

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
    use crate::object::direct::array::OwnedArray;
    use crate::object::direct::boolean::Boolean;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::name::OwnedName;
    use crate::object::direct::null::Null;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::numeric::Real;
    use crate::object::direct::string::OwnedHexadecimal;
    use crate::object::direct::string::OwnedLiteral;
    use crate::object::direct::string::OwnedString;

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
