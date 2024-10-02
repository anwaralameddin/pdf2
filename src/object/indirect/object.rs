use ::nom::bytes::complete::tag;
use ::nom::combinator::opt;
use ::nom::sequence::delimited;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::id::Id;
use super::IndirectValue;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse::KW_ENDOBJ;
use crate::parse::KW_OBJ;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.10 Indirect objects, p33]
#[derive(Debug, PartialEq)]
pub(crate) struct IndirectObject<'buffer> {
    pub(crate) id: Id,
    pub(crate) value: IndirectValue<'buffer>,
    pub(crate) span: Span,
}

impl Display for IndirectObject<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}\n{}\n{}", self.id, KW_OBJ, self.value, KW_ENDOBJ)
    }
}

impl<'buffer> ObjectParser<'buffer> for IndirectObject<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<Self> {
        // REFERENCE: [7.3.10 Indirect objects, p33]
        let id = Id::parse(buffer, offset).map_err(|err| {
            ParseRecoverable::new(
                err.buffer(),
                stringify!(Id),
                ParseErrorCode::RecNotFound(Box::new(err.code())),
            )
        })?;
        let id_span = id.span();
        let offset = id_span.end();
        let remains = &buffer[offset..];
        let remains_len = remains.len();

        let (remains, _) = terminated(tag(KW_OBJ), opt(white_space_or_comment))(remains).map_err(
            parse_recoverable!(
                e,
                ParseRecoverable::new(
                    e.input,
                    stringify!(IndirectObject),
                    ParseErrorCode::NotFound(e.code)
                )
            ),
        )?;

        let offset = offset + (remains_len - remains.len());
        // Here, we know that the buffer starts with an indirect object, and
        // the following errors should be propagated as IndirectObjectFailure
        let object = IndirectValue::parse(buffer, offset).map_err(|err| {
            ParseFailure::new(
                err.buffer(),
                stringify!(IndirectObject),
                ParseErrorCode::RecMissingSubobject(
                    stringify!(IndirectValue),
                    Box::new(err.code()),
                ),
            )
        })?;
        let object_span = object.span();
        let offset = object_span.end();
        let remains = &buffer[offset..];

        // REFERENCE: [7.3.8.1 General, p31]
        let (remains, _) = delimited(
            opt(white_space_or_comment),
            tag(KW_ENDOBJ),
            opt(white_space_or_comment),
        )(remains)
        .map_err(parse_failure!(
            e,
            ParseFailure::new(
                e.input,
                stringify!(IndirectObject),
                ParseErrorCode::MissingClosing(e.code),
            )
        ))?;

        let span = Span::new(
            id_span.start(),
            id_span.len() + (remains_len - remains.len()),
        );
        let indirect_object = Self {
            id,
            value: object,
            span,
        };
        Ok(indirect_object)
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use super::*;

    impl<'buffer> IndirectObject<'buffer> {
        pub(crate) fn new(id: Id, value: impl Into<IndirectValue<'buffer>>, span: Span) -> Self {
            Self {
                id,
                value: value.into(),
                span,
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::numeric::Integer;
    use crate::object::indirect::reference::Reference;
    use crate::object::indirect::stream::Stream;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::parse_assert_eq;

    #[test]
    fn object_valid() {
        // Synthetic tests
        let buffer = b"1 0 obj /Name\nendobj";
        parse_assert_eq!(
            IndirectObject,
            buffer,
            IndirectObject {
                id: unsafe { Id::new_unchecked(1, 0, 0, 4) },
                value: Name::from(("Name", Span::new(8, 5))).into(),
                span: Span::new(0, buffer.len())
            },
        );

        let buffer = b"1 0 obj 2 0 R endobj";
        let value = unsafe { Reference::new_unchecked(2, 0, 8, 5) }.into();
        let object = IndirectObject {
            id: unsafe { Id::new_unchecked(1, 0, 0, 4) },
            value,
            span: Span::new(0, buffer.len()),
        };
        parse_assert_eq!(IndirectObject, buffer, object);

        let buffer =
            b"1 0 obj\n<</Length 29>>\nstream\nA stream with a direct length\nendstream\nendobj";
        let value = Stream::new(
            Dictionary::new(
                [(
                    KEY_LENGTH.to_vec(),
                    Integer::new(29, Span::new(18, 2)).into(),
                )],
                Span::new(8, 14),
            ),
            "A stream with a direct length".as_bytes(),
            Span::new(8, 62),
        );
        let object = IndirectObject {
            id: unsafe { Id::new_unchecked(1, 0, 0, 4) },
            value: value.into(),
            span: Span::new(0, buffer.len()),
        };
        parse_assert_eq!(IndirectObject, buffer, object);

        // TODO Add tests

        // Include an indirect stream object with an indirect reference to its
        // length
    }

    #[test]
    fn object_invalid() {
        // Synthetic tests
        // Indirect Object: Incomplete
        let parse_result = IndirectObject::parse(b"1 0 obj /Name e", 0);
        let expected_error = ParseFailure::new(
            b"e",
            stringify!(IndirectObject),
            ParseErrorCode::MissingClosing(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Missing the endobj keyword
        let parse_result = IndirectObject::parse(b"1 0 obj /Name <", 0);
        let expected_error = ParseFailure::new(
            b"<",
            stringify!(IndirectObject),
            ParseErrorCode::MissingClosing(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Incomplete object
        let parse_result = IndirectObject::parse(b"1 0 obj (A partial literal string", 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(IndirectObject),
            ParseErrorCode::RecMissingSubobject(
                stringify!(IndirectValue),
                Box::new(ParseErrorCode::MissingClosing(ErrorKind::Char)),
            ),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Missing the object keyword
        let parse_result = IndirectObject::parse(b"1 0 /Name<", 0);
        let expected_error = ParseRecoverable::new(
            b"/Name<",
            stringify!(IndirectObject),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Indirect Object: Missing value
        let parse_result = IndirectObject::parse(b"1 0 obj endobj", 0);
        let expected_error = ParseFailure::new(
            b"endobj",
            stringify!(IndirectObject),
            ParseErrorCode::RecMissingSubobject(
                stringify!(IndirectValue),
                Box::new(ParseErrorCode::NotFoundUnion),
            ),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
