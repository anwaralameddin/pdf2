pub(crate) mod hexadecimal;
pub(crate) mod literal;

use ::std::ffi::OsString;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

pub(crate) use self::hexadecimal::OwnedHexadecimal;
pub(crate) use self::literal::OwnedLiteral;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::process::encoding::Encoding;
use crate::Byte;

/// REFERENCE: [7.3.4 String objects, p25]
#[derive(Debug, Clone)]
pub enum OwnedString {
    Hexadecimal(OwnedHexadecimal),
    Literal(OwnedLiteral),
}

impl Display for OwnedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Hexadecimal(hexadecimal) => write!(f, "{}", hexadecimal),
            Self::Literal(literal) => write!(f, "{}", literal),
        }
    }
}

impl PartialEq for OwnedString {
    fn eq(&self, other: &Self) -> bool {
        if let (Self::Hexadecimal(self_hex), Self::Hexadecimal(other_hex)) = (self, other) {
            self_hex == other_hex
        } else if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped
        } else {
            // If an escape call fails, the string is not valid, so we don't need to compare
            false
        }
    }
}

impl Parser<'_> for OwnedString {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        OwnedLiteral::parse_suppress_recoverable(buffer)
            .or_else(|| OwnedHexadecimal::parse_suppress_recoverable(buffer))
            .unwrap_or_else(|| {
                Err(ParseRecoverable {
                    buffer,
                    object: stringify!(String),
                    code: ParseErrorCode::NotFoundUnion,
                }
                .into())
            })
    }
}

mod process {

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::error::ProcessResult;

    impl OwnedString {
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            match self {
                Self::Hexadecimal(hexadecimal) => hexadecimal.escape(),
                Self::Literal(literal) => literal.escape(),
            }
        }

        pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
            let _encoded = encoding.encode(string)?;
            todo!("Implement String_::encode")
            // We need to choose between hexadecimal and literal string
            // Consider introducing the `Escape` trait with the `escape` method
            // to have consistent implementations
        }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            encoding.decode(&self.escape()?)
        }
    }
}

mod convert {
    use super::*;
    use crate::impl_from;

    impl_from!(OwnedHexadecimal, Hexadecimal, OwnedString);
    impl_from!(OwnedLiteral, Literal, OwnedString);

    impl OwnedString {
        pub(crate) fn as_hexadecimal(&self) -> Option<&OwnedHexadecimal> {
            if let Self::Hexadecimal(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_literal(&self) -> Option<&OwnedLiteral> {
            if let Self::Literal(v) = self {
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

    #[test]
    fn string_valid() {
        // Synthetic tests
        let (buffer, string_literal) = OwnedString::parse(b"(A Hexadecimal String)").unwrap();
        assert_eq!(buffer, &[]);
        assert_eq!(
            string_literal,
            OwnedLiteral::from("A Hexadecimal String").into()
        );

        let (buffer, string_hex) =
            OwnedString::parse(b"<412048657861646563696D616C20537472696E67>").unwrap();
        assert_eq!(buffer, &[]);
        assert_eq!(
            string_hex,
            OwnedLiteral::from("A Hexadecimal String").into()
        );

        assert_eq!(string_literal, string_hex);
    }
}
