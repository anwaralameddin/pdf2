use ::nom::branch::alt;
use ::nom::character::complete::char;
use ::nom::character::complete::digit1;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::nom::sequence::pair;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_i128;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Integer {
    value: i128,
    span: Span,
}

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

impl ObjectParser<'_> for Integer {
    fn parse(buffer: &[Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];

        let (remains, value) = recognize::<_, _, NomError<_>, _>(pair(
            opt(alt((char('-'), (char('+'))))),
            digit1,
        ))(remains)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Integer),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;

        // REFERENCE: [7.3.3 Numeric objects, p 24]
        // Real numbers cannot be used where integers are expected. Hence, the
        // next character should not be '.'
        if char::<_, NomError<_>>('.')(remains).is_ok() {
            return Err(ParseRecoverable::new(
                &buffer[offset..],
                stringify!(Integer),
                ParseErrorCode::WrongObjectType,
            )
            .into());
        }
        // Here, we know that the buffer starts with an integer, and the
        // following errors should be propagated as IntegerFailure

        let len = value.len();
        // It is not guaranteed that the string of digits is a valid i128, e.g.
        // the value could overflow
        // While, initially, the error here seems to be a ParseErr::Failure, it
        // is propagated as ParseErr::Error that some numbers may be too large
        // for i128 yet fit within the f64 range.
        let value = ascii_to_i128(value).ok_or_else(|| {
            ParseRecoverable::new(value, stringify!(Integer), ParseErrorCode::ParseIntError)
        })?;

        let span = Span::new(offset, len);
        Ok(Self { value, span })
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl Deref for Integer {
        type Target = i128;

        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }

    impl Integer {
        pub fn new(value: i128, span: Span) -> Self {
            Self { value, span }
        }

        pub(crate) fn as_u64(&self) -> Option<u64> {
            if let Ok(v) = u64::try_from(self.value) {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_usize(&self) -> Option<usize> {
            if let Ok(v) = usize::try_from(self.value) {
                Some(v)
            } else {
                None
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

    #[test]
    fn numeric_integer_valid() {
        parse_assert_eq!(Integer, b"0", Integer::new(0, Span::new(0, 1)));
        parse_assert_eq!(Integer, b"-0", Integer::new(0, Span::new(0, 2)));
        parse_assert_eq!(Integer, b"+1", Integer::new(1, Span::new(0, 2)));
        parse_assert_eq!(Integer, b"-1", Integer::new(-1, Span::new(0, 2)));
        parse_assert_eq!(Integer, b"1", Integer::new(1, Span::new(0, 1)));
        parse_assert_eq!(
            Integer,
            b"-170141183460469231731687303715884105728<",
            Integer::new(i128::MIN, Span::new(0, 40))
        );
        parse_assert_eq!(
            Integer,
            b"170141183460469231731687303715884105727<",
            Integer::new(i128::MAX, Span::new(0, 39))
        );
        parse_assert_eq!(Integer, b"-1 2", Integer::new(-1, Span::new(0, 2)));
    }

    #[test]
    fn numeric_integer_invalid() {
        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"+", 0);
        let expected_error = ParseRecoverable::new(
            b"",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"- <", 0);
        let expected_error = ParseRecoverable::new(
            b" <",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"+<", 0);
        let expected_error = ParseRecoverable::new(
            b"<",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"<", 0);
        let expected_error = ParseRecoverable::new(
            b"<",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Found real number
        let parse_result = Integer::parse(b"-1.0 ", 0);
        let expected_error = ParseRecoverable::new(
            b"-1.0 ",
            stringify!(Integer),
            ParseErrorCode::WrongObjectType,
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Out of range
        let parse_result = Integer::parse(b"-170141183460469231731687303715884105729<", 0);
        let expected_error = ParseRecoverable::new(
            b"-170141183460469231731687303715884105729",
            stringify!(Integer),
            ParseErrorCode::ParseIntError,
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Out of range
        let parse_result = Integer::parse(b"+170141183460469231731687303715884105728<", 0);
        let expected_error = ParseRecoverable::new(
            b"+170141183460469231731687303715884105728",
            stringify!(Integer),
            ParseErrorCode::ParseIntError,
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
