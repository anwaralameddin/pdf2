use ::std::num::TryFromIntError;
use ::thiserror::Error;

use super::increment::stream::error::XRefStreamErrorCode;
use crate::fmt::debug_bytes;
use crate::object::error::ObjectErr;
use crate::object::indirect::id::Id;
use crate::Byte;
use crate::GenerationNumber;
use crate::IndexNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

pub(crate) type XRefResult<'buffer, T> = Result<T, XRefErr<'buffer>>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum XRefErr<'buffer> {
    #[error("Duplicate object number: {0}")]
    DuplicateObjectNumber(u64),
    #[error(
        "In-use Object. Number: {}. Generation: {}. Offset: {}",
        object_number,
        generation_number,
        offset
    )]
    InUseObjectNumber {
        object_number: ObjectNumberOrZero,
        generation_number: GenerationNumber,
        offset: Offset,
    },
    #[error(
        "Compressed Object. Number: {}. Stream: {}. Index: {}",
        object_number,
        stream_id,
        index
    )]
    CompressedObjectNumber {
        object_number: ObjectNumberOrZero,
        stream_id: Id,
        index: IndexNumber,
    },
    // TODO (TEMP) Replace Vec<Byte> below with Span
    #[error("Object number. Error {1}. Input{}", debug_bytes(.0))]
    StreamObjectNumber(Vec<Byte>, TryFromIntError),
    #[error("Generation number. Error: {1}. Input: {0}")]
    GenerationNumber(u64, TryFromIntError),
    #[error("Generation number. Input{}", debug_bytes(.0))]
    StreamGenerationNumber(Vec<Byte>),
    #[error("Field overflow. Input: {}", debug_bytes(.0))]
    StreamFieldOverflow(Vec<Byte>),
    #[error("Offset. Input: {}", debug_bytes(.0))]
    StreamOffSet(Vec<Byte>),
    #[error("Next free object. Input: {}", debug_bytes(.0))]
    StreamNextFree(Vec<Byte>),
    #[error("Index number. Input{}", debug_bytes(.0))]
    SteamIndexNumber(Vec<Byte>),
    //
    #[error("XRefStream: {0}")]
    XRefStream(#[from] XRefStreamErrorCode),
    // TODO (TEMP) Replace String with FilterErrorCode when Span is implemented
    #[error("Filter: {0}")]
    Filter(String),
    //
    #[error("Object: {0}")]
    Object(ObjectErr<'buffer>),
}

#[macro_export]
macro_rules! xref_err {
    ($e:ident, $error:expr) => {
        |err| match err {
            NomErr::Incomplete(_) => unreachable!(
                "::nom::complete functions do not return the Incomplete error variant."
            ),
            NomErr::Error($e) | NomErr::Failure($e) => $error,
        }
    };
}

mod convert {
    use super::*;
    use crate::impl_from_ref;
    use crate::process::filter::error::FilterErr;

    // TODO (TEMP) Replace with references
    impl From<FilterErr<'_>> for XRefErr<'_> {
        fn from(value: FilterErr) -> Self {
            XRefErr::Filter(value.to_string())
        }
    }

    impl_from_ref!('buffer, ObjectErr<'buffer>, Object, XRefErr<'buffer>);
}
