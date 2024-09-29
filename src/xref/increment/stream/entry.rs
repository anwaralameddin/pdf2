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
    use super::*;
    use crate::parse::num::bytes_to_u16;
    use crate::parse::num::bytes_to_u64;
    use crate::parse::num::bytes_to_usize;
    use crate::xref::error::XRefErr;
    use crate::Byte;

    impl TryFrom<(&[Byte], &[Byte], &[Byte])> for Entry {
        type Error = XRefErr<'static>;
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
                bytes_to_u64(field1).ok_or_else(|| XRefErr::StreamFieldOverflow(field1.to_vec()))?
            };

            match value1 {
                0 => {
                    let next_free = bytes_to_u64(field2)
                        .ok_or_else(|| XRefErr::StreamNextFree(field2.to_vec()))?;
                    let generation_number = bytes_to_u16(field3)
                        .ok_or_else(|| XRefErr::StreamGenerationNumber(field3.to_vec()))?;
                    Ok(Self::Free(next_free, generation_number))
                }
                1 => {
                    let offset = bytes_to_usize(field2)
                        .ok_or_else(|| XRefErr::StreamOffSet(field2.to_vec()))?;
                    let generation_number = bytes_to_u16(field3)
                        .ok_or_else(|| XRefErr::StreamGenerationNumber(field3.to_vec()))?;

                    Ok(Self::InUse(offset, generation_number))
                }
                2 => {
                    let value2 = bytes_to_u64(field2)
                        .ok_or_else(|| XRefErr::StreamFieldOverflow(field2.to_vec()))?;
                    let object_number = ObjectNumber::try_from(value2)
                        .map_err(|err| XRefErr::StreamObjectNumber(field2.to_vec(), err))?;

                    // REFERENCE: [7.5.8.3 Cross-reference stream data, p68]
                    let id = Id::new(object_number, GenerationNumber::default());
                    let index = bytes_to_u64(field3)
                        .ok_or_else(|| XRefErr::StreamGenerationNumber(field3.to_vec()))?;
                    Ok(Self::Compressed(id, index))
                }
                _ => {
                    let value2 = bytes_to_u64(field2)
                        .ok_or_else(|| XRefErr::StreamFieldOverflow(field2.to_vec()))?;
                    let value3 = bytes_to_u64(field3)
                        .ok_or_else(|| XRefErr::StreamFieldOverflow(field3.to_vec()))?;
                    Ok(Self::NullReference(value1, value2, value3))
                }
            }
        }
    }
}
