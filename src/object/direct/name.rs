use ::nom::character::complete::char;
use ::nom::sequence::preceded;
use ::nom::Err as NomErr;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::hash::Hash;
use ::std::hash::Hasher;

use crate::fmt::debug_bytes;
use crate::object::BorrowedBuffer;
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
#[derive(Clone, Copy)]
pub struct Name<'buffer>(&'buffer [Byte]);

#[derive(Clone)]
pub struct OwnedName(Bytes);

impl Display for Name<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "/")?;
        for &byte in self.0.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}

impl Display for OwnedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&Name::from(self), f)
    }
}

impl Debug for Name<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "/{}", debug_bytes(self.0))
    }
}

impl Debug for OwnedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&Name::from(self), f)
    }
}

impl PartialEq for Name<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped
        } else {
            // If an escape call fails, the name is not valid, so we don't need to compare
            false
        }
    }
}

impl PartialEq for OwnedName {
    fn eq(&self, other: &Self) -> bool {
        Name::from(self) == Name::from(other)
    }
}

impl PartialEq<&str> for Name<'_> {
    fn eq(&self, other: &&str) -> bool {
        if let Ok(name) = self.escape() {
            name == other.as_bytes()
        } else {
            false
        }
    }
}

impl PartialEq<str> for OwnedName {
    fn eq(&self, other: &str) -> bool {
        Name::from(self) == other
    }
}

impl Eq for Name<'_> {}

impl Eq for OwnedName {}

impl Hash for Name<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.escape() {
            Ok(escaped) => escaped.hash(state),
            _ => self.0.hash(state),
        }
    }
}

impl Hash for OwnedName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Name::from(self).hash(state)
    }
}

impl<'buffer> Parser<'buffer> for Name<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, value) =
            preceded(char('/'), printable_token)(buffer).map_err(parse_recoverable!(
                e,
                ParseRecoverable {
                    buffer: e.input,
                    object: stringify!(Name),
                    code: ParseErrorCode::NotFound(e.code),
                }
            ))?;

        let name = Self(value);
        Ok((buffer, name))
    }
}

impl Parser<'_> for OwnedName {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Name::parse(buffer).map(|(buffer, name)| (buffer, name.to_owned_buffer()))
    }
}

mod process {
    use ::nom::character::is_hex_digit;
    use ::std::ffi::OsString;
    use ::std::result::Result as StdResult;

    use super::error::NameEscape;
    use super::error::NameEscapeCode;
    use super::*;
    use crate::parse::num::hex_val;
    use crate::process::encoding::Decoder;
    use crate::process::encoding::Encoding;
    use crate::process::error::ProcessResult;
    use crate::process::escape::error::EscapeError;
    use crate::process::escape::error::OwnedEscapeError;

    #[derive(Debug, Clone, Copy)]
    enum PrevByte {
        NumberSign,
        FistHexDigit(Byte),
        Other,
    }

