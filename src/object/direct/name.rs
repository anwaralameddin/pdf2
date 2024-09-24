use ::nom::character::complete::char;
use ::nom::sequence::preceded;
use ::nom::Err as NomErr;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::hash::Hash;

use crate::fmt::debug_bytes;
use crate::parse::character_set::printable_token;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_recoverable;
use crate::Byte;
use crate::Bytes;

// FIXME Take the PDF version into account when parsing names as #-escaped
// characters are not valid in PDF 1.0 or 1.1
// FIXME The below does not take into account that the null character is not
// allowed in names

/// REFERENCE: [7.3.5 Name objects, p27-28]
#[derive(Clone)]
pub struct Name(Bytes);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "/")?;
        for &byte in self.0.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "/{}", debug_bytes(&self.0))
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped
        } else {
            // If an escape call fails, the name is not valid, so we don't need to compare
            false
        }
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        if let Ok(name) = self.escape() {
            name == other.as_bytes()
        } else {
            false
        }
    }
}

impl Eq for Name {}

impl Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.escape() {
            Ok(escaped) => escaped.hash(state),
            _ => self.0.hash(state),
        }
    }
}

impl Parser<'_> for Name {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, value) =
            preceded(char('/'), printable_token)(buffer).map_err(parse_recoverable!(
                e,
                ParseRecoverable {
                    buffer: e.input,
                    object: stringify!(Name),
                    code: ParseErrorCode::NotFound(e.code),
                }
            ))?;

        let name = Self(value.into());
        Ok((buffer, name))
    }
}

mod process {
    use ::nom::character::is_hex_digit;
    use ::std::result::Result as StdResult;

    use self::error::NameEscape;
    use super::*;
    use crate::parse::num::hex_val;
    use crate::process::error::ProcessResult;
    use crate::process::escape::error::EscapeError;

    #[derive(Debug, Clone, Copy)]
    enum PrevByte {
        NumberSign,
        FistHexDigit(Byte),
        Other,
    }

    impl Name {
        /// REFERENCE: [7.3.5 Name objects, p28]
        /// FIXME Only a prefix solidus, printable characters, number signs and
        /// pairs of hexadecimal digits are allowed in names.
        pub(crate) fn escape(&self) -> StdResult<Vec<Byte>, EscapeError> {
            // FIXME Fail if the inner bytes include non-printable tokens
            // or if #00 is found
            let mut escaped = Vec::with_capacity(self.0.len());
            let mut prev = PrevByte::Other;
            for &byte in self.0.iter() {
                match (byte, prev) {
                    (b'#', PrevByte::Other) => {
                        prev = PrevByte::NumberSign;
                    }
                    (_, PrevByte::NumberSign) if is_hex_digit(byte) => {
                        let hex_digit = hex_val(byte).ok_or(NameEscape::InvalidHexDigit(
                            self.to_string(),
                            char::from(byte),
                        ))?; // TODO (TEMP) Refactor to avoid to_string
                        prev = PrevByte::FistHexDigit(hex_digit);
                    }
                    (_, PrevByte::FistHexDigit(prev_hex_digit))
                        if is_hex_digit(byte) && prev_hex_digit < 16 =>
                    {
                        let hex_digit = hex_val(byte).ok_or(NameEscape::InvalidHexDigit(
                            self.to_string(),
                            char::from(byte),
                        ))?; // TODO (TEMP) Refactor to avoid to_string
                        let value = prev_hex_digit * 16 + hex_digit;
                        escaped.push(value);
                        prev = PrevByte::Other;
                    }
                    (_, PrevByte::FistHexDigit(prev_hex_digit)) if is_hex_digit(byte) => {
                        unreachable!(
                            "Other branchs only create PrevByte::FistHexDigit with hex_digit < \
                             16, found: {}",
                            prev_hex_digit
                        );
                    }
                    (c, PrevByte::NumberSign) => {
                        return Err(
                            NameEscape::InvalidHexDigit(self.to_string(), char::from(c)).into()
                        );
                        // TODO (TEMP) Refactor to avoid to_string
                    }
                    (c, PrevByte::FistHexDigit(prev_hex_digit)) => {
                        return Err(NameEscape::IncompleteHexCode(
                            self.to_string(), // TODO (TEMP) Refactor to avoid to_string
                            prev_hex_digit,
                            char::from(c),
                        )
                        .into());
                    }
                    (c, PrevByte::Other) => {
                        escaped.push(c);
                    }
                }
            }

            match prev {
                PrevByte::NumberSign => {
                    return Err(NameEscape::TraillingNumberSign(self.to_string()).into());
                    // TODO (TEMP) Refactor to avoid to_string
                }
                PrevByte::FistHexDigit(value) => {
                    return Err(NameEscape::TraillingHexDigit(self.to_string(), value).into());
                    // TODO (TEMP) Refactor to avoid to_string
                }
                PrevByte::Other => {}
            }

            Ok(escaped)
        }

