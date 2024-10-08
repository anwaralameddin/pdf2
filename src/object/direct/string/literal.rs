use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take;
use ::nom::bytes::complete::take_while1;
use ::nom::character::complete::char;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::nom::multi::many0;
use ::nom::sequence::delimited;
use ::nom::sequence::pair;
use ::nom::sequence::preceded;
use ::nom::Err as NomErr;
use ::nom::IResult;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::process::escape::Escape;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.4.2 Literal strings, p25-28}
#[derive(Debug, Clone, Copy)]
pub struct Literal<'buffer> {
    value: &'buffer [Byte],
    span: Span,
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "(")?;
        for &byte in self.value.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        write!(f, ")")
    }
}

impl PartialEq for Literal<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped && self.span == other.span
        } else {
            // If an escape call fails, the string is not valid, so we don't need to compare
            false
        }
    }
}

impl<'buffer> ObjectParser<'buffer> for Literal<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];
        let start = offset;

        // NOTE: many0 does not result in Failures, so there is no need to
        // handle its errors separately from `char('<')`
        let (remains, value) = preceded(
            char('('),
            recognize(pair(
                parse::not_parentheses,
                many0(pair(parse::inner_parentheses, parse::not_parentheses)),
            )),
        )(remains)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Literal),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        // Here, we know that the buffer starts with a literal string, and
        // the following errors should be propagated as LiteralFailure
        char::<_, NomError<_>>(')')(remains).map_err(parse_failure!(
            e,
            ParseFailure::new(
                e.input,
                stringify!(Literal),
                ParseErrorCode::MissingClosing(e.code)
            )
        ))?;
        let offset = offset + value.len() + 2;

        let span = Span::new(start, offset);
        Ok(Self { value, span })
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod parse {
    use super::*;
    use crate::Byte;

    pub(super) fn not_parentheses(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
        recognize(many0(alt((
            take_while1(|byte| byte != b'\\' && byte != b'(' && byte != b')'),
            recognize(pair(tag(br"\"), take(1usize))),
        ))))(buffer)
    }

    pub(super) fn inner_parentheses(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
        recognize(delimited(
            char('('),
            pair(
                not_parentheses,
                many0(pair(inner_parentheses, not_parentheses)),
            ),
            char(')'),
        ))(buffer)
    }
}

mod escape {

    use super::*;
    use crate::process::escape::error::EscapeResult;
    use crate::process::escape::Escape;
    use crate::Byte;

    #[derive(Debug, Copy, Clone)]
    enum PrevByte {
        Solidus,
        Cr,
        Octal { digits: Byte, value: Byte },
        Other,
    }

    impl Escape for Literal<'_> {
        /// REFERENCE:
        /// - [7.3.4.2 Literal strings, p25]
        /// - ["Table 3: Escape sequences in literal strings", p25-26]
        fn escape(&self) -> EscapeResult<Vec<Byte>> {
            let mut prev = PrevByte::Other;
            let mut escaped = Vec::with_capacity(self.value.len());

            for &byte in self.value.iter() {
                match (byte, prev) {
                    (b'\\', PrevByte::Other | PrevByte::Cr) => {
                        prev = PrevByte::Solidus;
                    }
                    (b'\\', PrevByte::Octal { value, .. }) => {
                        escaped.push(value);
                        prev = PrevByte::Solidus;
                    }
                    (b'\\', PrevByte::Solidus) => {
                        escaped.push(b'\\');
                        prev = PrevByte::Other;
                    }
                    (b'n', PrevByte::Solidus) => {
                        escaped.push(b'\n');
                        prev = PrevByte::Other;
                    }
                    (b'r', PrevByte::Solidus) => {
                        escaped.push(b'\r');
                        prev = PrevByte::Other;
                    }
                    (b't', PrevByte::Solidus) => {
                        escaped.push(b'\t');
                        prev = PrevByte::Other;
                    }
                    (b'b', PrevByte::Solidus) => {
                        escaped.push(b'\x08');
                        prev = PrevByte::Other;
                    }
                    (b'f', PrevByte::Solidus) => {
                        escaped.push(b'\x0C');
                        prev = PrevByte::Other;
                    }
                    (b'(', PrevByte::Solidus) => {
                        escaped.push(b'(');
                        prev = PrevByte::Other;
                    }
                    (b')', PrevByte::Solidus) => {
                        escaped.push(b')');
                        prev = PrevByte::Other;
                    }
                    // REFERENCE: [7.3.4.2 Literal strings, p26]
                    // Ignore the reverse solidus and the end-of-line marker if
                    // an end-of-line marker follows the reverse solidus.
                    (b'\r', PrevByte::Solidus) => {
                        // \CR -> ''
                        prev = PrevByte::Cr;
                    }
                    // REFERENCE: [7.3.4.2 Literal strings, p26]
                    // Replace all a non-escaped end-of-line marker with a line
                    // feed character.
                    (b'\r', PrevByte::Other | PrevByte::Cr) => {
                        // CR -> LF
                        escaped.push(b'\n');
                        prev = PrevByte::Cr;
                    }
                    (b'\r', PrevByte::Octal { value, .. }) => {
                        escaped.push(value);
                        escaped.push(b'\n');
                        prev = PrevByte::Cr;
                    }
                    // REFERENCE: [7.3.4.2 Literal strings, p26]
                    // Ignore the reverse solidus and the end-of-line marker if
                    // an end-of-line marker follows the reverse solidus.
                    (b'\n', PrevByte::Solidus) => {
                        // \LF -> ''
                        prev = PrevByte::Other;
                    }
                    // CR was already processed and converted to LF
                    (b'\n', PrevByte::Cr) => {
                        // CRLF -> LF
                        prev = PrevByte::Other;
                    }
                    (b'\n', PrevByte::Other) => {
                        // LF -> LF
                        escaped.push(b'\n');
                    }
                    // REFERENCE: [7.3.4.2 Literal strings, p26]
                    // One, two or three octal digits can be used to represent
                    // any byte value.
                    (b'0'..=b'7', PrevByte::Solidus) => {
                        let value = byte - b'0';
                        prev = PrevByte::Octal { digits: 1, value };
                    }
                    (b'0'..=b'7', PrevByte::Octal { digits, value }) => {
                        let digit = byte - b'0';
                        if let (Some(value), 1 | 2) = (extend_octal(value, digit), digits) {
                            prev = PrevByte::Octal {
                                value,
                                digits: digits + 1,
                            };
                        } else {
                            escaped.push(value);
                            escaped.push(byte);
                            prev = PrevByte::Other;
                        }
                    }
                    // REFERENCE: [7.3.4.2 Literal strings, p26]
                    // Ignore the reverse solidus when followed by a character
                    // not in "Table 3 — Escape sequences in literal strings".
                    (_, PrevByte::Solidus | PrevByte::Cr) => {
                        // TODO Replace with `log::warn!`
                        eprintln!(
                            "REVERSE SOLIDUS followed by an unsupported byte: \\x{:02X}",
                            byte
                        );
                        escaped.push(byte);

                        prev = PrevByte::Other;
                    }
                    (_, PrevByte::Other) => {
                        escaped.push(byte);
                    }
                    (_, PrevByte::Octal { value, .. }) => {
                        escaped.push(value);
                        escaped.push(byte);
                        prev = PrevByte::Other;
                    }
                }
            }
            if let PrevByte::Octal { value, .. } = prev {
                escaped.push(value);
            }

            Ok(escaped)
        }
    }

    // TODO Convert into a Result
    pub(super) fn extend_octal(octal: Byte, digit: Byte) -> Option<Byte> {
        if digit > 7 {
            unreachable!(
                "The caller provides a digit between 0 and 7, found: {}",
                digit
            );
        }
        if octal >= 32 {
            return None;
        }
        Some(octal * 8 + digit)
    }
}

mod encode {
    use ::std::ffi::OsString;

    use super::*;
    use crate::process::encoding::error::EncodingResult;
    use crate::process::encoding::Encoding;

    impl Literal<'_> {
        // pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
        // let encoded = encoding.encode(string)?;
        // Ok(Self(encoded.into()))
        // }

        pub(crate) fn decode(&self, _encoding: Encoding) -> EncodingResult<OsString> {
            // encoding.decode(&self.escape()?)
            todo!("Implement Literal::decode")
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;
    use crate::Byte;

    impl<'buffer> Literal<'buffer> {
        pub fn new(value: &'buffer [Byte], span: Span) -> Self {
            Self { value, span }
        }
    }

    impl<'buffer> From<(&'buffer str, Span)> for Literal<'buffer> {
        fn from((value, span): (&'buffer str, Span)) -> Self {
            Self::new(value.as_bytes(), span)
        }
    }

    impl<'buffer> Deref for Literal<'buffer> {
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

    #[test]
    fn string_literal_valid() {
        // Synthetic tests
        parse_assert_eq!(
            Literal,
            b"(A literal string)",
            Literal::from(("A literal string", Span::new(0, 18))),
        );
        parse_assert_eq!(
            Literal,
            b"(A literal
string)",
            Literal::from((r"A literal\nstring", Span::new(0, 18))),
        );
        parse_assert_eq!(
            Literal,
            br"({A \(literal string!()} with unbalanced escaped parentheses)",
            Literal::from((
                r"{A \(literal string!()} with unbalanced escaped parentheses",
                Span::new(0, 61)
            )),
        );
        parse_assert_eq!(
            Literal,
            b"(((A))literal(string)(()))",
            Literal::from(("((A))literal(string)(())", Span::new(0, 26))),
        );
    }

    #[test]
    fn string_literal_invalid() {
        // Synthetic tests
        // Literal: Missing end parenthesis
        let parse_result = Literal::parse(b"(Unbalanced parentheses", 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Literal),
            ParseErrorCode::MissingClosing(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Literal: Missing end parenthesis
        let parse_result = Literal::parse(br"(Escaped parentheses\)", 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Literal),
            ParseErrorCode::MissingClosing(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Literal: Not found at the start of the buffer
        let parse_result = Literal::parse(b"Unbalanced parentheses)", 0);
        let expected_error = ParseRecoverable::new(
            b"Unbalanced parentheses)",
            stringify!(Literal),
            ParseErrorCode::NotFound(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);
    }

    #[test]
    fn string_literal_escape() {
        // Synthetic tests
        let literal_solidus_eol = Literal::from((
            r"A \
literal \
string",
            Span::new(0, 21),
        ));
        let literal_solidus_eol_escaped = Literal::from(("A literal string", Span::new(0, 21)));
        assert_eq!(literal_solidus_eol, literal_solidus_eol_escaped);

        let literal_eol = Literal::from((
            "A literal string
",
            Span::new(0, 17),
        ));
        let literal_eol_escaped = Literal::from((r"A literal string\n", Span::new(0, 17)));
        assert_eq!(literal_eol, literal_eol_escaped);

        let literal_eol_2 = Literal::from(("A literal string\r\n", Span::new(0, 19)));
        let literal_eol_2_escaped = Literal::from(("A literal string\n", Span::new(0, 19)));
        assert_eq!(literal_eol_2, literal_eol_2_escaped);

        let literal_unsupported_solidus =
            Literal::from((r"Unsupported \ escape.", Span::new(0, 22)));
        let literal_unsupported_solidus_escaped =
            Literal::from(("Unsupported  escape.", Span::new(0, 22)));
        assert_eq!(
            literal_unsupported_solidus,
            literal_unsupported_solidus_escaped
        );

        let literal_unsupported_escape =
            Literal::from((r"Unsupported escape \z.", Span::new(0, 22)));
        let literal_unsupported_escape_escaped =
            Literal::from((r"Unsupported escape z.", Span::new(0, 22)));
        assert_eq!(
            literal_unsupported_escape,
            literal_unsupported_escape_escaped
        );

        let literal = Literal::from((r"\101", Span::new(0, 4)));
        let literal_escaped = Literal::from((r"A", Span::new(0, 4)));
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from((r"\377", Span::new(0, 4)));
        let literal_escaped = Literal::new(b"\xFF".as_slice(), Span::new(0, 4));
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from((r"\77", Span::new(0, 3)));
        let literal_escaped = Literal::from(("?", Span::new(0, 3)));
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from((r"\077", Span::new(0, 4)));
        let literal_escaped = Literal::from(("?", Span::new(0, 4)));
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from((
            r"\124\150\151\163\40\151\163\40\141\40\163\164\162\151\156\147\40\151\156\40\157\143\164\141\154\40\162\145\160\162\145\163\145\156\164\141\164\151\157\156\56",
            Span::new(0, 77),
        ));
        let literal_escaped = Literal::from((
            r"This is a string in octal representation.",
            Span::new(0, 77),
        ));
        assert_eq!(literal, literal_escaped);
    }

    #[test]

    fn non_parentheses() {
        // Synthetic tests
        // Here, all cases need to be terminated by a non-escaped parenthesis ( or )
        assert_eq!(
            parse::not_parentheses(br"0abc\)\\"),
            Ok((b"".as_slice(), br"0abc\)\\".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(b"abc("),
            Ok((b"(".as_slice(), b"abc".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(br"abc\()"),
            Ok((b")".as_slice(), br"abc\(".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(b"abc)"),
            Ok((b")".as_slice(), b"abc".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(br"abc\)("),
            Ok((b"(".as_slice(), br"abc\)".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(br"abc\\)"),
            Ok((b")".as_slice(), br"abc\\".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(br"abc\\_)"),
            Ok((b")".as_slice(), br"abc\\_".as_slice()))
        );

        assert_eq!(
            parse::not_parentheses(br"abc\_("),
            Ok((b"(".as_slice(), br"abc\_".as_slice()))
        );
    }
}
