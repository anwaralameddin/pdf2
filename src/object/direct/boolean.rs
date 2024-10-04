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
use crate::parse::ObjectParser;
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

impl ObjectParser<'_> for Boolean {
    fn parse(buffer: &[Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];
        let start = offset;

        let (_, (value, len)) = alt((
            map(tag::<_, _, NomError<_>>(KW_TRUE), |_true| (true, 4)),
            map(tag(KW_FALSE), |_false| (false, 5)),
        ))(remains)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Boolean),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        let offset = offset + len;

        let span = Span::new(start, offset);
        Ok(Self { value, span })
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
    use crate::parse_assert_eq;

    #[test]
    fn boolean_valid() {
        parse_assert_eq!(Boolean, b"true", Boolean::new(true, Span::new(0, 4)));
        parse_assert_eq!(Boolean, b"true<", Boolean::new(true, Span::new(0, 4)),);
        parse_assert_eq!(Boolean, b"true    <<", Boolean::new(true, Span::new(0, 4)),);

        parse_assert_eq!(Boolean, b"false<", Boolean::new(false, Span::new(0, 5)),);
        parse_assert_eq!(Boolean, b"false .", Boolean::new(false, Span::new(0, 5)),);

        parse_assert_eq!(Boolean, b"true false", Boolean::new(true, Span::new(0, 4)),);
        parse_assert_eq!(Boolean, b"false true", Boolean::new(false, Span::new(0, 5)),);
        parse_assert_eq!(Boolean, b"truefalse", Boolean::new(true, Span::new(0, 4)),);
        parse_assert_eq!(Boolean, b"falsetrue", Boolean::new(false, Span::new(0, 5)),);
    }

    #[test]
    fn boolean_invalid() {
        // Boolean: Not found
        let parse_result = Boolean::parse(b"tr", 0);
        let expected_error = ParseRecoverable::new(
            b"tr",
            stringify!(Boolean),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