        /// REFERENCE: [7.3.5 Name objects, p29]
        /// Names should be encoded as UTF-8 when interpreted as text.
        // TODO Implement `encode` and `decode` more generically as for `String_`s
        pub(crate) fn decode_as_utf8(&self) -> ProcessResult<&str> {
            ::std::str::from_utf8(&self.0).map_err(Into::into)
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::Name;
    use super::*;
    use crate::Byte;

    impl From<&[Byte]> for Name {
        fn from(value: &[Byte]) -> Self {
            Self(value.into())
        }
    }

    impl From<&str> for Name {
        fn from(value: &str) -> Self {
            Self::from(value.as_bytes())
        }
    }

    impl Deref for Name {
        type Target = Bytes;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    use crate::Byte;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum NameEscape {
        #[error("Name: A non hexadecimal character following the number sign: {1} in {0}")]
        InvalidHexDigit(String, char),
        #[error("Name: Incomplete hex code: #{1:x?} followed by: {2} in {0}")]
        IncompleteHexCode(String, Byte, char),
        #[error("Name: Trailing number sign in {0}")]
        TraillingNumberSign(String),
        #[error("Name: Trailing hex digit: #{1:x?} in {0}")]
        TraillingHexDigit(String, Byte),
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use self::error::NameEscape;
    use super::*;
    use crate::assert_err_eq;
    use crate::escape_assert_err;
    use crate::parse_assert_eq;
    use crate::process::escape::error::EscapeError;

    #[test]
    fn name_valid() {
        // Synthetic tests
        parse_assert_eq!(b"/ABC123", Name::from("ABC123"), "".as_bytes());
        parse_assert_eq!(b"/A_B+C^1!2@3", Name::from("A_B+C^1!2@3"), "".as_bytes());
        parse_assert_eq!(b"/123", Name::from("123"), "".as_bytes());
        parse_assert_eq!(b"/.@domain(", Name::from(".@domain"), "(".as_bytes());
        parse_assert_eq!(b"/#41#20Name)", Name::from("A Name"), ")".as_bytes());
        parse_assert_eq!(b"/#28Name#29", Name::from("(Name)"), "".as_bytes());
    }

    #[test]
    fn name_invalid() {
        // Synthetic tests
        // Name: Not found
        let parse_result = Name::parse(b"Name");
        let expected_error = ParseRecoverable {
            buffer: b"Name",
            object: stringify!(Name),
            code: ParseErrorCode::NotFound(ErrorKind::Char),
        };
        assert_err_eq!(parse_result, expected_error);
    }

    #[test]
    fn name_escape_valid() {
        // Synthetic tests
        assert_eq!(Name::from("#41#20Name").escape().unwrap(), b"A Name");
        assert_eq!(Name::from("#28Name#29").escape().unwrap(), b"(Name)");
        assert_eq!(Name::from("#23Name").escape().unwrap(), b"#Name");
    }

    #[test]
    fn name_escape_invalid() {
        // Synthetic tests

        // FIXME Calling escape on this object should return an error as ')' is
        // not a token
        // let object = &Name::from("Name)");

        // Name: A non-hexadecimal character following the number sign
        let object = &Name::from("Name#_");
        let expected_error =
            EscapeError::Name(NameEscape::InvalidHexDigit(object.to_string(), '_'));
        escape_assert_err!(object, expected_error);

        // Name: Incomplete hex code
        let object = &Name::from("Name#7_");
        let expected_error =
            EscapeError::Name(NameEscape::IncompleteHexCode(object.to_string(), 7, '_'));
        escape_assert_err!(object, expected_error);

        // Name: Trailing number sign
        let object = &Name::from("Name#");
        let expected_error = EscapeError::Name(NameEscape::TraillingNumberSign(object.to_string()));
        escape_assert_err!(object, expected_error);

        // Name: Trailing hex digit
        let object = &Name::from("Name#7");
        let expected_error =
            EscapeError::Name(NameEscape::TraillingHexDigit(object.to_string(), 7));
        escape_assert_err!(object, expected_error);
    }
}
