use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take_until;
use ::nom::character::complete::char;
use ::nom::character::complete::digit1;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::delimited;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;

use crate::parse::character_set::eol;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_usize;
use crate::parse::Parser;
use crate::parse::Span;
use crate::parse::EOF;
use crate::parse::KW_STARTXREF;
use crate::parse_failure;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.5.4 Cross-reference table] and [7.5.5 File trailer]
/// The length of the last three lines of the file is constrained to:
/// ```rs
/// KW_STARTXREF.len() + EOL::MAX_LEN + OFFSET_MAX_LEN + EOL::MAX_LEN + EOF.len() +
/// EOL::MAX_LEN = 9 + 2 + 10 + 2 + 5 + 2 = 30,
/// ```
/// assuming the same offset length constraint applies to the offset in the
/// startxref line.
const STARTXREF_MAX_SIZE: usize = 30;

/// The minimum length of the last three lines of a PDF file:
/// ```rs
/// KW_STARTXREF.len() + EOL::MIN_LEN + OFFSET_MIN_LEN + EOL::MIN_LEN =
/// 9 + 1 + 1 + 1 + 5 = 17
/// ```
const STARTXREF_MIN_SIZE: usize = 17;

/// REFERENCE: [7.5.5 File trailer, p58]
#[derive(Debug, PartialEq)]
pub(crate) struct StartXRef {
    offset: Offset,
    span: Span,
}

impl Display for StartXRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n{}", KW_STARTXREF, self.offset, EOF)
    }
}

impl Parser<'_> for StartXRef {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Self::parse_span(buffer, 0)
    }

    fn parse_span(buffer: &[Byte], _: Offset) -> ParseResult<(&[Byte], Self)> {
        // TODO
        // - Indicate the offset will be ignored
        // - buffer and offset need to be coupled
        let offset = if let Some(offset) = buffer.len().checked_sub(STARTXREF_MAX_SIZE) {
            offset
        } else {
            return Err(ParseFailure::new(
                buffer,
                stringify!(StartXRef),
                ParseErrorCode::TooSmallBuffer,
            )
            .into());
        };
        // alt((char('\r'), char('\n'))) is used here instead of eol to allow
        // for a file ending with "%%EOF\r" instead of "%%EOF\r\n". Also,
        // ´complete` rather than `streaming` variants of `tag` and `char` are
        // used to ensure that the parser does return an Incomplete error when
        // the file ends with the EOF marker without trailing EOL characters.
        let (buffer, consumed) = take_until::<_, _, NomError<_>>(KW_STARTXREF)(&buffer[offset..])
            .unwrap_or((&buffer[offset..], &[]));
        let start = offset + consumed.len();
        let (remains, start_xref_offset) = delimited(
            terminated(tag(KW_STARTXREF), eol),
            digit1,
            delimited(eol, tag(EOF), opt(alt((char('\r'), char('\n'))))),
        )(buffer)
        .map_err(parse_failure!(
            e,
            // Except for Subsection, Section and XRefStream, NotFound errors
            // for xref objects should be propagated as failures.
            ParseFailure::new(
                e.input,
                stringify!(StartXRef),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        let end = start + (buffer.len() - remains.len());

        let start_xref_offset = ascii_to_usize(start_xref_offset).ok_or_else(|| {
            ParseFailure::new(
                start_xref_offset,
                stringify!(StartXRef),
                ParseErrorCode::Offset,
            )
        })?;

        Ok((
            &[],
            Self {
                offset: start_xref_offset,
                span: Span::new(start, end),
            },
        ))
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl StartXRef {
        pub(crate) fn new(offset: Offset, span: Span) -> Self {
            Self { offset, span }
        }
    }

    impl Deref for StartXRef {
        type Target = Offset;

        fn deref(&self) -> &Self::Target {
            &self.offset
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;

    #[test]
    fn start_xref_valid() {
        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/CD74097EBFE5D8A25FE8A229299730FA_xref_stream.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, start_xref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(start_xref_offset, StartXRef::new(238838, Span::new(7, 30)));

        // PDF produced by MikTeX pdfTeX-1.40.11
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/907C09F6EB56BEAF5235FAC6F37F5B84_trailer.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, start_xref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(start_xref_offset, StartXRef::new(265666, Span::new(7, 30)));

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xref_stream.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, start_xref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(start_xref_offset, StartXRef::new(309373, Span::new(7, 30)));

        // PDF produced by pdfTeX-1.40.22
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, start_xref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(start_xref_offset, StartXRef::new(365385, Span::new(7, 30)));
    }

    #[test]
    fn start_xref_invalid() {
        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_byte_offset.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = ParseFailure::new(
            b"%%EOF\r\n",
            stringify!(StartXRef),
            ParseErrorCode::NotFound(ErrorKind::Digit),
        );
        assert_err_eq!(parse_result, expected_error);

        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_eof.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = ParseFailure::new(
            b"%%PDF-1.4\r\n",
            stringify!(StartXRef),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_eol.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = ParseFailure::new(
            b"dobj\r\nstartxre\r\nf999999%%EOF\r\n",
            stringify!(StartXRef),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_startxref.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = ParseFailure::new(
            b"tream\r\nendobj\r\n999999\r\n%%EOF\r\n",
            stringify!(StartXRef),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
