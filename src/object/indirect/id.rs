use ::nom::sequence::pair;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::num::ParseIntError;

use self::error::IdRecoverable;
use crate::fmt::debug_bytes;
use crate::parse::character_set::number1;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_error;
use crate::Byte;
use crate::GenerationNumber;
use crate::ObjectNumber;

/// REFERENCE: [7.3.10 Indirect Objects, p33]
/// The object identifier shall consist of two parts:
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Id {
    pub(crate) object_number: ObjectNumber,
    pub(crate) generation_number: GenerationNumber,
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.object_number, self.generation_number)
    }
}

impl Parser for Id {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, (object_number, generation)) = pair(
            terminated(number1, white_space_or_comment),
            terminated(number1, white_space_or_comment),
        )(buffer)
        .map_err(parse_error!(
            e,
            IdRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(buffer),
            }
        ))?;
        // This method should not return failure, as the pair of numbers could
        // be part of an array of numbers, not an Id.

        let object_number = object_number.parse().map_err(|err: ParseIntError| {
            ParseErr::Error(
                IdRecoverable::ObjectNumber(err.kind().clone(), object_number.to_string()).into(),
            )
        })?;

        let generation_number = generation.parse().map_err(|err: ParseIntError| {
            ParseErr::Error(
                IdRecoverable::GenerationNumber(err.kind().clone(), generation.to_string()).into(),
            )
        })?;

        let id = Self {
            object_number,
            generation_number,
        };
        Ok((buffer, id))
    }
}

mod convert {
    use super::*;

    impl Id {
        pub(crate) fn new(
            object_number: impl Into<ObjectNumber>,
            generation_number: impl Into<GenerationNumber>,
        ) -> Self {
            Self {
                object_number: object_number.into(),
                generation_number: generation_number.into(),
            }
        }
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::std::num::IntErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IdRecoverable {
        #[error("Invalid object number: {0:?}. Input: {1}")]
        ObjectNumber(IntErrorKind, String),
        #[error("Invalid generation number: {0:?}. Input: {1}")]
        GenerationNumber(IntErrorKind, String),
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;
    use ::std::num::IntErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse_assert_eq;

    impl Id {
        pub(crate) unsafe fn new_unchecked(
            object_number: u64,
            generation_number: GenerationNumber,
        ) -> Self {
            Self {
                object_number: ObjectNumber::new_unchecked(object_number),
                generation_number,
            }
        }
    }
    #[test]
    fn id_valid() {
        // Synthetic tests
        parse_assert_eq!(
            b"65535 65535 R <<",
            unsafe { Id::new_unchecked(65535, 65535) },
            "R <<".as_bytes(),
        );
    }

    #[test]
    fn id_invalid() {
        // Synthetic tests
        // Id: Not found
        let parse_result = Id::parse(b"/Name");
        let expected_error = ParseErr::Error(
            IdRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "/Name".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Missing generation number
        let parse_result = Id::parse(b"56789");
        let expected_error = ParseErr::Error(
            IdRecoverable::NotFound {
                code: ErrorKind::Char,
                input: "56789".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Starts with a negative number
        let parse_result = Id::parse(b"-12345 65535 R other objects");
        let expected_error = ParseErr::Error(
            IdRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "-12345 65535 R other objects".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Zero Object number
        let parse_result = Id::parse(b"0 65535 R other objects");
        let expected_error = ParseErr::Error(
            IdRecoverable::ObjectNumber(IntErrorKind::Zero, "0".to_string()).into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Object number too large
        let parse_result = Id::parse(b"98765432109876543210 65535 R other objects");
        let expected_error = ParseErr::Error(
            IdRecoverable::ObjectNumber(
                IntErrorKind::PosOverflow,
                "98765432109876543210".to_string(),
            )
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Generation number too large
        let parse_result = Id::parse(b"1 65536 R other objects");
        let expected_error = ParseErr::Error(
            IdRecoverable::GenerationNumber(IntErrorKind::PosOverflow, "65536".to_string()).into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Generation number too large
        let parse_result = Id::parse(b"1 6553500 R other objects");
        let expected_error = ParseErr::Error(
            IdRecoverable::GenerationNumber(IntErrorKind::PosOverflow, "6553500".to_string())
                .into(),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
