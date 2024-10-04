use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::sequence::preceded;
use ::nom::Err as NomErr;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::character_set::printable_token;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse_recoverable;
use crate::process::escape::Escape;
use crate::Byte;
use crate::Offset;

// FIXME Take the PDF version into account when parsing names as #-escaped
// characters are not valid in PDF 1.0 or 1.1
// FIXME The below does not take into account that the null character is not
// allowed in names

/// REFERENCE: [7.3.5 Name objects, p27-28]
#[derive(Debug, Clone, Copy)]
pub struct Name<'buffer> {
    value: &'buffer [Byte],
    span: Span,
}

impl Display for Name<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "/")?;
        for &byte in self.value.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}

impl PartialEq for Name<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let (Ok(self_escaped), Ok(other_escaped)) = (self.escape(), other.escape()) {
            self_escaped == other_escaped && self.span == other.span
        } else {
            // If an escape call fails, the name is not valid, so we don't need
            // to compare
            false
        }
    }
}

impl<'buffer> ObjectParser<'buffer> for Name<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];

        let (_, value) = preceded(char('/'), recognize(opt(printable_token)))(remains).map_err(
            parse_recoverable!(
                e,
                ParseRecoverable::new(e.input, stringify!(Name), ParseErrorCode::NotFound(e.code))
            ),
        )?;

        let len = value.len() + 1;
        let span = Span::new(offset, len);
        Ok(Self { value, span })
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod escape {
    use ::nom::character::is_hex_digit;

    use super::*;
    use crate::parse::num::hex_val;
    use crate::process::escape::error::EscapeErr;
    use crate::process::escape::error::EscapeErrorCode;
    use crate::process::escape::error::EscapeResult;
    use crate::process::escape::Escape;

    #[derive(Debug, Clone, Copy)]
    enum PrevByte {
        NumberSign,
        FistHexDigit(Byte),
        Other,
    }

    impl Escape for Name<'_> {
        /// REFERENCE: [7.3.5 Name objects, p28]
        /// FIXME Only a prefix solidus, printable characters, number signs and
        /// pairs of hexadecimal digits are allowed in names.
        fn escape(&self) -> EscapeResult<Vec<Byte>> {
            // FIXME Fail if the inner bytes include non-printable tokens
            // or if #00 is found
            let mut escaped = Vec::with_capacity(self.value.len());
            let mut prev = PrevByte::Other;
            for &byte in self.value.iter() {
                match (byte, prev) {
                    (b'#', PrevByte::Other) => {
                        prev = PrevByte::NumberSign;
                    }
                    (_, PrevByte::NumberSign) if is_hex_digit(byte) => {
                        let hex_digit = hex_val(byte).ok_or_else(|| {
                            EscapeErr::new(self, EscapeErrorCode::InvalidHexDigit(char::from(byte)))
                        })?;
                        prev = PrevByte::FistHexDigit(hex_digit);
                    }
                    (_, PrevByte::FistHexDigit(prev_hex_digit))
                        if is_hex_digit(byte) && prev_hex_digit < 16 =>
                    {
                        let hex_digit = hex_val(byte).ok_or_else(|| {
                            EscapeErr::new(self, EscapeErrorCode::InvalidHexDigit(char::from(byte)))
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
                        return Err(EscapeErr::new(
                            self,
                            EscapeErrorCode::InvalidHexDigit(char::from(c)),
                        ));
                    }
                    (c, PrevByte::FistHexDigit(prev_hex_digit)) => {
                        return Err(EscapeErr::new(
                            self,
                            EscapeErrorCode::IncompleteHexCode(prev_hex_digit, char::from(c)),
                        ));
                    }
                    (c, PrevByte::Other) => {
                        escaped.push(c);
                    }
                }
            }

            match prev {
                PrevByte::NumberSign => {
                    return Err(EscapeErr::new(self, EscapeErrorCode::TraillingNumberSign));
                }
                PrevByte::FistHexDigit(value) => {
                    return Err(EscapeErr::new(
                        self,
                        EscapeErrorCode::TraillingHexDigit(value),
                    ));
                }
                PrevByte::Other => {}
            }

            Ok(escaped)
        }
    }
}

mod encode {
    use ::std::ffi::OsString;

    use super::*;
    use crate::process::encoding::error::EncodingResult;

    impl Name<'_> {
        /// REFERENCE: [7.3.5 Name objects, p29]
        /// Names should be encoded as UTF-8 when interpreted as text.
        // TODO Implement `encode` and `decode` more generically as for `String_`s
        pub(crate) fn decode_as_utf8(&self) -> EncodingResult<OsString> {
            // Encoding::Utf8.decode(&self.escape()?)
            todo!("Implement Name::decode_as_utf8")
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;
    use crate::Byte;

    impl<'buffer> Name<'buffer> {
        pub fn new(value: &'buffer [Byte], span: Span) -> Self {
            Self { value, span }
        }
    }

    impl<'buffer> From<(&'buffer str, Span)> for Name<'buffer> {
        fn from((value, span): (&'buffer str, Span)) -> Self {
            Self::new(value.as_bytes(), span)
        }
    }

    impl<'buffer> Deref for Name<'buffer> {
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
    use crate::escape_assert_err;
    use crate::parse::ObjectParser;
    use crate::parse_assert_eq;
    use crate::process::escape::error::EscapeErr;
    use crate::process::escape::error::EscapeErrorCode;
    use crate::process::escape::Escape;

    #[test]
    fn name_valid() {
        // Synthetic tests
        parse_assert_eq!(Name, b"/", Name::from(("", Span::new(0, 1))));
        parse_assert_eq!(Name, b"/ABC123", Name::from(("ABC123", Span::new(0, 7))));
        parse_assert_eq!(
            Name,
            b"/A_B+C^1!2@3",
            Name::from(("A_B+C^1!2@3", Span::new(0, 12))),
        );
        parse_assert_eq!(Name, b"/123", Name::from(("123", Span::new(0, 4))));
        parse_assert_eq!(
            Name,
            b"/.@domain",
            Name::from((".@domain", Span::new(0, 9))),
        );
        parse_assert_eq!(
            Name,
            b"/#41#20Name",
            Name::from(("A Name", Span::new(0, 11))),
        );
        parse_assert_eq!(
            Name,
            b"/#28Name#29",
            Name::from(("(Name)", Span::new(0, 11))),
        );
    }

    #[test]
    fn name_invalid() {
        // Synthetic tests
        // Name: Not found
        let parse_result = Name::parse(b"Name", 0);
        let expected_error = ParseRecoverable::new(
            b"Name",
            stringify!(Name),
            ParseErrorCode::NotFound(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);
    }

    #[test]
    fn name_escape_valid() {
        // Synthetic tests
        assert_eq!(
            Name::parse(b"/#41#20Name", 0).unwrap().escape().unwrap(),
            b"A Name"
        );
        assert_eq!(
            Name::parse(b"/#28Name#29", 0).unwrap().escape().unwrap(),
            b"(Name)"
        );
        assert_eq!(
            Name::parse(b"/#23Name", 0).unwrap().escape().unwrap(),
            b"#Name"
        );
    }

    #[test]
    fn name_escape_invalid() {
        // Synthetic tests

        // FIXME Calling escape on this object should return an error as ')' is
        // not a token
        // let object = &Name::from("Name)");

        // Name: A non-hexadecimal character following the number sign
        let object = Name::parse(b"/Name#_", 0).unwrap();
        let expected_error = EscapeErr::new(&object, EscapeErrorCode::InvalidHexDigit('_'));
        escape_assert_err!(object, expected_error);

        // Name: Incomplete hex code
        let object = Name::parse(b"/Name#7_", 0).unwrap();
        let expected_error = EscapeErr::new(&object, EscapeErrorCode::IncompleteHexCode(7, '_'));
        escape_assert_err!(object, expected_error);

        // Name: Trailing number sign
        let object = Name::parse(b"/Name#", 0).unwrap();
        let expected_error = EscapeErr::new(&object, EscapeErrorCode::TraillingNumberSign);
        escape_assert_err!(object, expected_error);

        // Name: Trailing hex digit
        let object = Name::parse(b"/Name#7", 0).unwrap();
        let expected_error = EscapeErr::new(&object, EscapeErrorCode::TraillingHexDigit(7));
        escape_assert_err!(object, expected_error);
    }
}
