pub(crate) mod hexadecimal;
pub(crate) mod literal;

use ::std::ffi::OsString;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

pub(crate) use self::hexadecimal::Hexadecimal;
pub(crate) use self::hexadecimal::OwnedHexadecimal;
pub(crate) use self::literal::Literal;
pub(crate) use self::literal::OwnedLiteral;
use crate::object::BorrowedBuffer;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::process::encoding::Encoding;
use crate::Byte;

/// REFERENCE: [7.3.4 String objects, p25]
#[derive(Debug, Clone, Copy)]
pub enum String_<'buffer> {
    Hexadecimal(Hexadecimal<'buffer>),
    Literal(Literal<'buffer>),
}

#[derive(Debug, Clone)]
pub enum OwnedString {
    Hexadecimal(OwnedHexadecimal),
    Literal(OwnedLiteral),
}

impl Display for String_<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Hexadecimal(hexadecimal) => write!(f, "{}", hexadecimal),
            Self::Literal(literal) => write!(f, "{}", literal),
        }
    }
}

impl Display for OwnedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&String_::from(self), f)
    }
}

impl PartialEq for String_<'_> {
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

impl PartialEq for OwnedString {
    fn eq(&self, other: &Self) -> bool {
        String_::from(self) == String_::from(other)
    }
}

impl<'buffer> Parser<'buffer> for String_<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        Literal::parse_suppress_recoverable(buffer)
            .or_else(|| Hexadecimal::parse_suppress_recoverable(buffer))
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

impl Parser<'_> for OwnedString {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        String_::parse(buffer).map(|(buffer, string)| (buffer, string.to_owned_buffer()))
    }
}

mod process {

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::error::ProcessResult;

    impl String_<'_> {
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            match self {
                Self::Hexadecimal(hexadecimal) => hexadecimal.escape(),
                Self::Literal(literal) => literal.escape(),
            }
        }

        // pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
        //     let _encoded = encoding.encode(string)?;
        //     todo!("Implement String_::encode")
        //     // We need to choose between hexadecimal and literal string
        //     // Consider introducing the `Escape` trait with the `escape` method
        //     // to have consistent implementations
        // }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            encoding.decode(&self.escape()?)
        }
    }

    impl OwnedString {
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            String_::from(self).escape()
        }

        pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
            let _encoded = encoding.encode(string)?;
            todo!("Implement String_::encode")
            // We need to choose between hexadecimal and literal string
            // Consider introducing the `Escape` trait with the `escape` method
            // to have consistent implementations
        }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            String_::from(self).decode(encoding)
        }
    }
}

mod convert {
    use super::*;
    use crate::impl_from;
    use crate::impl_from_ref;
    use crate::object::BorrowedBuffer;

    impl BorrowedBuffer for String_<'_> {
        type OwnedBuffer = OwnedString;

        fn to_owned_buffer(self) -> Self::OwnedBuffer {
            match self {
                Self::Hexadecimal(hexadecimal) => {
                    Self::OwnedBuffer::Hexadecimal(hexadecimal.to_owned_buffer())
                }
                Self::Literal(literal) => Self::OwnedBuffer::Literal(literal.to_owned_buffer()),
            }
        }
    }

    impl<'buffer> From<&'buffer OwnedString> for String_<'buffer> {
        fn from(value: &'buffer OwnedString) -> Self {
            match value {
                OwnedString::Hexadecimal(hexadecimal) => Self::Hexadecimal(hexadecimal.into()),
                OwnedString::Literal(literal) => Self::Literal(literal.into()),
            }
        }
    }

    impl_from_ref!('buffer, Hexadecimal<'buffer>, Hexadecimal, String_<'buffer>);
    impl_from_ref!('buffer, Literal<'buffer>, Literal, String_<'buffer>);
    impl_from!(OwnedHexadecimal, Hexadecimal, OwnedString);
    impl_from!(OwnedLiteral, Literal, OwnedString);

    impl String_<'_> {
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
