pub(crate) mod id;
pub(crate) mod object;
pub(crate) mod reference;
pub(crate) mod stream;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::stream::Stream;
use super::direct::DirectValue;
use crate::object::indirect::reference::Reference;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::Byte;
use crate::Offset;

#[derive(Debug, PartialEq)]
pub(crate) enum IndirectValue<'buffer> {
    Stream(Stream<'buffer>),
    Direct(DirectValue<'buffer>),
}

impl Display for IndirectValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Stream(stream) => write!(f, "{}", stream),
            Self::Direct(direct) => write!(f, "{}", direct),
        }
    }
}

impl<'buffer> ObjectParser<'buffer> for IndirectValue<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<Self> {
        match Stream::parse(buffer, offset) {
            Ok(stream) => Ok(IndirectValue::Stream(stream)),
            Err(ParseErr::Recoverable(ParseRecoverable { code, .. })) => {
                if let ParseErrorCode::FoundDictionary(dictionary) = code {
                    Ok(IndirectValue::Direct(DirectValue::Dictionary(dictionary)))
                } else {
                    DirectValue::parse_suppress_recoverable(buffer, offset).unwrap_or_else(|| {
                        Err(ParseRecoverable::new(
                            &buffer[offset..],
                            stringify!(IndirectValue),
                            ParseErrorCode::NotFoundUnion,
                        )
                        .into())
                    })
                }
            }

            Err(err) => Err(err),
        }
    }

    fn span(&self) -> Span {
        match self {
            Self::Stream(stream) => stream.span(),
            Self::Direct(direct) => direct.span(),
        }
    }
}

mod lookup {
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
}

mod convert {

    use super::*;
    use crate::impl_from_ref;
    use crate::object::direct::array::Array;
    use crate::object::direct::boolean::Boolean;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::null::Null;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::numeric::Real;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::object::direct::string::String_;

    impl_from_ref!('buffer, Stream<'buffer>, Stream, IndirectValue<'buffer>);
    impl_from_ref!('buffer, DirectValue<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Reference, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Array<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Boolean, Direct, IndirectValue<'buffer>);
    // impl_from_ref!('buffer, bool, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Dictionary<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Name<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Null, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Integer, Direct, IndirectValue<'buffer>);
    // impl_from_ref!('buffer, u64, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Real, Direct, IndirectValue<'buffer>);
    // impl_from_ref!('buffer, f64, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Numeric, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Hexadecimal<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, Literal<'buffer>, Direct, IndirectValue<'buffer>);
    impl_from_ref!('buffer, String_<'buffer>, Direct, IndirectValue<'buffer>);

    impl<'buffer> IndirectValue<'buffer> {
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
}

// TODO Add tests
