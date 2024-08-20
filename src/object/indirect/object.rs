use ::nom::bytes::complete::tag;
use ::nom::combinator::opt;
use ::nom::error::ErrorKind;
use ::nom::sequence::delimited;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::IndirectObjectFailure;
use self::error::IndirectObjectRecoverable;
use super::id::Id;
use super::IndirectValue;
use super::Parser;
use crate::fmt::debug_bytes;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::KW_ENDOBJ;
use crate::parse::KW_OBJ;
use crate::parse_error;
use crate::parse_failure;
use crate::Byte;

/// REFERENCE: [7.3.10 Indirect objects, p33]
#[derive(Debug, PartialEq)]
pub(crate) struct IndirectObject {
    pub(crate) id: Id,
    pub(crate) value: IndirectValue,
}

impl Display for IndirectObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}\n{}\n{}", self.id, KW_OBJ, self.value, KW_ENDOBJ)
    }
}

impl Parser for IndirectObject {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        // REFERENCE: [7.3.10 Indirect objects, p33]
        let (buffer, id) = Id::parse_semi_quiet(buffer).unwrap_or_else(|| {
            Err(ParseErr::Error(
                IndirectObjectRecoverable::NotFound {
                    code: ErrorKind::Digit,
                    input: debug_bytes(buffer),
                }
                .into(),
            ))
        })?;
        let (buffer, _) =
            terminated(tag(KW_OBJ), opt(white_space_or_comment))(buffer).map_err(parse_error!(
                e,
                IndirectObjectRecoverable::NotFound {
                    code: e.code,
                    input: debug_bytes(e.input),
                }
            ))?;
        // Here, we know that the buffer starts with an indirect object, and
        // the following errors should be propagated as IndirectObjectFailure
        let (buffer, object) = IndirectValue::parse_semi_quiet(buffer).unwrap_or_else(|| {
            Err(ParseErr::Failure(
                IndirectObjectFailure::MissingValue(debug_bytes(buffer)).into(),
            ))
        })?;
        // REFERENCE: [7.3.8.1 General, p31]
        let (buffer, _) = delimited(
            opt(white_space_or_comment),
            tag(KW_ENDOBJ),
            opt(white_space_or_comment),
        )(buffer)
        .map_err(parse_failure!(
            e,
            IndirectObjectFailure::MissingClosing {
                code: e.code,
                input: debug_bytes(e.input),
            }
        ))?;

        let indirect_object = Self { id, value: object };
        Ok((buffer, indirect_object))
    }
}

mod convert {
    use super::*;

    impl IndirectObject {
        pub(crate) fn new(id: Id, value: impl Into<IndirectValue>) -> Self {
            Self {
                id,
                value: value.into(),
            }
        }
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IndirectObjectRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IndirectObjectFailure {
        #[error("Missing Value. Input: {0}")]
        MissingValue(String),
        #[error("Missing Closing: {code:?}. Input: {input}")]
        MissingClosing { code: ErrorKind, input: String },
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::string::literal::error::LiteralFailure;
    use crate::object::indirect::reference::Reference;
    use crate::object::indirect::stream::Stream;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::parse_assert_eq;

    #[test]
    fn object_valid() {
        // Synthetic tests
        let buffer = b"1 0 obj /Name\nendobj";
        parse_assert_eq!(
            buffer,
            IndirectObject {
                id: unsafe { Id::new_unchecked(1, 0) },
                value: Name::from("Name").into(),
            },
            "".as_bytes()
        );

        let buffer = b"1 0 obj 2 0 R endobj";
        let value = unsafe { Reference::new_unchecked(2, 0) }.into();
        let object = IndirectObject {
            id: unsafe { Id::new_unchecked(1, 0) },
            value,
        };
        parse_assert_eq!(buffer, object, "".as_bytes());

        let buffer =
            b"1 0 obj\n<</Length 29>>\nstream\nA stream with a direct length\nendstream\nendobj";
        let value = Stream::new(
            Dictionary::from_iter([(KEY_LENGTH.into(), 29.into())]),
            "A stream with a direct length".as_bytes(),
        );
        let object = IndirectObject {
            id: unsafe { Id::new_unchecked(1, 0) },
            value: value.into(),
        };
        parse_assert_eq!(buffer, object, "".as_bytes());

        // TODO Add tests

        // Include an indirect stream object with an indirect reference to its
        // length
    }

    #[test]
    fn object_invalid() {
        // Synthetic tests
        // Indirect Object: Incomplete
        let parse_result = IndirectObject::parse(b"1 0 obj /Name e");
        let expected_error = ParseErr::Failure(
            IndirectObjectFailure::MissingClosing {
                code: ErrorKind::Tag,
                input: "e".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Missing the endobj keyword
        let parse_result = IndirectObject::parse(b"1 0 obj /Name <");
        let expected_error = ParseErr::Failure(
            IndirectObjectFailure::MissingClosing {
                code: ErrorKind::Tag,
                input: "<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Incomplete object
        let parse_result = IndirectObject::parse(b"1 0 obj (A partial literal string");
        let expected_error = ParseErr::Failure(
            LiteralFailure::MissingClosing {
                code: ErrorKind::Char,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Missing the object keyword
        let parse_result = IndirectObject::parse(b"1 0 /Name<");
        let expected_error = ParseErr::Error(
            IndirectObjectRecoverable::NotFound {
                code: ErrorKind::Tag,
                input: "/Name<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Missing value
        let parse_result = IndirectObject::parse(b"1 0 obj endobj");
        let expected_error =
            ParseErr::Failure(IndirectObjectFailure::MissingValue("endobj".to_string()).into());
        assert_err_eq!(parse_result, expected_error);
    }
}
