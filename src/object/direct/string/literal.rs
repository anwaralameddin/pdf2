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
use ::std::ffi::OsString;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::LiteralFailure;
use self::error::LiteralRecoverable;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_error;
use crate::parse_failure;
use crate::process::encoding::Encoding;
use crate::Byte;
use crate::Bytes;

/// REFERENCE: [7.3.4.2 Literal strings, p25-28}
#[derive(Clone)]
pub struct Literal(Bytes);

impl Literal {}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "(")?;
        for &byte in self.0.iter() {
            write!(f, "{}", byte as char)?;
        }
        write!(f, ")")
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({})", debug_bytes(&self.0))
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped
        } else {
            // If an escape call fails, the string is not valid, so we don't need to compare
            false
        }
    }
}

impl Parser for Literal {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        // NOTE: many0 does not result in Failures, so there is no need to
        // handle its errors separately from `char('<')`
        let (buffer, value) = preceded(
            char('('),
            recognize(pair(
                parse::not_parentheses,
                many0(pair(parse::inner_parentheses, parse::not_parentheses)),
            )),
        )(buffer)
        .map_err(parse_error!(
            e,
            LiteralRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(buffer)
            }
        ))?;
        // Here, we know that the buffer starts with a literal string, and
        // the following errors should be propagated as LiteralFailure
        let (buffer, _) = char::<_, NomError<_>>(')')(buffer).map_err(parse_failure!(
            e,
            LiteralFailure::MissingClosing {
                code: e.code,
                input: debug_bytes(buffer)
            }
        ))?;

        let literal = Self(value.into());
        Ok((buffer, literal))
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

mod process {

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::error::ProcessResult;
    use crate::Byte;

    #[derive(Debug, Copy, Clone)]
    enum PrevByte {
        Solidus,
        Cr,
        Octal { digits: Byte, value: Byte },
        Other,
    }

    impl Literal {
        /// REFERENCE:
        /// - [7.3.4.2 Literal strings, p25]
        /// - ["Table 3: Escape sequences in literal strings", p25-26]
        pub(crate) fn escape(&self) -> ProcessResult<Vec<Byte>> {
            let mut prev = PrevByte::Other;
            let mut escaped = Vec::with_capacity(self.0.len());

            for &byte in self.0.iter() {
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
                        if let (Some(value), 1 | 2) = (process::extend_octal(value, digit), digits)
                        {
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

        pub(crate) fn encode(encoding: Encoding, string: &OsString) -> ProcessResult<Self> {
            let encoded = encoding.encode(string)?;
            Ok(Self(encoded.into()))
        }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            encoding.decode(&self.escape()?)
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

mod convert {
    use ::std::ops::Deref;

    use super::*;
    use crate::Byte;

    impl From<&[Byte]> for Literal {
        fn from(value: &[Byte]) -> Self {
            Self(value.into())
        }
    }

    impl From<&str> for Literal {
        fn from(value: &str) -> Self {
            Self::from(value.as_bytes())
        }
    }

    impl Deref for Literal {
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
    pub enum LiteralRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum LiteralFailure {
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
    fn string_literal_valid() {
        // Synthetic tests
        parse_assert_eq!(
            b"(A literal string)",
            Literal::from("A literal string"),
            "".as_bytes(),
        );
        parse_assert_eq!(
            b"(A literal
string)",
            Literal::from(r"A literal\nstring"),
            "".as_bytes(),
        );
        parse_assert_eq!(
            br"({A \(literal string!()} with unbalanced escaped parentheses)",
            Literal::from(r"{A \(literal string!()} with unbalanced escaped parentheses"),
            "".as_bytes(),
        );
        parse_assert_eq!(
            b"(((A))literal(string)(()))",
            Literal::from("((A))literal(string)(())"),
            "".as_bytes()
        );
    }

    #[test]
    fn string_literal_invalid() {
        // Synthetic tests
        // Literal: Missing end parenthesis
        let parse_result = Literal::parse(b"(Unbalanced parentheses");
        let expected_error = ParseErr::Failure(
            LiteralFailure::MissingClosing {
                code: ErrorKind::Char,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Literal: Missing end parenthesis
        let parse_result = Literal::parse(br"(Escaped parentheses\)");
        let expected_error = ParseErr::Failure(
            LiteralFailure::MissingClosing {
                code: ErrorKind::Char,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Literal: Not found at the start of the buffer
        let parse_result = Literal::parse(b"Unbalanced parentheses)");
        let expected_error = ParseErr::Error(
            LiteralRecoverable::NotFound {
                code: ErrorKind::Char,
                input: "Unbalanced parentheses)".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);
    }

    #[test]
    fn string_literal_escape() {
        // Synthetic tests
        let literal_solidus_eol = Literal::from(
            r"A \
literal \
string",
        );
        let literal_solidus_eol_escaped = Literal::from("A literal string");
        assert_eq!(literal_solidus_eol, literal_solidus_eol_escaped);

        let literal_eol = Literal::from(
            "A literal string
",
        );
        let literal_eol_escaped = Literal::from(r"A literal string\n");
        assert_eq!(literal_eol, literal_eol_escaped);

        let literal_eol_2 = Literal::from(b"A literal string\r\n".as_slice());
        let literal_eol_2_escaped = Literal::from(b"A literal string\n".as_slice());
        assert_eq!(literal_eol_2, literal_eol_2_escaped);

        let literal_unsupported_solidus = Literal::from(r"Unsupported \ escape.");
        let literal_unsupported_solidus_escaped = Literal::from("Unsupported  escape.");
        assert_eq!(
            literal_unsupported_solidus,
            literal_unsupported_solidus_escaped
        );

        let literal_unsupported_escape = Literal::from(r"Unsupported escape \z.");
        let literal_unsupported_escape_escaped = Literal::from(r"Unsupported escape z.");
        assert_eq!(
            literal_unsupported_escape,
            literal_unsupported_escape_escaped
        );

        let literal = Literal::from(r"\101");
        let literal_escaped = Literal::from(r"A");
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from(r"\377");
        let literal_escaped = Literal::from(b"\xFF".as_slice());
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from(r"\77");
        let literal_escaped = Literal::from("?");
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from(r"\077");
        let literal_escaped = Literal::from("?");
        assert_eq!(literal, literal_escaped);

        let literal = Literal::from(
            r"\124\150\151\163\40\151\163\40\141\40\163\164\162\151\156\147\40\151\156\40\157\143\164\141\154\40\162\145\160\162\145\163\145\156\164\141\164\151\157\156\56",
        );
        let literal_escaped = Literal::from(r"This is a string in octal representation.");
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
