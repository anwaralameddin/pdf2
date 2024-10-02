use ::nom::character::complete::char;
use ::nom::character::complete::hex_digit1;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::nom::multi::many0;
use ::nom::sequence::preceded;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::process::encoding::Encoding;
use crate::process::escape::Escape;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.4.3 Hexadecimal strings, p27]
#[derive(Debug, Clone, Copy)]
pub struct Hexadecimal<'buffer> {
    value: &'buffer [Byte],
    span: Span,
}

impl Display for Hexadecimal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<")?;
        for &byte in self.value.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        write!(f, ">")
    }
}

impl PartialEq for Hexadecimal<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped && self.span == other.span
        } else {
            // If an escape call fails, the string is not valid, so we don't need to compare
            false
        }
    }
}

impl<'buffer> ObjectParser<'buffer> for Hexadecimal<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];

        // REFERENCE: [7.3.4.3 Hexadecimal strings, p27]
        // White-space characters are allowed and ignored in a hexadecimal
        // string.
        // NOTE: many0 does not result in Failures, so there is no need to
        // handle its errors separately from `char('<')`
        let (remains, value) = preceded(
            char('<'),
            recognize(preceded(
                opt(white_space_or_comment),
                many0(terminated(hex_digit1, opt(white_space_or_comment))),
            )),
        )(remains)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Hexadecimal),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        // Here, we know that the buffer starts with a hexadecimal string, and
        // the following errors should be propagated as HexadecimalFailure
        char::<_, NomError<_>>('>')(remains).map_err(parse_failure!(
            e,
            ParseFailure::new(
                e.input,
                stringify!(Hexadecimal),
                ParseErrorCode::MissingClosing(e.code)
            )
        ))?;

        let len = value.len() + 2;
        let span = Span::new(offset, len);
        Ok(Self { value, span })
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod escape {

    use super::*;
    use crate::process::escape::error::EscapeErr;
    use crate::process::escape::error::EscapeErrorCode;
    use crate::process::escape::error::EscapeResult;
    use crate::process::escape::Escape;
    use crate::process::filter::ascii_hex::AHx;
    use crate::process::filter::Filter;

    impl Escape for Hexadecimal<'_> {
        fn escape(&self) -> EscapeResult<Vec<Byte>> {
            let escaped = AHx.defilter(self.value).map_err(|err| {
                EscapeErr::new(self.value, EscapeErrorCode::Hexadecimal(err.code))
            })?;
            Ok(escaped)
        }
    }
}

mod encode {
    use ::std::ffi::OsString;

    use super::*;
    use crate::process::encoding::error::EncodingErr;
    use crate::process::encoding::error::EncodingErrorCode;
    use crate::process::encoding::error::EncodingResult;
    use crate::process::encoding::Decoder;
    use crate::process::filter::ascii_hex::AHx;
    use crate::process::filter::Filter;

    impl Hexadecimal<'_> {
        // pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
        // let encoded = AHx.filter(encoding.encode(string)?)?;
        // Ok(Self(encoded.into()))
        // }

        pub(crate) fn decode(&self, encoding: Encoding) -> EncodingResult<OsString> {
            encoding.decode(
                |data| {
                    AHx.defilter(data)
                        .map_err(|err| EncodingErr::new(data, EncodingErrorCode::Filter(err)))
                },
                self.value,
            )
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl<'buffer> Hexadecimal<'buffer> {
        pub fn new(value: &'buffer [Byte], span: Span) -> Self {
            Self { value, span }
        }
    }

    impl<'buffer> From<(&'buffer str, Span)> for Hexadecimal<'buffer> {
        fn from((value, span): (&'buffer str, Span)) -> Self {
            Self::new(value.as_bytes(), span)
        }
    }

    impl<'buffer> Deref for Hexadecimal<'buffer> {
        type Target = &'buffer [Byte];

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
    use crate::process::escape::Escape;

    #[test]
    fn string_hexadecimal_valid() {
        // Synthetic tests
        parse_assert_eq!(
            Hexadecimal,
            b"<41 20 48 65 78 61 64 65 63 69 6D 61 6C 20 53 74 72 69 6E 67>",
            Hexadecimal::from(("412048657861646563696D616C20537472696E67", Span::new(0, 61))),
        );
        parse_assert_eq!(
            Hexadecimal,
            b"<41 2>",
            Hexadecimal::from(("4120", Span::new(0, 6))),
        );
    }

    #[test]
    fn string_hexadecimal_invalid() {
        // Synthetic tests
        // Hexadecimal: Missing closing angle bracket
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67", 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Hexadecimal),
            ParseErrorCode::MissingClosing(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Dictionary opening
        let parse_result = Hexadecimal::parse(b"<<412048657861646563696D616C20537472696E67>", 0);
        let expected_error = ParseFailure::new(
            b"<412048657861646563696D616C20537472696E67>",
            stringify!(Hexadecimal),
            ParseErrorCode::MissingClosing(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Not found
        let parse_result = Hexadecimal::parse(b"412048657861646563696D616C20537472696E67>", 0);
        let expected_error = ParseRecoverable::new(
            b"412048657861646563696D616C20537472696E67>",
            stringify!(Hexadecimal),
            ParseErrorCode::NotFound(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Missing end angle bracket
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67<", 0);
        let expected_error = ParseFailure::new(
            b"<",
            stringify!(Hexadecimal),
            ParseErrorCode::MissingClosing(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Unsupported digits
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67XX>", 0);
        let expected_error = ParseFailure::new(
            b"XX>",
            stringify!(Hexadecimal),
            ParseErrorCode::MissingClosing(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);
    }

    #[test]
    fn string_hexadecimal_escape() {
        // Synthetic tests
        let hexadecimal =
            Hexadecimal::from(("412048657861646563696D616C20537472696E67", Span::new(0, 61)));
        let escaped = hexadecimal.escape().unwrap();
        assert_eq!(escaped, b"A Hexadecimal String");

        let hexadecimal = Hexadecimal::from(("412048", Span::new(0, 6)));
        let escaped = hexadecimal.escape().unwrap();
        assert_eq!(escaped, b"\x41\x20\x48");

        let hexadecimal = Hexadecimal::from(("41204", Span::new(0, 5)));
        let escaped = hexadecimal.escape().unwrap();
        assert_eq!(escaped, b"\x41\x20\x40");

        let hexadecimal_upper =
            Hexadecimal::from(("412048657861646563696D616C20537472696E67", Span::new(0, 61)));
        let hexadecimal_lower =
            Hexadecimal::from(("412048657861646563696d616c20537472696e67", Span::new(0, 61)));
        assert_eq!(hexadecimal_upper, hexadecimal_lower);
    }
}
