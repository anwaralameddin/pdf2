use ::nom::error::ErrorKind;
use ::thiserror::Error;

use crate::fmt::debug_bytes;
use crate::object::direct::dictionary::error::DataTypeError;
use crate::object::direct::name::OwnedName;
use crate::process::error::NewProcessErr;
use crate::Byte;
use crate::ObjectNumberOrZero;

pub(crate) type ParseResult<'buffer, T> = Result<T, ParseErr<'buffer>>;
/// Recoverable parsing error
/// This error is used when the parser is unable to determine the value type
/// and the buffer needs to be reprocessed with a different parser
pub(crate) type ParseRecoverable<'buffer> = ParseError<'buffer, true>;
/// Unrecoverable parsing error
/// This error is used when the parser is able to determine the value type
/// but fails to parse it completely
pub(crate) type ParseFailure<'buffer> = ParseError<'buffer, false>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseErr<'buffer> {
    #[error("Parse Recoverable: {0}")]
    Recoverable(ParseRecoverable<'buffer>),
    #[error("Parse Failure: {0}")]
    Failure(ParseFailure<'buffer>),
}

// TODO This type's constructors are too verbose. Use a macro to make it more
// concise
#[derive(Debug, Error, PartialEq, Clone)]
#[error("{object}. {code}. Buffer: {}", debug_bytes(.buffer))]
pub struct ParseError<'buffer, const RECOVERABLE: bool> {
    pub(crate) buffer: &'buffer [Byte],
    pub(crate) object: &'static str,
    pub(crate) code: ParseErrorCode,
}

// TODO IncrementErrorCode does not implement Copy. This is due to
// `Trailer::try_from` resulting in NewProcessErr
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseErrorCode {
    // Whole buffer errors
    #[error("Buffer is too small")]
    TooSmallBuffer,
    // Whole object errors
    #[error("Object type")]
    ObjectType,
    #[error("Not found. Error: {}", .0.description())]
    NotFound(ErrorKind),
    #[error("Missing data. Error: {}", .0.description())]
    MissingData(ErrorKind),
    #[error("Missing closing. Error: {}", .0.description())]
    MissingClosing(ErrorKind),
    // Union Errors
    #[error("Not found")]
    NotFoundUnion,
    // Collection Errors
    // TODO (TEMP) Replace with &'buffer ParseErrorCode<'buffer>
    #[error("Not found. Error: {0}")]
    RecNotFound(Box<ParseErrorCode>),
    #[error("Missing subobject {0}. Error: {1}")]
    RecMissingSubobject(&'static str, Box<ParseErrorCode>),
    #[error("Missing key: Error: {0}")]
    RecMissingKey(&'static str),
    #[error("Missing value. Key {0}. Error: {1}")]
    RecMissingValue(OwnedName, Box<ParseErrorCode>),
    #[error("Missing closing. Error: {0}")]
    RecMissingClosing(Box<ParseErrorCode>),
    #[error(
        "Entry number {} in subsection {} {}. Error: {}",
        index,
        first_object_number,
        entry_count,
        code
    )]
    Entry {
        index: usize,
        first_object_number: ObjectNumberOrZero,
        entry_count: usize,
        code: Box<ParseErrorCode>,
    },
    // TODO Move to NumErrorCode
    #[error("Object number")]
    ObjectNumber, // ObjectNumber::new
    #[error("Generation number")]
    GenerationNumber, // ascii_to_u16
    #[error("First object number")]
    FirstObjectNumber, // ascii_to_u64
    #[error("Offset")]
    OffSet, // ascii_to_usize
    #[error("Next free object number")]
    NextFree, // ascii_to_u64
    #[error("Entry type")]
    EntryType, // f or n
    #[error("Entry count")]
    EntryCount, // TODO ascii_to_usize
    #[error("Parse as i128")]
    ParseIntError, // ascii_to_i128
    #[error("Parse as f64")]
    ParseFloatError, // ascii_to_f64
    // Errors requiring processing
    #[error("Invalid trailer dictionary. Error: {0}")]
    InvalidTrailerDictionary(NewProcessErr),
    // TODO(TEMP) Refactor into a variant of this enum
    #[error("Value type. Error: {0}")]
    ValueType(#[from] DataTypeError<'static>),
}

#[macro_export]
macro_rules! parse_failure {
    ($e:ident, $failure:expr) => {
        |err| match err {
            NomErr::Incomplete(_) => unreachable!(
                "::nom::complete functions do not return the Incomplete error variant."
            ),
            NomErr::Error($e) | NomErr::Failure($e) => ParseErr::Failure($failure.into()),
        }
    };
}

#[macro_export]
macro_rules! parse_recoverable {
    ($e:ident, $error:expr) => {
        |err| match err {
            NomErr::Incomplete(_) => unreachable!(
                "::nom::complete functions do not return the Incomplete error variant."
            ),
            NomErr::Error($e) | NomErr::Failure($e) => ParseErr::Recoverable($error),
        }
    };
}

mod convert {
    use super::*;

    impl<'buffer> ParseErr<'buffer> {
        pub fn buffer(&self) -> &'buffer [Byte] {
            match self {
                Self::Recoverable(err) => err.buffer,
                Self::Failure(err) => err.buffer,
            }
        }
    }

    impl ParseErr<'_> {
        pub fn code(self) -> ParseErrorCode {
            match self {
                Self::Recoverable(err) => err.code,
                Self::Failure(err) => err.code,
            }
        }
    }

    impl<'buffer> From<ParseFailure<'buffer>> for ParseErr<'buffer> {
        fn from(err: ParseFailure<'buffer>) -> Self {
            ParseErr::Failure(err)
        }
    }

    impl<'buffer> From<ParseRecoverable<'buffer>> for ParseErr<'buffer> {
        fn from(err: ParseRecoverable<'buffer>) -> Self {
            ParseErr::Recoverable(err)
        }
    }
}
