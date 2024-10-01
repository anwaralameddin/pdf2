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
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse::KW_NULL;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.9 Null object, p33]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Null {
    span: Span,
}

impl Display for Null {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "null")
    }
}

impl ObjectParser<'_> for Null {
    fn parse(buffer: &[Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];

        let (..) = tag::<_, _, NomError<_>>(KW_NULL)(remains).map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(e.input, stringify!(Null), ParseErrorCode::NotFound(e.code))
        ))?;

        let span = Span::new(offset, KW_NULL.len());

        Ok(Self { span })
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use super::*;

    impl Null {
        pub fn new(span: Span) -> Self {
            Self { span }
        }
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
        parse_assert_eq!(Null, b"null", Null::new(Span::new(0, 4)));
        parse_assert_eq!(Null, b"null ", Null::new(Span::new(0, 4)));
        parse_assert_eq!(Null, b"null    r", Null::new(Span::new(0, 4)));
        // TODO(QUESTION): Should this be valid?
        parse_assert_eq!(Null, b"nulltrue", Null::new(Span::new(0, 4)));
    }

    #[test]
    fn null_invalid() {
        // Null: Not found
        let parse_result = Null::parse(b"nul", 0);
        let expected_error = ParseRecoverable::new(
            b"nul",
            stringify!(Null),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );

        assert_err_eq!(parse_result, expected_error);
    }
}
