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
use crate::parse::Parser;
use crate::parse_recoverable;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Integer(i128);

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Parser<'_> for Integer {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (remains, value) = recognize::<_, _, NomError<_>, _>(pair(
            opt(alt((char('-'), (char('+'))))),
            digit1,
        ))(buffer)
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
                buffer,
                stringify!(Integer),
                ParseErrorCode::WrongObjectType,
            )
            .into());
        }
        let buffer = remains;
        // Here, we know that the buffer starts with an integer, and the
        // following errors should be propagated as IntegerFailure

        // It is not guaranteed that the string of digits is a valid i128, e.g.
        // the value could overflow
        // While, initially, the error here seems to be a ParseErr::Failure, it
        // is propagated as ParseErr::Error that some numbers may be too large
        // for i128 yet fit within the f64 range.
        let value = ascii_to_i128(value).ok_or_else(|| {
            ParseRecoverable::new(value, stringify!(Integer), ParseErrorCode::ParseIntError)
        })?;

        let value = Self(value);

        Ok((buffer, value))
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<i128> for Integer {
        fn from(value: i128) -> Self {
            Self(value)
        }
    }

    impl Deref for Integer {
        type Target = i128;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<u64> for Integer {
        fn from(value: u64) -> Self {
            Self(value.into())
        }
    }

    impl Integer {
        pub(crate) fn as_u64(&self) -> Option<u64> {
            if let Ok(v) = u64::try_from(self.0) {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_usize(&self) -> Option<usize> {
            if let Ok(v) = usize::try_from(self.0) {
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
        parse_assert_eq!(b"0", Integer(0), "".as_bytes());
        parse_assert_eq!(b"-0", Integer(-0), "".as_bytes());
        parse_assert_eq!(b"+1", Integer(1), "".as_bytes());
        parse_assert_eq!(b"-1", Integer(-1), "".as_bytes());
        parse_assert_eq!(b"1", Integer(1), "".as_bytes());
        parse_assert_eq!(
            b"-170141183460469231731687303715884105728<",
            Integer(i128::MIN),
            "<".as_bytes()
        );
        parse_assert_eq!(
            b"170141183460469231731687303715884105727<",
            Integer(i128::MAX),
            "<".as_bytes()
        );
        parse_assert_eq!(b"-1 2", Integer(-1), " 2".as_bytes());
    }

    #[test]
    fn numeric_integer_invalid() {
        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"+");
        let expected_error = ParseRecoverable::new(
            b"",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"- <");
        let expected_error = ParseRecoverable::new(
            b" <",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"+<");
        let expected_error = ParseRecoverable::new(
            b"<",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"<");
        let expected_error = ParseRecoverable::new(
            b"<",
            stringify!(Integer),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Found real number
        let parse_result = Integer::parse(b"-1.0 ");
        let expected_error = ParseRecoverable::new(
            b"-1.0 ",
            stringify!(Integer),
            ParseErrorCode::WrongObjectType,
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Out of range
        let parse_result = Integer::parse(b"-170141183460469231731687303715884105729<");
        let expected_error = ParseRecoverable::new(
            b"-170141183460469231731687303715884105729",
            stringify!(Integer),
            ParseErrorCode::ParseIntError,
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Out of range
        let parse_result = Integer::parse(b"+170141183460469231731687303715884105728<");
        let expected_error = ParseRecoverable::new(
            b"+170141183460469231731687303715884105728",
            stringify!(Integer),
            ParseErrorCode::ParseIntError,
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
