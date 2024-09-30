use ::nom::character::complete::char;
use ::nom::character::complete::digit1;
use ::nom::sequence::separated_pair;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::entry::Entry;
use crate::parse::character_set::eol;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::num::ascii_to_u64;
use crate::parse::num::ascii_to_usize;
use crate::parse::Parser;
use crate::parse::Span;
use crate::parse_recoverable;
use crate::Byte;
use crate::ObjectNumberOrZero;
use crate::Offset;

#[derive(Debug, PartialEq)]
pub(crate) struct Subsection {
    pub(crate) first_object_number: ObjectNumberOrZero,
    pub(crate) entries: Vec<Entry>,
    pub(crate) span: Span,
}

impl Display for Subsection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "{} {}", self.first_object_number, self.entries.len())?;
        for entry in &self.entries {
            write!(f, "{}", entry)?;
        }
        Ok(())
    }
}

impl Parser<'_> for Subsection {
    // REFERENCE: [7.5.4 Cross-reference table, p56-57]
    fn parse_span(buffer: &[Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
        let size = buffer.len();
        let start = offset;

        let (mut buffer, (first_object_number, entry_count)) =
            terminated(separated_pair(digit1, char(' '), digit1), eol)(buffer).map_err(
                parse_recoverable!(
                    e,
                    ParseRecoverable::new(
                        e.input,
                        stringify!(Subsection),
                        ParseErrorCode::NotFound(e.code)
                    )
                ),
            )?;
        // Here, we know that the buffer starts with a cross-reference subsection, and
        // the following errors should be propagated as SubsectionFail

        let first_object_number = ascii_to_u64(first_object_number).ok_or_else(|| {
            ParseFailure::new(
                first_object_number,
                stringify!(Subsection),
                ParseErrorCode::FirstObjectNumber,
            )
        })?;
        let entry_count = ascii_to_usize(entry_count).ok_or_else(|| {
            ParseFailure::new(
                entry_count,
                stringify!(Subsection),
                ParseErrorCode::EntryCount,
            )
        })?;

        let mut offset = offset + (size - buffer.len());
        let entries = (0..entry_count).try_fold(
            Vec::with_capacity(entry_count),
            |mut entries, index| -> ParseResult<Vec<Entry>> {
                let (remains, entry) = Entry::parse_span(buffer, offset).map_err(|err| {
                    ParseFailure::new(
                        err.buffer(),
                        stringify!(Subsection),
                        ParseErrorCode::SubsectionEntry {
                            index,
                            first_object_number,
                            entry_count,
                            code: Box::new(err.code()),
                        },
                    )
                })?;
                buffer = remains;
                offset = entry.span().end();
                entries.push(entry);
                Ok(entries)
            },
        )?;
        let span = Span::new(start, size - buffer.len());
        Ok((
            buffer,
            Self {
                first_object_number,
                entries,
                span,
            },
        ))
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use super::*;

    impl Subsection {
        pub(crate) fn new(
            first_object_number: ObjectNumberOrZero,
            entries: impl Into<Vec<Entry>>,
            span: Span,
        ) -> Self {
            Self {
                first_object_number,
                entries: entries.into(),
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
    use crate::parse::Span;
    use crate::parse_span_assert_eq;
    use crate::xref::increment::section::entry::Entry;
    use crate::xref::increment::section::entry::EntryData;

    #[test]
    fn subsection_valid() {
        // Synthetic test
        let buffer: &[Byte] = include_bytes!("../../../../tests/data/SYNTHETIC_subsection.bin");
        let subsection: Subsection = include!("../../../../tests/code/SYNTHETIC_subsection.rs");
        parse_span_assert_eq!(buffer, subsection, "trailer\r\n".as_bytes());

        // PDF produced by Microsoft Word for Office 365
        let buffer: &[Byte] = include_bytes!(
            "../../../../tests/data/B72168B54640B245A7CCF42DCDC8C026_subsection.bin"
        );
        let subsection: Subsection =
            include!("../../../../tests/code/B72168B54640B245A7CCF42DCDC8C026_subsection.rs");
        parse_span_assert_eq!(buffer, subsection, "trailer\r\n".as_bytes());
    }

    #[test]
    fn subsection_invalid() {
        // Synthetic tests

        // Subsection: Not found
        let buffer = b"0 1 R\r\n";
        let parse_result = Subsection::parse_span(buffer, 0);
        let expected_error = ParseRecoverable::new(
            b"R\r\n",
            stringify!(Subsection),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Incomplete buffer
        let buffer = b"\
        0 6\r\n0000000000 65535 f\r\n\
        0000000100 00000 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n";
        let parse_result = Subsection::parse_span(buffer, 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Subsection),
            ParseErrorCode::SubsectionEntry {
                index: 5,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::TakeWhileMN)),
            },
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Corrupted entry: Missing eol separator
        let buffer = b"0 6\r\n\
        0000000000 65535 f 0000000100 00000 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n\
        0000000500 00000 n\r\n";
        let parse_result = Subsection::parse_span(buffer, 0);
        let expected_error = ParseFailure::new(
            b"0000000100 00000 n\r\n\
                0000000200 00000 n\r\n\
                0000000300 00001 f\r\n\
                0000000400 00000 n\r\n\
                0000000500 00000 n\r\n",
            stringify!(Subsection),
            ParseErrorCode::SubsectionEntry {
                index: 0,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::TakeWhileMN)),
            },
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Missing generation number
        let buffer = b"0 6\r\n\
        0000000000 65535 f\r\n\
        0000000100 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n\
        0000000500 00000 n\r\n";
        let parse_result = Subsection::parse_span(buffer, 0);
        let expected_error = ParseFailure::new(
            b"n\r\n\
            0000000200 00000 n\r\n\
            0000000300 00001 f\r\n\
            0000000400 00000 n\r\n\
            0000000500 00000 n\r\n",
            stringify!(Subsection),
            ParseErrorCode::SubsectionEntry {
                index: 1,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::TakeWhileMN)),
            },
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Invalid entry type
        let buffer = b"0 6\r\n\
        0000000000 65535 r\r\n\
        0000000100 00000 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n\
        0000000500 00000 n\r\n";
        let parse_result = Subsection::parse_span(buffer, 0);
        let expected_error = ParseFailure::new(
            b"r\r\n\
            0000000100 00000 n\r\n\
            0000000200 00000 n\r\n\
            0000000300 00001 f\r\n\
            0000000400 00000 n\r\n\
            0000000500 00000 n\r\n",
            stringify!(Subsection),
            ParseErrorCode::SubsectionEntry {
                index: 0,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::Tag)),
            },
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
