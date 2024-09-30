use ::nom::character::complete::digit1;
use ::nom::sequence::pair;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_u16;
use crate::parse::num::ascii_to_u64;
use crate::parse::PdfParser;
use crate::parse_recoverable;
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

impl PdfParser<'_> for Id {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, (object_number, generation_number)) = pair(
            terminated(digit1, white_space_or_comment),
            terminated(digit1, white_space_or_comment),
        )(buffer)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(e.input, stringify!(Id), ParseErrorCode::NotFound(e.code))
        ))?;
        // This method should not return failure, as the pair of numbers could
        // be part of an array of numbers, not an Id.

        let object_number = ascii_to_u64(object_number)
            .and_then(ObjectNumber::new)
            .ok_or_else(|| {
                ParseRecoverable::new(object_number, stringify!(Id), ParseErrorCode::ObjectNumber)
            })?;

        let generation_number = ascii_to_u16(generation_number).ok_or_else(|| {
            ParseRecoverable::new(
                generation_number,
                stringify!(Id),
                ParseErrorCode::GenerationNumber,
            )
        })?;

        let id = Self {
            object_number,
            generation_number,
        };
        Ok((buffer, id))
    }

    fn spans(&self) -> Vec<crate::parse::Span> {
        unreachable!("Id spans are covered by Reference and Object")
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

#[cfg(test)]
mod tests {

    use ::nom::error::ErrorKind;

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
        let expected_error = ParseRecoverable::new(
            b"/Name",
            stringify!(Id),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );

        assert_err_eq!(parse_result, expected_error);

        // Id: Missing generation number
        let parse_result = Id::parse(b"56789 ");
        let expected_error = ParseRecoverable::new(
            b"",
            stringify!(Id),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Starts with a negative number
        let parse_result = Id::parse(b"-12345 65535 R other objects");
        let expected_error = ParseRecoverable::new(
            b"-12345 65535 R other objects",
            stringify!(Id),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Zero Object number
        let parse_result = Id::parse(b"0 65535 R other objects");
        let expected_error =
            ParseRecoverable::new(b"0", stringify!(Id), ParseErrorCode::ObjectNumber);
        assert_err_eq!(parse_result, expected_error);

        // Id: Object number too large
        let parse_result = Id::parse(b"98765432109876543210 65535 R other objects");
        let expected_error = ParseRecoverable::new(
            b"98765432109876543210",
            stringify!(Id),
            ParseErrorCode::ObjectNumber,
        );
        assert_err_eq!(parse_result, expected_error);

        // Id: Generation number too large
        let parse_result = Id::parse(b"1 65536 R other objects");
        let expected_error =
            ParseRecoverable::new(b"65536", stringify!(Id), ParseErrorCode::GenerationNumber);
        assert_err_eq!(parse_result, expected_error);

        // Id: Generation number too large
        let parse_result = Id::parse(b"1 6553500 R other objects");
        let expected_error =
            ParseRecoverable::new(b"6553500", stringify!(Id), ParseErrorCode::GenerationNumber);
        assert_err_eq!(parse_result, expected_error);
    }
}
