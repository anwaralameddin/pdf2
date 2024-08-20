pub(crate) mod integer;
pub(crate) mod real;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::NumericRecoverable;
pub(crate) use self::integer::Integer;
pub(crate) use self::real::Real;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

/// REFERENCE: [7.3.3 Numeric objects, p24]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Numeric {
    Integer(Integer),
    Real(Real),
}

impl Display for Numeric {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Integer(n) => write!(f, "{}", n),
            Self::Real(r) => write!(f, "{}", r),
        }
    }
}

impl Parser for Numeric {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Integer::parse_semi_quiet(buffer)
            .or_else(|| Real::parse_semi_quiet(buffer))
            .unwrap_or_else(|| {
                Err(ParseErr::Error(
                    NumericRecoverable::NotFound(debug_bytes(buffer)).into(),
                ))
            })
    }
}

mod convert {

    use super::*;
    use crate::impl_from;

    impl_from!(Integer, Integer, Numeric);
    impl_from!(i128, Integer, Numeric);
    impl_from!(u64, Integer, Numeric);
    impl_from!(Real, Real, Numeric);
    impl_from!(f64, Real, Numeric);

    impl Numeric {
        pub(crate) fn as_integer(&self) -> Option<&Integer> {
            if let Self::Integer(v) = self {
                Some(v)
            } else {
                None
            }
        }
    }
}

pub(crate) mod error {

    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum NumericRecoverable {
        #[error("Not found. Input: {0}")]
        NotFound(String),
    }
}

#[cfg(test)]
mod tests {

    use super::real::error::RealFailure;
    use super::*;
    use crate::assert_err_eq;
    use crate::parse_assert_eq;

    #[test]
    fn numeric_valid() {
        parse_assert_eq!(b"0", Numeric::from(0i128), "".as_bytes());
        parse_assert_eq!(b"-0", Numeric::from(-0i128), "".as_bytes());
        parse_assert_eq!(b"+1", Numeric::from(1i128), "".as_bytes());
        parse_assert_eq!(b"-1", Numeric::from(-1i128), "".as_bytes());
        parse_assert_eq!(b"1", Numeric::from(1i128), "".as_bytes());
        parse_assert_eq!(
            b"-170141183460469231731687303715884105728<",
            Numeric::from(i128::MIN),
            "<".as_bytes()
        );
        parse_assert_eq!(
            b"170141183460469231731687303715884105727<",
            Numeric::from(i128::MAX),
            "<".as_bytes()
        );
        parse_assert_eq!(b"-1 2", Numeric::from(-1i128), " 2".as_bytes());

        parse_assert_eq!(b"0.0", Numeric::from(0.0), "".as_bytes());
        parse_assert_eq!(b"-0.0", Numeric::from(0.0), "".as_bytes());
        parse_assert_eq!(b"-.0001", Numeric::from(-0.0001), "".as_bytes());
        parse_assert_eq!(b"1. 2", Numeric::from(1.0), " 2".as_bytes());
        parse_assert_eq!(b"+1. .0 2.0", Numeric::from(1.0), " .0 2.0".as_bytes());
        // f64::MIN = -1.7976931348623157E+308f64 but i64::MIN = -9223372036854775808
        parse_assert_eq!(
            b"-9223372036854775808.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999", Numeric::from(
                -9223372036854775808.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999
            ), "".as_bytes()
        );
        // f64::MAX = 1.7976931348623157E+308f64 but i64::MAX = 9223372036854775807
        parse_assert_eq!(
            b"9223372036854775807.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999", Numeric::from(
                9223372036854775807.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999
            ), "".as_bytes()
        );
    }

    #[test]
    fn numeric_invalid() {
        let parse_result = Numeric::parse(b" <");
        let expected_error = ParseErr::Error(NumericRecoverable::NotFound(" <".to_string()).into());
        assert_err_eq!(parse_result, expected_error);

        let parse_result = Numeric::parse(b"+<");
        let expected_error = ParseErr::Error(NumericRecoverable::NotFound("+<".to_string()).into());
        assert_err_eq!(parse_result, expected_error);

        let parse_result = Numeric::parse(b"+.");
        let expected_error = ParseErr::Error(NumericRecoverable::NotFound("+.".to_string()).into());
        assert_err_eq!(parse_result, expected_error);

        // TODO(QUESTION) Is there a need to allow such large numbers?
        // Too large for the i128 but within the f64 range
        let buffer = b"-170141183460469231731687303715884105729";
        let parse_result = Numeric::parse(buffer);
        let expected_error =
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(buffer)).into());
        assert_err_eq!(parse_result, expected_error);

        // Too large for the i128 but within the f64 range
        let buffer = b"170141183460469231731687303715884105728";
        let parse_result = Numeric::parse(buffer);
        let expected_error =
            ParseErr::Failure(RealFailure::ParseFloatError(debug_bytes(buffer)).into());
        assert_err_eq!(parse_result, expected_error);
    }
}