    impl Name<'_> {
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
                        let hex_digit = hex_val(byte).ok_or(NameEscape {
                            name: self,
                            code: NameEscapeCode::InvalidHexDigit(char::from(byte)),
                        })?;
                        prev = PrevByte::FistHexDigit(hex_digit);
                    }
                    (_, PrevByte::FistHexDigit(prev_hex_digit))
                        if is_hex_digit(byte) && prev_hex_digit < 16 =>
                    {
                        let hex_digit = hex_val(byte).ok_or(NameEscape {
                            name: self,
                            code: NameEscapeCode::InvalidHexDigit(char::from(byte)),
                        })?;
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
                        return Err(NameEscape {
                            name: self,
                            code: NameEscapeCode::InvalidHexDigit(char::from(c)),
                        }
                        .into());
                    }
                    (c, PrevByte::FistHexDigit(prev_hex_digit)) => {
                        return Err(NameEscape {
                            name: self,
                            code: NameEscapeCode::IncompleteHexCode(prev_hex_digit, char::from(c)),
                        }
                        .into());
                    }
                    (c, PrevByte::Other) => {
                        escaped.push(c);
                    }
                }
            }

            match prev {
                PrevByte::NumberSign => {
                    return Err(NameEscape {
                        name: self,
                        code: NameEscapeCode::TraillingNumberSign,
                    }
                    .into());
                }
                PrevByte::FistHexDigit(value) => {
                    return Err(NameEscape {
                        name: self,
                        code: NameEscapeCode::TraillingHexDigit(value),
                    }
                    .into());
                }
                PrevByte::Other => {}
            }

            Ok(escaped)
        }

        /// REFERENCE: [7.3.5 Name objects, p29]
        /// Names should be encoded as UTF-8 when interpreted as text.
        // TODO Implement `encode` and `decode` more generically as for `String_`s
        pub(crate) fn decode_as_utf8(&self) -> ProcessResult<OsString> {
            Encoding::Utf8.decode(&self.escape()?)
        }
    }

    impl OwnedName {
        pub(crate) fn escape(&self) -> StdResult<Vec<Byte>, OwnedEscapeError> {
            Name::from(self).escape().map_err(Into::into)
        }

        pub(crate) fn decode_as_utf8(&self) -> ProcessResult<OsString> {
            Name::from(self).decode_as_utf8()
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::OwnedName;
    use super::*;
    use crate::object::BorrowedBuffer;
    use crate::Byte;

    impl BorrowedBuffer for Name<'_> {
        type OwnedBuffer = OwnedName;

        fn to_owned_buffer(self) -> Self::OwnedBuffer {
            OwnedName(Bytes::from(self.0))
        }
    }

    impl<'buffer> From<&'buffer OwnedName> for Name<'buffer> {
        fn from(value: &'buffer OwnedName) -> Self {
            Name(value.0.as_ref())
        }
    }

    impl<'buffer> From<&'buffer [Byte]> for Name<'buffer> {
        fn from(value: &'buffer [Byte]) -> Self {
            Self(value)
        }
    }

    impl From<&[Byte]> for OwnedName {
        fn from(value: &[Byte]) -> Self {
            Name::from(value).to_owned_buffer()
        }
    }

    impl<'buffer> From<&'buffer str> for Name<'buffer> {
        fn from(value: &'buffer str) -> Self {
            Self::from(value.as_bytes())
        }
    }

    impl From<&str> for OwnedName {
        fn from(value: &str) -> Self {
            Name::from(value).to_owned_buffer()
        }
    }

    impl<'buffer> Deref for Name<'buffer> {
        type Target = &'buffer [Byte];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Deref for OwnedName {
        type Target = Bytes;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    use super::OwnedName;
    use crate::fmt::debug_bytes;
    use crate::Byte;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    #[error("Name Escape: {}. Error: {code}", debug_bytes(.name))]
    pub struct NameEscape<'buffer> {
        pub name: &'buffer [Byte],
        pub code: NameEscapeCode,
    }

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum NameEscapeCode {
        #[error("A non hexadecimal character following the number sign:  {0}")]
        InvalidHexDigit(char),
        #[error("Incomplete hex code: #{0:02X} followed by: {1}")]
        IncompleteHexCode(Byte, char),
        #[error("Trailing number sign")]
        TraillingNumberSign,
        #[error("Trailing hex digit: #{0:02X}")]
        TraillingHexDigit(Byte),
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum OwnedNameEscape {
        #[error("Name: A non hexadecimal character following the number sign: {1} in {0}")]
        InvalidHexDigit(OwnedName, char),
        #[error("Name: Incomplete hex code: #{1:x?} followed by: {2} in {0}")]
        IncompleteHexCode(OwnedName, Byte, char),
        #[error("Name: Trailing number sign in {0}")]
        TraillingNumberSign(OwnedName),
        #[error("Name: Trailing hex digit: #{1:x?} in {0}")]
        TraillingHexDigit(OwnedName, Byte),
    }

    mod convert {
        use super::NameEscape;
        use super::OwnedNameEscape;
        use crate::object::direct::name::Name;
        use crate::object::BorrowedBuffer;

        // TODO (TEMP) Remove this when ProcessErr is refactored to accept lifetime parameters
        impl From<NameEscape<'_>> for OwnedNameEscape {
            fn from(err: NameEscape) -> Self {
                let owned_name = Name(err.name).to_owned_buffer();
                match err.code {
                    super::NameEscapeCode::InvalidHexDigit(c) => {
                        Self::InvalidHexDigit(owned_name, c)
                    }
                    super::NameEscapeCode::IncompleteHexCode(b, c) => {
                        Self::IncompleteHexCode(owned_name, b, c)
                    }
                    super::NameEscapeCode::TraillingNumberSign => {
                        Self::TraillingNumberSign(owned_name)
                    }
                    super::NameEscapeCode::TraillingHexDigit(b) => {
                        Self::TraillingHexDigit(owned_name, b)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::error::NameEscape;
    use super::error::NameEscapeCode;
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
        let object = Name::from("Name#_");
        let expected_error = EscapeError::Name(NameEscape {
            name: &object,
            code: NameEscapeCode::InvalidHexDigit('_'),
        });

        escape_assert_err!(object, expected_error);

        // Name: Incomplete hex code
        let object = Name::from("Name#7_");
        let expected_error = EscapeError::Name(NameEscape {
            name: &object,
            code: NameEscapeCode::IncompleteHexCode(7, '_'),
        });
        escape_assert_err!(object, expected_error);

        // Name: Trailing number sign
        let object = Name::from("Name#");
        let expected_error = EscapeError::Name(NameEscape {
            name: &object,
            code: NameEscapeCode::TraillingNumberSign,
        });
        escape_assert_err!(object, expected_error);

        // Name: Trailing hex digit
        let object = Name::from("Name#7");
        let expected_error = EscapeError::Name(NameEscape {
            name: &object,
            code: NameEscapeCode::TraillingHexDigit(7),
        });
        escape_assert_err!(object, expected_error);
    }
}
