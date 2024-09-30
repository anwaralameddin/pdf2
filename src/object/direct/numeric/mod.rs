pub(crate) mod integer;
pub(crate) mod real;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

pub(crate) use self::integer::Integer;
pub(crate) use self::real::Real;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
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

impl Parser<'_> for Numeric {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Integer::parse_suppress_recoverable(buffer)
            .or_else(|| Real::parse_suppress_recoverable(buffer))
            .unwrap_or_else(|| {
                Err(ParseRecoverable::new(
                    buffer,
                    stringify!(Numeric),
                    ParseErrorCode::NotFoundUnion,
                )
                .into())
            })
    }
}

mod convert {

    use super::*;
    use crate::impl_from;

    impl_from!(Integer, Integer, Numeric);
    // impl_from!(i128, Integer, Numeric);
    // impl_from!(u64, Integer, Numeric);
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assert_err_eq;
    use crate::parse::error::ParseFailure;
    use crate::parse::Span;
    use crate::parse_assert_eq;
    use crate::parse_span_assert_eq;

    #[test]
    fn numeric_valid() {
        parse_span_assert_eq!(
            b"0",
            Numeric::from(Integer::new(0, Span::new(0, 1))),
            "".as_bytes()
        );
        parse_span_assert_eq!(
            b"-0",
            Numeric::from(Integer::new(0, Span::new(0, 2))),
            "".as_bytes()
        );
        parse_span_assert_eq!(
            b"+1",
            Numeric::from(Integer::new(1, Span::new(0, 2))),
            "".as_bytes()
        );
        parse_span_assert_eq!(
            b"-1",
            Numeric::from(Integer::new(-1, Span::new(0, 2))),
            "".as_bytes()
        );
        parse_span_assert_eq!(
            b"1",
            Numeric::from(Integer::new(1, Span::new(0, 1))),
            "".as_bytes()
        );
        parse_span_assert_eq!(
            b"-170141183460469231731687303715884105728<",
            Numeric::from(Integer::new(i128::MIN, Span::new(0, 39))),
            "<".as_bytes()
        );
        parse_span_assert_eq!(
            b"170141183460469231731687303715884105727<",
            Numeric::from(Integer::new(i128::MAX, Span::new(0, 39))),
            "<".as_bytes()
        );
        parse_span_assert_eq!(
            b"-1 2",
            Numeric::from(Integer::new(-1, Span::new(0, 2))),
            " 2".as_bytes()
        );

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
        let expected_error =
            ParseRecoverable::new(b" <", stringify!(Numeric), ParseErrorCode::NotFoundUnion);
        assert_err_eq!(parse_result, expected_error);

        let parse_result = Numeric::parse(b"+<");
        let expected_error =
            ParseRecoverable::new(b"+<", stringify!(Numeric), ParseErrorCode::NotFoundUnion);
        assert_err_eq!(parse_result, expected_error);

        let parse_result = Numeric::parse(b"+.");
        let expected_error =
            ParseRecoverable::new(b"+.", stringify!(Numeric), ParseErrorCode::NotFoundUnion);
        assert_err_eq!(parse_result, expected_error);

        // TODO(QUESTION) Is there a need to allow such large numbers?
        // Too large for the i128 but within the f64 range
        let buffer = b"-170141183460469231731687303715884105729";
        let parse_result = Numeric::parse(buffer);
        let expected_error =
            ParseFailure::new(buffer, stringify!(Real), ParseErrorCode::ParseFloatError);
        assert_err_eq!(parse_result, expected_error);

        // Too large for the i128 but within the f64 range
        let buffer = b"170141183460469231731687303715884105728";
        let parse_result = Numeric::parse(buffer);
        let expected_error =
            ParseFailure::new(buffer, stringify!(Real), ParseErrorCode::ParseFloatError);
        assert_err_eq!(parse_result, expected_error);
    }
}
