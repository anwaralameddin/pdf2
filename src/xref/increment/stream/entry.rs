use crate::object::indirect::id::Id;
use crate::GenerationNumber;
use crate::IndexNumber;
use crate::ObjectNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

/// REFERENCE:
/// - [7.5.8.3 Cross-reference stream data, p67]
/// - [Table 18 — Entries in a cross-reference stream, p67-68]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Entry {
    Free(ObjectNumberOrZero, GenerationNumber),
    InUse(Offset, GenerationNumber),
    Compressed(Id, IndexNumber),
    NullReference(u64, u64, u64),
}

mod convert {
    use self::error::EntryError;
    use super::*;
    use crate::parse::num::bytes_to_u64;
    use crate::process::error::ProcessErr;
    use crate::Byte;

    impl TryFrom<(&[Byte], &[Byte], &[Byte])> for Entry {
        type Error = ProcessErr;
        /// REFERENCE: [7.5.8.3 Cross-reference stream data, p67]
        fn try_from(value: (&[Byte], &[Byte], &[Byte])) -> Result<Self, Self::Error> {
            let (field1, field2, field3) = value;
            // REFERENCE: [Table 17 — Additional entries specific to a
            // cross-reference stream dictionary, p67]
            let value1 = if field1.is_empty() {
                1
            } else {
                // REFERENCE: [7.5.8.3 Cross-reference stream data, p67]
                // Fields are provided in the big-endian order
                bytes_to_u64(field1).ok_or_else(|| EntryError::Overflow(field1.to_vec()))?
            };
            let value2 =
                bytes_to_u64(field2).ok_or_else(|| EntryError::Overflow(field2.to_vec()))?;
            let value3 =
                bytes_to_u64(field3).ok_or_else(|| EntryError::Overflow(field3.to_vec()))?;

            match value1 {
                0 => {
                    let next_free = value2;
                    let generation_number = GenerationNumber::try_from(value3)
                        .map_err(|_| EntryError::GenerationNumber(value3))?;
                    Ok(Self::Free(next_free, generation_number))
                }
                1 => {
                    let offset = value2;
                    let generation_number = GenerationNumber::try_from(value3)
                        .map_err(|_| EntryError::GenerationNumber(value3))?;
                    Ok(Self::InUse(offset, generation_number))
                }
                2 => {
                    let object_number = ObjectNumber::try_from(value2)
                        .map_err(|_| EntryError::ObjectNumber(value2))?;
                    // REFERENCE: [7.5.8.3 Cross-reference stream data, p68]
                    let id = Id::new(object_number, GenerationNumber::default());
                    let index = value3;
                    Ok(Self::Compressed(id, index))
                }
                _ => Ok(Self::NullReference(value1, value2, value3)),
            }
        }
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    use crate::Byte;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum EntryError {
        #[error("Overflow: {0:?}")]
        Overflow(Vec<Byte>),
        #[error("Invalid generation number. Input: {0}")]
        GenerationNumber(u64),
        #[error("Invalid index number. Input: {0}")]
        ObjectNumber(u64),
        #[error("Invalid index number. Input: {0}")]
        IndexNumber(u64),
    }
}
