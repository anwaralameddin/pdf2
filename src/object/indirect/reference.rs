use ::nom::bytes::complete::tag;
use ::nom::error::Error as NomError;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::id::Id;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse::KW_R;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.10 Indirect Objects, p33]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reference {
    id: Id,
    span: Span,
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.id, KW_R)
    }
}

impl ObjectParser<'_> for Reference {
    fn parse(buffer: &[Byte], offset: Offset) -> ParseResult<Self> {
        let id = Id::parse(buffer, offset).map_err(|err| {
            ParseRecoverable::new(
                err.buffer(),
                stringify!(Reference),
                ParseErrorCode::RecNotFound(Box::new(err.code())),
            )
        })?;
        let id_span = id.span();
        let remains = &buffer[id_span.end()..];

        // At this point, even though we have an Id, it is unclear if it is a
        // reference or a sequence of integers. For example, `12 0` appearing in
        // an array can be part of the indirect reference `12 0 R` or simply a
        // pair of integers in that array.
        tag::<_, _, NomError<_>>(KW_R.as_bytes())(remains).map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Reference),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;

        let span = Span::new(id_span.start(), id_span.len() + 1);
        let reference = Self { id, span };
        Ok(reference)
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl Deref for Reference {
        type Target = Id;

        fn deref(&self) -> &Self::Target {
            &self.id
        }
    }
}

#[cfg(test)]
mod tests {

    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse::Span;
    use crate::parse_assert_eq;
    use crate::GenerationNumber;

    impl Reference {
        pub fn new(id: Id, span: Span) -> Self {
            Self { id, span }
        }

        pub(crate) unsafe fn new_unchecked(
            object_number: u64,
            generation_number: GenerationNumber,
            start: usize,
            len: usize,
        ) -> Self {
            Self {
                id: Id::new_unchecked(object_number, generation_number, start, len - 1),
                span: Span::new(start, len),
            }
        }
    }

    #[test]
    fn reference_valid() {
        // Synthetic tests
        let reference = unsafe { Reference::new_unchecked(1, 0, 0, 5) };
        parse_assert_eq!(Reference, b"1 0 R", reference);
        let reference = unsafe { Reference::new_unchecked(12345, 65535, 0, 13) };
        parse_assert_eq!(Reference, b"12345 65535 R<<", reference);
        parse_assert_eq!(Reference, b"12345 65535 Rc", unsafe {
            Reference::new_unchecked(12345, 65535, 0, 13)
        },);
    }

    #[test]
    fn reference_invalid() {
        // Synthetic tests
        // Reference: Incomplete
        let parse_result = Reference::parse(b"1 0", 0);
        let expected_error = ParseRecoverable::new(
            b"",
            stringify!(Reference),
            ParseErrorCode::RecNotFound(Box::new(ParseErrorCode::NotFound(ErrorKind::Char))),
        );
        assert_err_eq!(parse_result, expected_error);

        // Reference: Id not found
        let parse_result = Reference::parse(b"/Name", 0);
        let expected_error = ParseRecoverable::new(
            b"/Name",
            stringify!(Reference),
            ParseErrorCode::RecNotFound(Box::new(ParseErrorCode::NotFound(ErrorKind::Digit))),
        );

        assert_err_eq!(parse_result, expected_error);

        // Reference: Id error
        let parse_result = Reference::parse(b"0 65535 R other objects", 0);
        let expected_error = ParseRecoverable::new(
            b"0",
            stringify!(Reference),
            ParseErrorCode::RecNotFound(Box::new(ParseErrorCode::ObjectNumber)),
        );
        assert_err_eq!(parse_result, expected_error);

        // Reference: Not found
        let parse_result = Reference::parse(b"12345 65535 <", 0);
        let expected_error = ParseRecoverable::new(
            b"<",
            stringify!(Reference),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
