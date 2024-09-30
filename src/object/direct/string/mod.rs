pub(crate) mod hexadecimal;
pub(crate) mod literal;

use ::std::ffi::OsString;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

pub(crate) use self::hexadecimal::Hexadecimal;
pub(crate) use self::literal::Literal;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::Span;
use crate::process::encoding::Encoding;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.4 String objects, p25]
#[derive(Debug, Clone, Copy)]
pub enum String_<'buffer> {
    Hexadecimal(Hexadecimal<'buffer>),
    Literal(Literal<'buffer>),
}

impl Display for String_<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Hexadecimal(hexadecimal) => write!(f, "{}", hexadecimal),
            Self::Literal(literal) => write!(f, "{}", literal),
        }
    }
}

impl PartialEq for String_<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Hexadecimal(self_hex), Self::Hexadecimal(other_hex)) => self_hex == other_hex,
            (Self::Literal(self_lit), Self::Literal(other_lit)) => self_lit == other_lit,
            _ => false,
        }
    }
}

impl<'buffer> Parser<'buffer> for String_<'buffer> {
    fn parse_span(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
        Literal::parse_suppress_recoverable_span(buffer, offset)
            .or_else(|| Hexadecimal::parse_suppress_recoverable_span(buffer, offset))
            .unwrap_or_else(|| {
                Err(ParseRecoverable::new(
                    buffer,
                    stringify!(String),
                    ParseErrorCode::NotFoundUnion,
                )
                .into())
            })
    }

    fn span(&self) -> Span {
        match self {
            Self::Hexadecimal(hexadecimal) => hexadecimal.span(),
            Self::Literal(literal) => literal.span(),
        }
    }
}

mod escape {
    use super::*;
    use crate::process::escape::error::EscapeResult;
    use crate::process::escape::Escape;

    impl Escape for String_<'_> {
        fn escape(&self) -> EscapeResult<Vec<Byte>> {
            match self {
                Self::Hexadecimal(hexadecimal) => hexadecimal.escape(),
                Self::Literal(literal) => literal.escape(),
            }
        }
    }
}

mod encode {

    use super::*;
    use crate::process::encoding::error::EncodingResult;

    impl String_<'_> {
        // pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
        //     let _encoded = encoding.encode(string)?;
        //     todo!("Implement String_::encode")
        //     // We need to choose between hexadecimal and literal string
        //     // Consider introducing the `Escape` trait with the `escape` method
        //     // to have consistent implementations
        // }

        pub(crate) fn decode(&self, _encoding: Encoding) -> EncodingResult<OsString> {
            // encoding.decode(&self.escape()?)
            todo!("Implement String_::decode")
        }
    }
}

mod convert {
    use super::*;
    use crate::impl_from_ref;

    impl_from_ref!('buffer, Hexadecimal<'buffer>, Hexadecimal, String_<'buffer>);
    impl_from_ref!('buffer, Literal<'buffer>, Literal, String_<'buffer>);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Span;

    #[test]
    fn string_valid() {
        // Synthetic tests
        let (buffer, string_literal) = String_::parse_span(b"(A Hexadecimal String)", 0).unwrap();
        assert_eq!(buffer, &[]);
        assert_eq!(
            string_literal,
            Literal::from(("A Hexadecimal String", Span::new(0, 22))).into()
        );

        let (buffer, string_hex) =
            String_::parse_span(b"<412048657861646563696D616C20537472696E67>", 0).unwrap();
        assert_eq!(buffer, &[]);
        assert_eq!(
            string_hex,
            Hexadecimal::from(("412048657861646563696D616C20537472696E67", Span::new(0, 42)))
                .into()
        );

        // TODO The below requires a comparison method for `String_` that compares the escaped values and ignores the span
        // assert_eq!(
        //     string_hex,
        //     Literal::from(("A Hexadecimal String", Span::new(0, 42))).into()
        // );

        // assert_eq!(string_literal, string_hex);
    }
}
