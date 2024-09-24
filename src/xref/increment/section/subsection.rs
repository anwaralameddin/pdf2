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
use crate::parse_recoverable;
use crate::Byte;
use crate::ObjectNumberOrZero;

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Subsection {
    pub(crate) first_object_number: ObjectNumberOrZero,
    pub(crate) entries: Vec<Entry>,
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
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (mut buffer, (first_object_number, entry_count)) =
            terminated(separated_pair(digit1, char(' '), digit1), eol)(buffer).map_err(
                parse_recoverable!(
                    e,
                    ParseRecoverable {
                        buffer: e.input,
                        object: stringify!(Subsection),
                        code: ParseErrorCode::NotFound(e.code)
                    }
                ),
            )?;
        // Here, we know that the buffer starts with a cross-reference subsection, and
        // the following errors should be propagated as SubsectionFail

        let first_object_number = ascii_to_u64(first_object_number).ok_or(ParseFailure {
            buffer: first_object_number,
            object: stringify!(Subsection),
            code: ParseErrorCode::FirstObjectNumber,
        })?;
        let entry_count = ascii_to_usize(entry_count).ok_or(ParseFailure {
            buffer: entry_count,
            object: stringify!(Subsection),
            code: ParseErrorCode::EntryCount,
        })?;

        (0..entry_count)
            .try_fold(Vec::with_capacity(entry_count), |mut entries, index| {
                let (remains, entry) = Entry::parse(buffer).map_err(|err| ParseFailure {
                    buffer: err.buffer(),
                    object: stringify!(Subsection),
                    code: ParseErrorCode::Entry {
                        index,
                        first_object_number,
                        entry_count,
                        code: Box::new(err.code()),
                    },
                })?;
                buffer = remains;
                entries.push(entry);
                Ok(entries)
            })
            .map(|entries| {
                (
                    buffer,
                    Self {
                        first_object_number,
                        entries,
                    },
                )
            })
    }
}

mod convert {
    use super::*;

    impl Subsection {
        pub(crate) fn new(
            first_object_number: ObjectNumberOrZero,
            entries: impl Into<Vec<Entry>>,
        ) -> Self {
            Self {
                first_object_number,
                entries: entries.into(),
            }
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
    fn subsection_valid() {
        // Synthetic test
        let buffer: &[Byte] = include_bytes!("../../../../tests/data/SYNTHETIC_subsection.bin");
        let subsection: Subsection = include!("../../../../tests/code/SYNTHETIC_subsection.rs");
        parse_assert_eq!(buffer, subsection, "trailer\r\n".as_bytes());

        // PDF produced by Microsoft Word for Office 365
        let buffer: &[Byte] = include_bytes!(
            "../../../../tests/data/B72168B54640B245A7CCF42DCDC8C026_subsection.bin"
        );
        let subsection: Subsection =
            include!("../../../../tests/code/B72168B54640B245A7CCF42DCDC8C026_subsection.rs");
        parse_assert_eq!(buffer, subsection, "trailer\r\n".as_bytes());
    }

    #[test]
    fn subsection_invalid() {
        // Synthetic tests

        // Subsection: Not found
        let buffer = b"0 1 R\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseRecoverable {
            buffer: b"R\r\n",
            object: stringify!(Subsection),
            code: ParseErrorCode::NotFound(ErrorKind::Tag),
        };
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Incomplete buffer
        let buffer = b"\
        0 6\r\n0000000000 65535 f\r\n\
        0000000100 00000 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseFailure {
            buffer: b"",
            object: stringify!(Subsection),
            code: ParseErrorCode::Entry {
                index: 5,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::TakeWhileMN)),
            },
        };
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Corrupted entry: Missing eol separator
        let buffer = b"0 6\r\n\
        0000000000 65535 f 0000000100 00000 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n\
        0000000500 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseFailure {
            buffer: b"0000000100 00000 n\r\n\
                0000000200 00000 n\r\n\
                0000000300 00001 f\r\n\
                0000000400 00000 n\r\n\
                0000000500 00000 n\r\n",
            object: stringify!(Subsection),
            code: ParseErrorCode::Entry {
                index: 0,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::TakeWhileMN)),
            },
        };
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Missing generation number
        let buffer = b"0 6\r\n\
        0000000000 65535 f\r\n\
        0000000100 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n\
        0000000500 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseFailure {
            buffer: b"n\r\n\
            0000000200 00000 n\r\n\
            0000000300 00001 f\r\n\
            0000000400 00000 n\r\n\
            0000000500 00000 n\r\n",
            object: stringify!(Subsection),
            code: ParseErrorCode::Entry {
                index: 1,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::TakeWhileMN)),
            },
        };
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Invalid entry type
        let buffer = b"0 6\r\n\
        0000000000 65535 r\r\n\
        0000000100 00000 n\r\n\
        0000000200 00000 n\r\n\
        0000000300 00001 f\r\n\
        0000000400 00000 n\r\n\
        0000000500 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseFailure {
            buffer: b"r\r\n\
            0000000100 00000 n\r\n\
            0000000200 00000 n\r\n\
            0000000300 00001 f\r\n\
            0000000400 00000 n\r\n\
            0000000500 00000 n\r\n",
            object: stringify!(Subsection),
            code: ParseErrorCode::Entry {
                index: 0,
                first_object_number: 0,
                entry_count: 6,
                code: Box::new(ParseErrorCode::NotFound(ErrorKind::Tag)),
            },
        };
        assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
