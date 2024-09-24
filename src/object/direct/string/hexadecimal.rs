use ::nom::bytes::complete::tag;
use ::nom::character::complete::char;
use ::nom::character::complete::hex_digit1;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::nom::multi::many0;
use ::nom::sequence::preceded;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::ffi::OsString;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::fmt::debug_bytes;
use crate::object::BorrowedBuffer;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::process::encoding::Encoding;
use crate::Byte;
use crate::Bytes;

/// REFERENCE: [7.3.4.3 Hexadecimal strings, p27]
#[derive(Clone, Copy)]
pub struct Hexadecimal<'buffer>(&'buffer [Byte]);

#[derive(Clone)]
pub struct OwnedHexadecimal(Bytes);

impl Display for Hexadecimal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<")?;
        for &byte in self.0.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        write!(f, ">")
    }
}

impl Display for OwnedHexadecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&Hexadecimal::from(self), f)
    }
}

impl Debug for Hexadecimal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<{}>", debug_bytes(self.0))
    }
}

impl Debug for OwnedHexadecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&Hexadecimal::from(self), f)
    }
}

impl PartialEq for Hexadecimal<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped
        } else {
            // If an escape call fails, the string is not valid, so we don't need to compare
            false
        }
    }
}

impl PartialEq for OwnedHexadecimal {
    fn eq(&self, other: &Self) -> bool {
        Hexadecimal::from(self) == Hexadecimal::from(other)
    }
}

impl<'buffer> Parser<'buffer> for Hexadecimal<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        // This check is unnecessary because the start of Names and hexadecimal
        // strings are mutually exclusive. However, this allows early return if
        // a dictionary start is found
        let is_dictionary = tag::<_, _, NomError<_>>(b"<<")(buffer);
        if is_dictionary.is_ok() {
            return Err(ParseRecoverable {
                buffer,
                object: stringify!(Hexadecimal),
                code: ParseErrorCode::ObjectType,
            }
            .into());
        }
        // REFERENCE: [7.3.4.3 Hexadecimal strings, p27]
        // White-space characters are allowed and ignored in a hexadecimal
        // string.
        // NOTE: many0 does not result in Failures, so there is no need to
        // handle its errors separately from `char('<')`
        let (buffer, value) = preceded(
            char('<'),
            recognize(preceded(
                opt(white_space_or_comment),
                many0(terminated(hex_digit1, opt(white_space_or_comment))),
            )),
        )(buffer)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable {
                buffer: e.input,
                object: stringify!(Hexadecimal),
                code: ParseErrorCode::NotFound(e.code),
            }
        ))?;
        // Here, we know that the buffer starts with a hexadecimal string, and
        // the following errors should be propagated as HexadecimalFailure
        let (buffer, _) = char::<_, NomError<_>>('>')(buffer).map_err(parse_failure!(
            e,
            ParseFailure {
                buffer: e.input,
                object: stringify!(Hexadecimal),
                code: ParseErrorCode::MissingClosing(e.code),
            }
        ))?;

        let hexadecimal = Self(value);
        Ok((buffer, hexadecimal))
    }
}

impl Parser<'_> for OwnedHexadecimal {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Hexadecimal::parse(buffer)
            .map(|(buffer, hexadecimal)| (buffer, hexadecimal.to_owned_buffer()))
    }
}

mod process {

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::error::ProcessResult;
    use crate::process::filter::ascii_hex::AHx;
    use crate::process::filter::Filter;

