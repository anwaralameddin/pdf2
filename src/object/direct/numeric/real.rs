use ::nom::branch::alt;
use ::nom::character::complete::char;
use ::nom::character::complete::digit0;
use ::nom::character::complete::digit1;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::nom::sequence::pair;
use ::nom::sequence::preceded;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::RealFailure;
use self::error::RealRecoverable;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_f64;
use crate::parse::Parser;
use crate::parse_error;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Real(f64);

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Parser for Real {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        // REFERENCE: [7.3.3 Numeric objects, p24]
        // A real number is represented in its decimal form and does not permit
        // exponential notation.
        // An integer can be used in place of a real number.
        let (buffer, value) = recognize::<_, _, NomError<_>, _>(pair(
            opt(alt((char('-'), (char('+'))))),
            alt((
                recognize(pair(digit1, opt(preceded(char('.'), digit0)))),
                recognize(pair(digit0, preceded(char('.'), digit1))),
            )),
        ))(buffer)
        .map_err(parse_error!(
            e,
            RealRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(e.input),
            }
        ))?;
        // Here, we know that the buffer starts with a real number, and the
        // following errors should be propagated as RealFailure

        // It is not guaranteed that the string of digits and '.' is a valid
        // f64, e.g.  the value could overflow
        let value = ascii_to_f64(value).ok_or_else(|| {
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(value)).into())
        })?;

        let value = Self(value);
        Ok((buffer, value))
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<f64> for Real {
        fn from(value: f64) -> Self {
            Self(value)
        }
    }

    impl Deref for Real {
        type Target = f64;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum RealRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum RealFailure {
        #[error("ParseFloatError: Failed to parse as f64. Input: {0}")]
        ParseFloatError(String),
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse_assert_eq;

    #[test]
    fn numeric_real_valid() {
        parse_assert_eq!(b"-0", Real(0.0), "".as_bytes());
        parse_assert_eq!(b"-0", Real(0.0), "".as_bytes());
        parse_assert_eq!(b"0.0", Real(0.0), "".as_bytes());
        parse_assert_eq!(b"-.0001", Real(-0.0001), "".as_bytes());
        parse_assert_eq!(b"1. 2", Real(1.0), " 2".as_bytes());
        parse_assert_eq!(b"+1 .0 2.0", Real(1.0), " .0 2.0".as_bytes());
        // f64::MIN = -1.7976931348623157E+308f64 but i64::MIN = -9223372036854775808
        parse_assert_eq!(
            b"-9223372036854775808.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999", Real(
                -9223372036854775808.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999
            ), "".as_bytes()
        );
        // f64::MAX = 1.7976931348623157E+308f64 but i64::MAX = 9223372036854775807
        parse_assert_eq!(
            b"9223372036854775807.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999", Real(
                9223372036854775807.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999
            ), "".as_bytes()
        );
    }

    #[test]
    fn numeric_real_invalid() {
        // Real number: Missing digits
        let real_incomplete = Real::parse(b"+.");
        let expected_error = ParseErr::Error(
            RealRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(real_incomplete, expected_error);

        // Real number: Missing digits
        let parse_result = Real::parse(b" <");
        let expected_error = ParseErr::Error(
            RealRecoverable::NotFound {
                code: ErrorKind::Char,
                input: " <".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Real number: Missing digits
        let parse_result = Real::parse(b"+.<");
        let expected_error = ParseErr::Error(
            RealRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO(QUESTION) Is there a need to allow such large numbers?
        // f64::MIN = -1.7976931348623157E+308f64
        let buffer = b"-179769313486231580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let parse_result = Real::parse(buffer);
        let expected_error =
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(buffer)).into());
        assert_err_eq!(parse_result, expected_error);
        // f64::MAX = 1.7976931348623157E+308f64
        let buffer = b"179769313486231580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let parse_result = Real::parse(buffer);
        let expected_error =
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(buffer)).into());
        assert_err_eq!(parse_result, expected_error);
        // f64::NEG_INFINITY
        let buffer = b"-179769313486231589999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        let parse_result = Real::parse(buffer);
        let expected_error =
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(buffer)).into());
        assert_err_eq!(parse_result, expected_error);
        // f64::INFINITY
        let buffer =
            b"179769313486231589999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        let parse_result = Real::parse(buffer);
        let expected_error =
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(buffer)).into());
        assert_err_eq!(parse_result, expected_error);
    }
}
