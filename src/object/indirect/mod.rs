pub(crate) mod id;
pub(crate) mod object;
pub(crate) mod reference;
pub(crate) mod stream;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::IndirectValueRecoverable;
use self::stream::Stream;
use crate::fmt::debug_bytes;
use crate::object::direct::DirectValue;
use crate::object::indirect::reference::Reference;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

#[derive(Debug, PartialEq)]
pub(crate) enum IndirectValue {
    Stream(Stream),
    Direct(DirectValue),
}

impl Display for IndirectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            IndirectValue::Stream(stream) => write!(f, "{}", stream),
            IndirectValue::Direct(direct) => write!(f, "{}", direct),
        }
    }
}

impl Parser for IndirectValue {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Stream::parse_semi_quiet(buffer)
            .or_else(|| DirectValue::parse_semi_quiet(buffer))
            .unwrap_or_else(|| {
                Err(ParseErr::Error(
                    IndirectValueRecoverable::NotFound(debug_bytes(buffer)).into(),
                ))
            })
    }
}

mod process {
    use ::std::collections::HashMap;

    use super::*;

    impl IndirectValue {
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
    use crate::impl_from;
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

    impl_from!(Stream, Stream, IndirectValue);
    impl_from!(DirectValue, Direct, IndirectValue);
    impl_from!(Reference, Direct, IndirectValue);
    impl_from!(Array, Direct, IndirectValue);
    impl_from!(Boolean, Direct, IndirectValue);
    impl_from!(bool, Direct, IndirectValue);
    impl_from!(Dictionary, Direct, IndirectValue);
    impl_from!(Name, Direct, IndirectValue);
    impl_from!(Null, Direct, IndirectValue);
    impl_from!(Integer, Direct, IndirectValue);
    impl_from!(u64, Direct, IndirectValue);
    impl_from!(Real, Direct, IndirectValue);
    impl_from!(f64, Direct, IndirectValue);
    impl_from!(Numeric, Direct, IndirectValue);
    impl_from!(Hexadecimal, Direct, IndirectValue);
    impl_from!(Literal, Direct, IndirectValue);
    impl_from!(String_, Direct, IndirectValue);

    impl IndirectValue {
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

pub(crate) mod error {
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IndirectValueRecoverable {
        #[error("Not found: {0}")]
        NotFound(String),
    }
}

// TODO Add tests