    impl Hexadecimal<'_> {
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            let escaped = AHx.defilter(self.0)?;
            Ok(escaped)
        }

        // pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
        //     let encoded = AHx.filter(encoding.encode(string)?)?;
        //     Ok(Self(encoded.into()))
        // }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            encoding.decode(&AHx.defilter(self.0)?)
        }
    }

    impl OwnedHexadecimal {
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            Hexadecimal::from(self).escape()
        }

        pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
            let encoded = AHx.filter(encoding.encode(string)?)?;
            Ok(Self(encoded.into()))
        }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            Hexadecimal::from(self).decode(encoding)
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;
    use crate::object::BorrowedBuffer;

    impl BorrowedBuffer for Hexadecimal<'_> {
        type OwnedBuffer = OwnedHexadecimal;

        fn to_owned_buffer(self) -> Self::OwnedBuffer {
            OwnedHexadecimal(Bytes::from(self.0))
        }
    }

    impl<'buffer> From<&'buffer OwnedHexadecimal> for Hexadecimal<'buffer> {
        fn from(value: &'buffer OwnedHexadecimal) -> Self {
            Hexadecimal(value.0.as_ref())
        }
    }

    impl<'buffer> From<&'buffer str> for Hexadecimal<'buffer> {
        fn from(value: &'buffer str) -> Self {
            Self(value.as_bytes())
        }
    }

    impl From<&str> for OwnedHexadecimal {
        fn from(value: &str) -> Self {
            Hexadecimal::from(value).to_owned_buffer()
        }
    }

    impl<'buffer> Deref for Hexadecimal<'buffer> {
        type Target = &'buffer [Byte];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Deref for OwnedHexadecimal {
        type Target = Bytes;

        fn deref(&self) -> &Self::Target {
            &self.0
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
    fn string_hexadecimal_valid() {
        // Synthetic tests
        parse_assert_eq!(
            b"<41 20 48 65 78 61 64 65 63 69 6D 61 6C 20 53 74 72 69 6E 67>",
            Hexadecimal::from("412048657861646563696D616C20537472696E67"),
            "".as_bytes(),
        );
        parse_assert_eq!(b"<41 2>", Hexadecimal::from("4120"), "".as_bytes(),);
    }

    #[test]
    fn string_hexadecimal_invalid() {
        // Synthetic tests
        // Hexadecimal: Missing closing angle bracket
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67");
        let expected_error = ParseFailure {
            buffer: b"",
            object: stringify!(Hexadecimal),
            code: ParseErrorCode::MissingClosing(ErrorKind::Char),
        };
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Dictionary opening
        let parse_result = Hexadecimal::parse(b"<<412048657861646563696D616C20537472696E67>");
        let expected_error = ParseRecoverable {
            buffer: b"<<412048657861646563696D616C20537472696E67>",
            object: stringify!(Hexadecimal),
            code: ParseErrorCode::ObjectType,
        };
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Not found
        let parse_result = Hexadecimal::parse(b"412048657861646563696D616C20537472696E67>");
        let expected_error = ParseRecoverable {
            buffer: b"412048657861646563696D616C20537472696E67>",
            object: stringify!(Hexadecimal),
            code: ParseErrorCode::NotFound(ErrorKind::Char),
        };
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Missing end angle bracket
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67<");
        let expected_error = ParseFailure {
            buffer: b"<",
            object: stringify!(Hexadecimal),
            code: ParseErrorCode::MissingClosing(ErrorKind::Char),
        };
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Unsupported digits
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67XX>");
        let expected_error = ParseFailure {
            buffer: b"XX>",
            object: stringify!(Hexadecimal),
            code: ParseErrorCode::MissingClosing(ErrorKind::Char),
        };
        assert_err_eq!(parse_result, expected_error);
    }

    #[test]
    fn string_hexadecimal_escape() {
        // Synthetic tests
        let hexadecimal = Hexadecimal::from("412048657861646563696D616C20537472696E67");
        let escaped = hexadecimal.escape().unwrap();
        assert_eq!(escaped, b"A Hexadecimal String");

        let hexadecimal = Hexadecimal::from("412048");
        let escaped = hexadecimal.escape().unwrap();
        assert_eq!(escaped, b"\x41\x20\x48");

        let hexadecimal = Hexadecimal::from("41204");
        let escaped = hexadecimal.escape().unwrap();
        assert_eq!(escaped, b"\x41\x20\x40");

        let hexadecimal_upper = Hexadecimal::from("412048657861646563696D616C20537472696E67");
        let hexadecimal_lower = Hexadecimal::from("412048657861646563696d616c20537472696e67");
        assert_eq!(hexadecimal_upper, hexadecimal_lower);
    }
}
