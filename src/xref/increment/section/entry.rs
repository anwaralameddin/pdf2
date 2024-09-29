use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take_while_m_n;
use ::nom::character::complete::char;
use ::nom::error::Error as NomError;
use ::nom::sequence::pair;
use ::nom::sequence::separated_pair;
use ::nom::sequence::terminated;
use ::nom::AsChar;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::character_set::is_white_space;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_failure;
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

impl Parser<'_> for Entry {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, entry) = terminated(
            separated_pair(
                separated_pair(
                    take_while_m_n(BIG_LEN, BIG_LEN, AsChar::is_dec_digit),
                    char::<_, NomError<_>>(' '),
                    take_while_m_n(SMALL_LEN, SMALL_LEN, AsChar::is_dec_digit),
                ),
                char(' '),
                alt((tag(b"f"), tag(b"n"))),
            ),
            pair(
                // The below uses `many_m_n` instead of `eol` to parse exactly
                // 20 bytes per entry.
                take_while_m_n(1, 1, is_white_space),
                take_while_m_n(1, 1, |byte| byte == b'\n' || byte == b'\r'),
            ),
        )(buffer)
        .map_err(parse_failure!(
            e,
            // Except for Subsection, Section and XRefStream, NotFound errors
            // for xref objects should be propagated as failures.
            ParseFailure::new(e.input, stringify!(Entry), ParseErrorCode::NotFound(e.code))
        ))?;
        let entry = Entry::try_from(entry)?;
        Ok((buffer, entry))
    }
}

mod convert {
    use super::*;
    use crate::parse::error::ParseFailure;
    use crate::parse::num::ascii_to_u16;
    use crate::parse::num::ascii_to_u64;
    use crate::parse::num::ascii_to_usize;
    use crate::Byte;

    impl<'buffer> TryFrom<((&'buffer [Byte], &'buffer [Byte]), &'buffer [Byte])> for Entry {
        type Error = ParseFailure<'buffer>;

        fn try_from(
            value: ((&'buffer [Byte], &'buffer [Byte]), &'buffer [Byte]),
        ) -> Result<Self, Self::Error> {
            let ((num_64, num_16), entry_type) = value;
            match entry_type {
                b"f" => {
                    let next_free = ascii_to_u64(num_64).ok_or_else(||Self::Error::new(
                        num_64,
                        stringify!(Entry),
                        ParseErrorCode::NextFree,
                    ))?;
                    let generation_number = ascii_to_u16(num_16).ok_or_else(||Self::Error::new(
                        num_16,
                        stringify!(Entry),
                        ParseErrorCode::GenerationNumber,
                    ))?;
                    Ok(Self::Free(next_free, generation_number))
                }
                b"n" => {
                    let offset = ascii_to_usize(num_64).ok_or_else(||Self::Error::new(
                        num_64,
                        stringify!(Entry),
                        ParseErrorCode::OffSet,
                    ))?;
                    let generation_number = ascii_to_u16(num_16).ok_or_else(||Self::Error::new(
                        num_16,
                        stringify!(Entry),
                        ParseErrorCode::GenerationNumber,
                    ))?;
                    Ok(Self::InUse(offset, generation_number))
                }
                _ => Err(Self::Error::new(
                    entry_type,
                    stringify!(Entry),
                    ParseErrorCode::EntryType,
                )),
            }
        }
    }
}
