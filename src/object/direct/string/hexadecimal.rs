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

use self::error::HexadecimalFailure;
use self::error::HexadecimalRecoverable;
use crate::fmt::debug_bytes;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_error;
use crate::parse_failure;
use crate::process::encoding::Encoding;
use crate::Byte;
use crate::Bytes;

/// REFERENCE: [7.3.4.3 Hexadecimal strings, p27]
#[derive(Clone)]
pub struct Hexadecimal(Bytes);

impl Display for Hexadecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<")?;
        for &byte in self.0.iter() {
            write!(f, "{}", byte as char)?;
        }
        write!(f, ">")
    }
}

impl Debug for Hexadecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<{}>", debug_bytes(&self.0))
    }
}

impl PartialEq for Hexadecimal {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped
        } else {
            // If an escape call fails, the string is not valid, so we don't need to compare
            false
        }
    }
}

impl Parser for Hexadecimal {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        // This check is unnecessary because the start of Names and hexadecimal
        // strings are mutually exclusive. However, this allows early return if
        // a dictionary start is found
        let is_dictionary = tag::<_, _, NomError<_>>(b"<<")(buffer);
        if is_dictionary.is_ok() {
            return Err(ParseErr::Error(
                HexadecimalRecoverable::DictionaryOpening(debug_bytes(buffer)).into(),
            ));
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
        .map_err(parse_error!(
            e,
            HexadecimalRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(buffer)
            }
        ))?;
        // Here, we know that the buffer starts with a hexadecimal string, and
        // the following errors should be propagated as HexadecimalFailure
        let (buffer, _) = char::<_, NomError<_>>('>')(buffer).map_err(parse_failure!(
            e,
            HexadecimalFailure::MissingClosing {
                code: e.code,
                input: debug_bytes(buffer)
            }
        ))?;

        let hexadecimal = Self(value.into());
        Ok((buffer, hexadecimal))
    }
}

mod process {

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::error::ProcessResult;
    use crate::process::filter::ascii_hex::AHx;
    use crate::process::filter::Filter;
    impl Hexadecimal {
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            let escaped = AHx.defilter(&*self.0)?;
            Ok(escaped)
        }

        pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
            let encoded = AHx.filter(encoding.encode(string)?)?;
            Ok(Self(encoded.into()))
        }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            encoding.decode(&AHx.defilter(&*self.0)?)
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<&str> for Hexadecimal {
        fn from(value: &str) -> Self {
            Self(value.as_bytes().into())
        }
    }

    impl Deref for Hexadecimal {
        type Target = Bytes;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum HexadecimalRecoverable {
        #[error("Dictionary Found. Input: {0}")]
        DictionaryOpening(String),
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum HexadecimalFailure {
        #[error("Missing Clossing: {code:?}. Input: {input}")]
        MissingClosing { code: ErrorKind, input: String },
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
        let expected_error = ParseErr::Failure(
            HexadecimalFailure::MissingClosing {
                code: ErrorKind::Char,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Dictionary opening
        let parse_result = Hexadecimal::parse(b"<<412048657861646563696D616C20537472696E67>");
        let expected_error = ParseErr::Error(
            HexadecimalRecoverable::DictionaryOpening(
                "<<412048657861646563696D616C20537472696E67>".to_string(),
            )
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Not found
        let parse_result = Hexadecimal::parse(b"412048657861646563696D616C20537472696E67>");
        let expected_error = ParseErr::Error(
            HexadecimalRecoverable::NotFound {
                code: ErrorKind::Char,
                input: "412048657861646563696D616C20537472696E67>".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Missing end angle bracket
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67<");
        let expected_error = ParseErr::Failure(
            HexadecimalFailure::MissingClosing {
                code: ErrorKind::Char,
                input: "<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Hexadecimal: Unsupported digits
        let parse_result = Hexadecimal::parse(b"<412048657861646563696D616C20537472696E67XX>");
        let expected_error = ParseErr::Failure(
            HexadecimalFailure::MissingClosing {
                code: ErrorKind::Char,
                input: "XX>".to_string(),
            }
            .into(),
        );
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
