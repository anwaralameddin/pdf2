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

use self::error::IntegerRecoverable;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_i128;
use crate::parse::Parser;
use crate::parse_error;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Integer(i128);

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Parser for Integer {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, value) = recognize::<_, _, NomError<_>, _>(pair(
            opt(alt((char('-'), (char('+'))))),
            digit1,
        ))(buffer)
        .map_err(parse_error!(
            e,
            IntegerRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(e.input),
            }
        ))?;

        // REFERENCE: [7.3.3 Numeric objects, p 24]
        // Real numbers cannot be used where integers are expected. Hence, the
        // next character should not be '.'
        if char::<_, NomError<_>>('.')(buffer).is_ok() {
            return Err(ParseErr::Error(
                IntegerRecoverable::FoundReal(debug_bytes(value), debug_bytes(buffer)).into(),
            ));
        }
        // Here, we know that the buffer starts with an integer, and the
        // following errors should be propagated as IntegerFailure

        // It is not guaranteed that the string of digits is a valid i128, e.g.
        // the value could overflow
        // While, initially, the error here seems to be a ParseErr::Failure, it
        // is propagated as ParseErr::Error that some numbers may be too large
        // for i128 yet fit within the f64 range.
        let value = ascii_to_i128(value).ok_or_else(|| {
            ParseErr::Error(IntegerRecoverable::ParseIntError(debug_bytes(value)).into())
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

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IntegerRecoverable {
        #[error("ParseIntError: Failed to parse as i128. Input: {0}")]
        ParseIntError(String),
        #[error("Found Real: {0} is part of a real number with a decimal part {1}")]
        FoundReal(String, String),
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
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
        let expected_error = ParseErr::Error(
            IntegerRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"- <");
        let expected_error = ParseErr::Error(
            IntegerRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: " <".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"+<");
        let expected_error = ParseErr::Error(
            IntegerRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Missing digits
        let parse_result = Integer::parse(b"<");
        let expected_error = ParseErr::Error(
            IntegerRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Found real number
        let parse_result = Integer::parse(b"-1.0 ");
        let expected_error = ParseErr::Error(
            IntegerRecoverable::FoundReal("-1".to_string(), ".0 ".to_string()).into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Out of range
        let parse_result = Integer::parse(b"-170141183460469231731687303715884105729<");
        let expected_error = ParseErr::Error(
            IntegerRecoverable::ParseIntError(
                "-170141183460469231731687303715884105729".to_string(),
            )
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Invalid integer number: Out of range
        let parse_result = Integer::parse(b"+170141183460469231731687303715884105728<");
        let expected_error = ParseErr::Error(
            IntegerRecoverable::ParseIntError(
                "+170141183460469231731687303715884105728".to_string(),
            )
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
