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

use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_f64;
use crate::parse::Parser;
use crate::parse::Span;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Real {
    value: f64,
    span: Span,
}

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

impl Parser<'_> for Real {
    fn parse_span(buffer: &[Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
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
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(e.input, stringify!(Real), ParseErrorCode::NotFound(e.code))
        ))?;
        // Here, we know that the buffer starts with a real number, and the
        // following errors should be propagated as RealFailure

        let len = value.len();
        // It is not guaranteed that the string of digits and '.' is a valid
        // f64, e.g.  the value could overflow
        let value = ascii_to_f64(value).ok_or_else(|| {
            ParseFailure::new(value, stringify!(Real), ParseErrorCode::ParseFloatError)
        })?;

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

    impl Real {
        pub fn new(value: f64, span: Span) -> Self {
            Self { value, span }
        }
    }

    impl Deref for Real {
        type Target = f64;

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
    fn numeric_real_valid() {
        parse_span_assert_eq!(b"-0", Real::new(0.0, Span::new(0, 2)), "".as_bytes());
        parse_span_assert_eq!(b"-0", Real::new(0.0, Span::new(0, 2)), "".as_bytes());
        parse_span_assert_eq!(b"0.0", Real::new(0.0, Span::new(0, 3)), "".as_bytes());
        parse_span_assert_eq!(
            b"-.0001",
            Real::new(-0.0001, Span::new(0, 6)),
            "".as_bytes()
        );
        parse_span_assert_eq!(b"1. 2", Real::new(1.0, Span::new(0, 2)), " 2".as_bytes());
        parse_span_assert_eq!(
            b"+1 .0 2.0",
            Real::new(1.0, Span::new(0, 2)),
            " .0 2.0".as_bytes()
        );
        // f64::MIN = -1.7976931348623157E+308f64 but i64::MIN = -9223372036854775808
        parse_span_assert_eq!(
            b"-9223372036854775808.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999",
            Real::new(
                -9223372036854775808.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999,
                Span::new(0, 313)
            ),
            "".as_bytes()
        );
        // f64::MAX = 1.7976931348623157E+308f64 but i64::MAX = 9223372036854775807
        parse_span_assert_eq!(
            b"9223372036854775807.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999",
            Real::new(
                9223372036854775807.9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999,
                Span::new(0, 312)
            ), 
            "".as_bytes()
        );
    }

    #[test]
    fn numeric_real_invalid() {
        // Real number: Missing digits
        let real_incomplete = Real::parse_span(b"+.", 0);
        let expected_error = ParseRecoverable::new(
            b"",
            stringify!(Real),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(real_incomplete, expected_error);

        // Real number: Missing digits
        let parse_result = Real::parse_span(b" <", 0);
        let expected_error = ParseRecoverable::new(
            b" <",
            stringify!(Real),
            ParseErrorCode::NotFound(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Real number: Missing digits
        let parse_result = Real::parse_span(b"+.<", 0);
        let expected_error = ParseRecoverable::new(
            b"<",
            stringify!(Real),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO(QUESTION) Is there a need to allow such large numbers?
        // f64::MIN = -1.7976931348623157E+308f64
        let buffer = b"-179769313486231580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let parse_result = Real::parse_span(buffer, 0);
        let expected_error =
            ParseFailure::new(buffer, stringify!(Real), ParseErrorCode::ParseFloatError);
        assert_err_eq!(parse_result, expected_error);
        // f64::MAX = 1.7976931348623157E+308f64
        let buffer = b"179769313486231580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let parse_result = Real::parse_span(buffer, 0);
        let expected_error =
            ParseFailure::new(buffer, stringify!(Real), ParseErrorCode::ParseFloatError);
        assert_err_eq!(parse_result, expected_error);
        // f64::NEG_INFINITY
        let buffer = b"-179769313486231589999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        let parse_result = Real::parse_span(buffer, 0); 
        let expected_error =
            ParseFailure::new(buffer, stringify!(Real), ParseErrorCode::ParseFloatError);
        assert_err_eq!(parse_result, expected_error);
        // f64::INFINITY
        let buffer =
            b"179769313486231589999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        let parse_result = Real::parse_span(buffer, 0);
        let expected_error =
            ParseFailure::new(buffer, stringify!(Real), ParseErrorCode::ParseFloatError);
        assert_err_eq!(parse_result, expected_error);
    }
}
