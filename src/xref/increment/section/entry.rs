use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

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

mod convert {
    use self::error::EntryFailure;
    use super::*;
    use crate::fmt::debug_bytes;
    use crate::parse::error::ParseFailure;
    use crate::parse::num::ascii_to_u16;
    use crate::parse::num::ascii_to_u64;
    use crate::Byte;

    impl TryFrom<((&[Byte], &[Byte]), char)> for Entry {
        type Error = ParseFailure;

        fn try_from(value: ((&[Byte], &[Byte]), char)) -> Result<Self, Self::Error> {
            let ((num_64, num_16), entry_type) = value;
            match entry_type {
                'f' => {
                    let next_free = ascii_to_u64(num_64)
                        .ok_or_else(|| EntryFailure::NextFree(debug_bytes(num_64)))?;
                    let generation_number = ascii_to_u16(num_16)
                        .ok_or_else(|| EntryFailure::GenerationNumber(debug_bytes(num_16)))?;
                    Ok(Self::Free(next_free, generation_number))
                }
                'n' => {
                    let offset = ascii_to_u64(num_64)
                        .ok_or_else(|| EntryFailure::OffSet(debug_bytes(num_64)))?;
                    let generation_number = ascii_to_u16(num_16)
                        .ok_or_else(|| EntryFailure::GenerationNumber(debug_bytes(num_16)))?;
                    Ok(Self::InUse(offset, generation_number))
                }
                _ => Err(EntryFailure::EntryType(entry_type.to_string()).into()),
            }
        }
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum EntryFailure {
        #[error("Invalid offset. Input: {0}")]
        OffSet(String),
        #[error("Invalid next free object number. Input: {0}")]
        NextFree(String),
        #[error("Invalid generation number. Input: {0}")]
        GenerationNumber(String),
        #[error("Invalid entry type. Input: {0}")]
        EntryType(String),
    }
}
