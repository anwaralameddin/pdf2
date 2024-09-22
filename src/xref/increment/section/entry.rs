use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take_while_m_n;
use ::nom::character::complete::char;
use ::nom::combinator::map;
use ::nom::error::Error as NomError;
use ::nom::sequence::pair;
use ::nom::sequence::separated_pair;
use ::nom::sequence::terminated;
use ::nom::AsChar;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::new_parse_failure;
use crate::parse::character_set::is_white_space;
use crate::parse::error::NewParseErr;
use crate::parse::error::NewParseFailure;
use crate::parse::error::NewParseResult;
use crate::parse::error::ParseErrorCode;
use crate::parse::NewParser;
use crate::Byte;
use crate::GenerationNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

/// REFERENCE: [7.5.4 Cross-reference table, p56-57]
pub(super) const BIG_LEN: usize = 10;
/// REFERENCE: [7.5.4 Cross-reference table, p56-57]
pub(super) const SMALL_LEN: usize = 5;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Entry {
    // TODO(QUESTION): When can the generation number of free objects be zero?
    Free(ObjectNumberOrZero, GenerationNumber),
    InUse(Offset, GenerationNumber),
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // The trailing space is essential to ensure the entry is exactly 20 bytes.
        // That, on all platforms, the newline character appended by `writeln!` is the
        // line feed character without a carriage return.
        match self {
            Self::InUse(offset, generation_number) => writeln!(
                f,
                "{:0BIG_LEN$} {:0SMALL_LEN$} n ",
                offset, generation_number
            ),
            Self::Free(next_free, generation_number) => writeln!(
                f,
                "{:0BIG_LEN$} {:0SMALL_LEN$} f ",
                next_free, generation_number
            ),
        }
    }
}

impl NewParser<'_> for Entry {
    fn parse(buffer: &[Byte]) -> NewParseResult<(&[Byte], Self)> {
        let (buffer, entry) = terminated(
            map(
                separated_pair(
                    separated_pair(
                        take_while_m_n(BIG_LEN, BIG_LEN, AsChar::is_dec_digit),
                        char::<_, NomError<_>>(' '),
                        take_while_m_n(SMALL_LEN, SMALL_LEN, AsChar::is_dec_digit),
                    ),
                    char(' '),
                    alt((tag(b"f"), tag(b"n"))),
                ),
                |value| -> NewParseResult<Entry> { Entry::try_from(value).map_err(Into::into) },
            ),
            pair(
                // The below uses `many_m_n` instead of `eol` to parse exactly
                // 20 bytes per entry.
                take_while_m_n(1, 1, is_white_space),
                take_while_m_n(1, 1, |byte| byte == b'\n' || byte == b'\r'),
            ),
        )(buffer)
        .map_err(new_parse_failure!(
            e,
            NewParseFailure {
                buffer: e.input,
                code: ParseErrorCode::NotFound(stringify!(Entry), Some(e.code))
            }
        ))?;
        Ok((buffer, entry?))
    }
}

mod convert {
    use super::error::EntryCode;
    use super::*;
    use crate::parse::error::NewParseFailure;
    use crate::parse::num::ascii_to_u16;
    use crate::parse::num::ascii_to_u64;
    use crate::Byte;

    impl<'buffer> TryFrom<((&'buffer [Byte], &'buffer [Byte]), &'buffer [Byte])> for Entry {
        type Error = NewParseFailure<'buffer>;

        fn try_from(
            value: ((&'buffer [Byte], &'buffer [Byte]), &'buffer [Byte]),
        ) -> Result<Self, Self::Error> {
            let ((num_64, num_16), entry_type) = value;
            match entry_type {
                b"f" => {
                    let next_free = ascii_to_u64(num_64).ok_or(Self::Error {
                        buffer: num_64,
                        code: EntryCode::NextFree.into(),
                    })?;
                    let generation_number = ascii_to_u16(num_16).ok_or(Self::Error {
                        buffer: num_16,
                        code: EntryCode::GenerationNumber.into(),
                    })?;
                    Ok(Self::Free(next_free, generation_number))
                }
                b"n" => {
                    let offset = ascii_to_u64(num_64).ok_or(Self::Error {
                        buffer: num_64,
                        code: ParseErrorCode::OffSet(stringify!(Entry)),
                    })?;
                    let generation_number = ascii_to_u16(num_16).ok_or(Self::Error {
                        buffer: num_16,
                        code: EntryCode::GenerationNumber.into(),
                    })?;
                    Ok(Self::InUse(offset, generation_number))
                }
                _ => Err(Self::Error {
                    buffer: entry_type,
                    code: EntryCode::EntryType.into(),
                }),
            }
        }
    }
}

pub(crate) mod error {

    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum EntryCode {
        #[error("Next free object number")]
        NextFree,
        #[error("Generation number")]
        GenerationNumber,
        #[error("Entry type")]
        EntryType,
    }
}
