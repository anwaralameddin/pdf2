use ::nom::error::ErrorKind;
use ::std::num::TryFromIntError;
use ::thiserror::Error;

use crate::error::DisplayUsingBuffer;
use crate::fmt::debug_bytes;
use crate::object::error::ObjectErr;
use crate::process::filter::error::FilterErr;
use crate::Byte;
use crate::GenerationNumber;
use crate::IndexNumber;
use crate::ObjectNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

pub(crate) type XRefResult<T> = Result<T, XRefErr>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum XRefErr {
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
        "Compressed Object. Number: {} 0. Stream: {} 0. Index: {}",
        object_number,
        stream_object_number,
        index
    )]
    CompressedObjectNumber {
        object_number: ObjectNumberOrZero,
        stream_object_number: ObjectNumber,
        index: IndexNumber,
    },
    #[error("Object number. Value: {}. Error {1}", debug_bytes(.0))]
    StreamObjectNumber(Vec<Byte>, TryFromIntError),
    #[error("Generation number. Value: {0}. Error: {1}")]
    GenerationNumber(u64, TryFromIntError),
    #[error("Generation number. Value: {}", debug_bytes(.0))]
    StreamGenerationNumber(Vec<Byte>),
    #[error("Field overflow. Value: {}", debug_bytes(.0))]
    StreamFieldOverflow(Vec<Byte>),
    #[error("Offset. Value: {}", debug_bytes(.0))]
    StreamOffSet(Vec<Byte>),
    #[error("Next free object. Value: {}", debug_bytes(.0))]
    StreamNextFree(Vec<Byte>),
    #[error("Index number. Value: {}", debug_bytes(.0))]
    SteamIndexNumber(Vec<Byte>),
    // This error variant should not be returned as `parser` in
    // `XRefStream::get_entries` should not error out
    #[error("Parsing Decoded data. Error kind: {}", .0.description())]
    EntriesDecodedParse(ErrorKind),
    #[error("Decoded data length {1}: Not a multiple of the sum of W values: {0:?}")]
    EntriesDecodedLength([usize; 3], usize),
    #[error(
        "Entries too short. First object number: {}. Entry count: {}. Missing the {}th entry",
        first_object_number,
        count,
        index
    )]
    EntriesTooShort {
        first_object_number: u64,
        count: IndexNumber,
        index: IndexNumber,
    },
    //
    #[error("Filter: {0}")]
    Filter(#[from] FilterErr),
    #[error("Object: {0}")]
    Object(#[from] ObjectErr),
}

impl DisplayUsingBuffer for XRefErr {
    fn display_using_buffer(&self, buffer: &[Byte]) -> String {
        match self {
            XRefErr::Filter(filter_err) => {
                format!("Filter. Error: {}", filter_err.display_using_buffer(buffer))
            }
            XRefErr::Object(object_err) => {
                format!("Object. Error: {}", object_err.display_using_buffer(buffer))
            }
            _ => self.to_string(),
        }
    }
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
