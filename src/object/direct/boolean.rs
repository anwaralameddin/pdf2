use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::combinator::map;
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
use crate::parse::Span;
use crate::parse::KW_FALSE;
use crate::parse::KW_TRUE;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

/// REFERENCE:  [7.3.2 Boolean objects, p24]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Boolean {
    value: bool,
    span: Span,
}

impl Display for Boolean {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

impl Parser<'_> for Boolean {
    fn parse_span(buffer: &[Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
        let (buffer, (value, len)) = alt((
            map(tag::<_, _, NomError<_>>(KW_TRUE), |_true| (true, 4)),
            map(tag(KW_FALSE), |_false| (false, 5)),
        ))(buffer)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Boolean),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;

        let span = Span::new(offset, len);
        Ok((buffer, Self { value, span }))
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl Boolean {
        pub fn new(value: bool, span: Span) -> Self {
            Self { value, span }
        }
    }

    impl Deref for Boolean {
        type Target = bool;

        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse_span_assert_eq;

    #[test]
    fn boolean_valid() {
        parse_span_assert_eq!(b"true", Boolean::new(true, Span::new(0, 4)), "".as_bytes());
        parse_span_assert_eq!(
            b"true<",
            Boolean::new(true, Span::new(0, 4)),
            "<".as_bytes()
        );
        parse_span_assert_eq!(
            b"true    <<",
            Boolean::new(true, Span::new(0, 4)),
            "    <<".as_bytes()
        );

        parse_span_assert_eq!(
            b"false<",
            Boolean::new(false, Span::new(0, 5)),
            "<".as_bytes()
        );
        parse_span_assert_eq!(
            b"false .",
            Boolean::new(false, Span::new(0, 5)),
            " .".as_bytes()
        );

        parse_span_assert_eq!(
            b"true false",
            Boolean::new(true, Span::new(0, 4)),
            " false".as_bytes()
        );
        parse_span_assert_eq!(
            b"false true",
            Boolean::new(false, Span::new(0, 5)),
            " true".as_bytes()
        );
        parse_span_assert_eq!(
            b"truefalse",
            Boolean::new(true, Span::new(0, 4)),
            "false".as_bytes()
        );
        parse_span_assert_eq!(
            b"falsetrue",
            Boolean::new(false, Span::new(0, 5)),
            "true".as_bytes()
        );
    }

    #[test]
    fn boolean_invalid() {
        // Boolean: Not found
        let parse_result = Boolean::parse_span(b"tr", 0);
        let expected_error = ParseRecoverable::new(
            b"tr",
            stringify!(Boolean),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
