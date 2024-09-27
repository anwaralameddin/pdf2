use ::nom::bytes::complete::tag;
use ::nom::error::Error as NomError;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::KW_NULL;
use crate::parse_recoverable;
use crate::Byte;

/// REFERENCE: [7.3.9 Null object, p33]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Null;

impl Display for Null {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "null")
    }
}

impl Parser<'_> for Null {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, _) = tag::<_, _, NomError<_>>(KW_NULL)(buffer).map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(e.input, stringify!(Null), ParseErrorCode::NotFound(e.code))
        ))?;

        Ok((buffer, Self))
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse_assert_eq;

    #[test]
    fn null_valid() {
        parse_assert_eq!(b"null", Null, "".as_bytes());
        parse_assert_eq!(b"null ", Null, " ".as_bytes());
        parse_assert_eq!(b"null    r", Null, "    r".as_bytes());
        // TODO(QUESTION): Should this be valid?
        parse_assert_eq!(b"nulltrue", Null, "true".as_bytes());
    }

    #[test]
    fn null_invalid() {
        // Null: Not found
        let parse_result = Null::parse(b"nul");
        let expected_error = ParseRecoverable::new(
            b"nul",
            stringify!(Null),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );

        assert_err_eq!(parse_result, expected_error);
    }
}
