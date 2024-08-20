use ::nom::branch::alt;
use ::nom::bytes::complete::take_while_m_n;
use ::nom::character::complete::char;
use ::nom::combinator::map;
use ::nom::error::Error as NomError;
use ::nom::multi::many_m_n;
use ::nom::sequence::pair;
use ::nom::sequence::separated_pair;
use ::nom::sequence::terminated;
use ::nom::AsChar;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::num::ParseIntError;

use self::error::SubsectionFailure;
use self::error::SubsectionRecoverable;
use super::entry::Entry;
use super::entry::BIG_LEN;
use super::entry::SMALL_LEN;
use crate::fmt::debug_bytes;
use crate::parse::character_set::eol;
use crate::parse::character_set::is_white_space;
use crate::parse::character_set::number1;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_error;
use crate::parse_failure;
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

impl Parser for Subsection {
    // REFERENCE: [7.5.4 Cross-reference table, p56-57]
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, (first_object_number, entry_count)) =
            terminated(separated_pair(number1, char(' '), number1), eol)(buffer).map_err(
                parse_error!(
                    e,
                    SubsectionRecoverable::NotFound {
                        code: e.code,
                        input: debug_bytes(e.input),
                    }
                ),
            )?;
        // Here, we know that the buffer starts with a cross-reference subsection, and
        // the following errors should be propagated as SubsectionFail

        let first_object_number: ObjectNumberOrZero =
            first_object_number.parse().map_err(|err: ParseIntError| {
                ParseErr::Failure(
                    SubsectionFailure::ObjectNumber(
                        err.kind().clone(),
                        first_object_number.to_string(),
                    )
                    .into(),
                )
            })?;
        let entry_count: usize = entry_count.parse().map_err(|err: ParseIntError| {
            ParseErr::Failure(
                SubsectionFailure::EntryCount(err.kind().clone(), entry_count.to_string()).into(),
            )
        })?;

        // The below uses `many_m_n` instead of `eol` to parse exactly 20 bytes
        // per entry.
        let (buffer, entries): (&[Byte], Vec<ParseResult<Entry>>) = many_m_n(
            entry_count,
            entry_count,
            terminated(
                map(
                    separated_pair(
                        separated_pair(
                            take_while_m_n(BIG_LEN, BIG_LEN, AsChar::is_dec_digit),
                            char::<_, NomError<_>>(' '),
                            take_while_m_n(SMALL_LEN, SMALL_LEN, AsChar::is_dec_digit),
                        ),
                        char(' '),
                        alt((char('f'), char('n'))),
                    ),
                    |value| -> ParseResult<Entry> { Entry::try_from(value).map_err(Into::into) },
                ),
                pair(
                    take_while_m_n(1, 1, is_white_space),
                    take_while_m_n(1, 1, |byte| byte == b'\n' || byte == b'\r'),
                ),
            ),
        )(buffer)
        .map_err(parse_failure!(
            e,
            SubsectionFailure::ParseEntries {
                first_object_number,
                entry_count,
                code: e.code,
                input: debug_bytes(e.input),
            }
        ))?;

        let entries = entries.into_iter().collect::<ParseResult<Vec<_>>>()?;

        Ok((
            buffer,
            Self {
                first_object_number,
                entries,
            },
        ))
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

pub(crate) mod error {

    use ::nom::error::ErrorKind;
    use ::std::num::IntErrorKind;
    use ::thiserror::Error;

    use crate::ObjectNumberOrZero;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum SubsectionRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum SubsectionFailure {
        #[error("Invalid object number: {0:?}. Input: {1}")]
        ObjectNumber(IntErrorKind, String),
        #[error("Invalid entry count: {0:?}. Input: {1}")]
        EntryCount(IntErrorKind, String),
        #[error(
            "Invalid entries for subsection {first_object_number} {entry_count}: {code:?}. Input: \
             {input}"
        )]
        ParseEntries {
            first_object_number: ObjectNumberOrZero,
            entry_count: usize,
            code: ErrorKind,
            input: String,
        },
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
        let expected_error = ParseErr::Error(
            SubsectionRecoverable::NotFound {
                code: ErrorKind::Tag,
                input: "R\r\n".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Incomplete buffer
        let buffer = b"0 6\r\n0000000000 65535 f\r\n0000000100 00000 n\r\n0000000200 00000 n\r\n0000000300 00001 f\r\n0000000400 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseErr::Failure(
            SubsectionFailure::ParseEntries {
                first_object_number: 0,
                entry_count: 6,
                code: ErrorKind::TakeWhileMN,
                input: "".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Corrupted entry: Missing eol separator
        let buffer = b"0 6\r\n0000000000 65535 f 0000000100 00000 n\r\n0000000200 00000 n\r\n0000000300 00001 f\r\n0000000400 00000 n\r\n0000000500 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseErr::Failure(
            SubsectionFailure::ParseEntries {
                first_object_number: 0,
                entry_count: 6,
                code: ErrorKind::TakeWhileMN,
                input: "0000000100 00000 n\r\n0000000200 00000 n\r\n0000000300 00001 \
                        f\r\n0000000400 00000 n\r\n0000000500 00000 n\r\n"
                    .to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Missing generation number
        let buffer = b"0 6\r\n0000000000 65535 f\r\n0000000100 n\r\n0000000200 00000 n\r\n0000000300 00001 f\r\n0000000400 00000 n\r\n0000000500 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseErr::Failure(
            SubsectionFailure::ParseEntries {
                first_object_number: 0,
                entry_count: 6,
                code: ErrorKind::TakeWhileMN,
                input: "n\r\n0000000200 00000 n\r\n0000000300 00001 f\r\n0000000400 00000 \
                        n\r\n0000000500 00000 n\r\n"
                    .to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Subsection: Invalid entry type
        let buffer = b"0 6\r\n0000000000 65535 r\r\n0000000100 00000 n\r\n0000000200 00000 n\r\n0000000300 00001 f\r\n0000000400 00000 n\r\n0000000500 00000 n\r\n";
        let parse_result = Subsection::parse(buffer);
        let expected_error = ParseErr::Failure(
            SubsectionFailure::ParseEntries {
                first_object_number: 0,
                entry_count: 6,
                code: ErrorKind::Char,
                input: "r\r\n0000000100 00000 n\r\n0000000200 00000 n\r\n0000000300 00001 \
                        f\r\n0000000400 00000 n\r\n0000000500 00000 n\r\n"
                    .to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
