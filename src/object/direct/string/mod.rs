pub(crate) mod hexadecimal;
pub(crate) mod literal;

use ::std::ffi::OsString;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::StringRecoverable;
pub(crate) use self::hexadecimal::Hexadecimal;
pub(crate) use self::literal::Literal;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::process::encoding::Encoding;
use crate::Byte;

/// REFERENCE: [7.3.4 String objects, p25]
#[derive(Debug, Clone)]
pub enum String_ {
    Hexadecimal(Hexadecimal),
    Literal(Literal),
}

impl Display for String_ {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Hexadecimal(hexadecimal) => write!(f, "{}", hexadecimal),
            Self::Literal(literal) => write!(f, "{}", literal),
        }
    }
}

impl PartialEq for String_ {
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

impl Parser for String_ {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Literal::parse_semi_quiet(buffer)
            .or_else(|| Hexadecimal::parse_semi_quiet(buffer))
            .unwrap_or_else(|| {
                Err(ParseErr::Error(
                    StringRecoverable::NotFound(debug_bytes(buffer)).into(),
                ))
            })
    }
}

mod process {

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::error::ProcessResult;

    impl String_ {
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

    impl_from!(Hexadecimal, Hexadecimal, String_);
    impl_from!(Literal, Literal, String_);

    impl String_ {
        pub(crate) fn as_hexadecimal(&self) -> Option<&Hexadecimal> {
            if let Self::Hexadecimal(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub(crate) fn as_literal(&self) -> Option<&Literal> {
            if let Self::Literal(v) = self {
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
    pub enum StringRecoverable {
        #[error("Not found. Input: {0}")]
        NotFound(String),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_valid() {
        // Synthetic tests
        let (buffer, string_literal) = String_::parse(b"(A Hexadecimal String)").unwrap();
        assert_eq!(buffer, &[]);
        assert_eq!(string_literal, Literal::from("A Hexadecimal String").into());

        let (buffer, string_hex) =
            String_::parse(b"<412048657861646563696D616C20537472696E67>").unwrap();
        assert_eq!(buffer, &[]);
        assert_eq!(string_hex, Literal::from("A Hexadecimal String").into());

        assert_eq!(string_literal, string_hex);
    }
}
