use ::nom::error::ErrorKind;
use ::thiserror::Error;

use crate::error::DisplayUsingBuffer;
use crate::fmt::debug_bytes;
use crate::object::direct::dictionary::Dictionary;
use crate::object::error::ObjectErr;
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

impl DisplayUsingBuffer for ParseErr<'_> {
    fn display_using_buffer(&self, buffer: &[Byte]) -> String {
        match self {
            Self::Recoverable(err) => {
                format!("Parse Recoverable: {}", err.display_using_buffer(buffer))
            }
            Self::Failure(err) => format!("Parse Failure: {}", err.display_using_buffer(buffer)),
        }
    }
}

#[derive(Debug, Error, PartialEq, Clone)]
#[error("{object}. Error: {code}. Buffer: {}", debug_bytes(.buffer))]
pub struct ParseError<'buffer, const RECOVERABLE: bool> {
    pub(crate) buffer: &'buffer [Byte],
    pub(crate) object: &'static str,
    pub(crate) code: ParseErrorCode<'buffer>,
}

impl<const RECOVERABLE: bool> DisplayUsingBuffer for ParseError<'_, RECOVERABLE> {
    fn display_using_buffer(&self, buffer: &[Byte]) -> String {
        format!(
            "{}. Error: {}. Buffer: {}",
            self.object,
            self.code.display_using_buffer(buffer),
            debug_bytes(self.buffer)
        )
    }
}

// Box<_>, DictionaryErr and String do not implement Copy
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseErrorCode<'buffer> {
    #[error("Found Dictionary: {0}")]
    FoundDictionary(Dictionary<'buffer>),
    // Whole buffer errors
    #[error("Buffer is too small")]
    TooSmallBuffer,
    // Whole object errors
    #[error("Wrong object type")]
    WrongObjectType,
    #[error("Not found. Nom: {}", .0.description())]
    NotFound(ErrorKind),
    #[error("Stream data. Nom: {}", .0.description())]
    StreamData(ErrorKind),
    #[error("Missing closing. Nom: {}", .0.description())]
    MissingClosing(ErrorKind),
    // Union Errors
    #[error("Not found")]
    NotFoundUnion,
    // Collection Errors
    #[error("Not found. Parse: {0}")]
    RecNotFound(Box<ParseErrorCode<'buffer>>),
    #[error("Missing subobject: {0}. Parse: {1}")]
    RecMissingSubobject(&'static str, Box<ParseErrorCode<'buffer>>),
    // TODO Replace Vec<Byte> with the object's span
    #[error("Missing value. Key: {}. Parse: {1}", debug_bytes(.0))]
    RecMissingValue(Vec<Byte>, Box<ParseErrorCode<'buffer>>),
    #[error("Missing closing. Parse: {0}")]
    RecMissingClosing(Box<ParseErrorCode<'buffer>>),
    #[error(
        "Entry number {} in subsection {} {}. Parse: {}",
        index,
        first_object_number,
        entry_count,
        code
    )]
    RecSubsectionEntry {
        index: usize,
        first_object_number: ObjectNumberOrZero,
        entry_count: usize,
        code: Box<ParseErrorCode<'buffer>>,
    },
    // TODO Move to NumErrorCode
    #[error("Object number")]
    ObjectNumber, // ObjectNumber::new
    #[error("Generation number")]
    GenerationNumber, // ascii_to_u16
    #[error("First object number")]
    FirstObjectNumber, // ascii_to_u64
    #[error("Offset")]
    Offset, // ascii_to_usize
    #[error("Next free object number")]
    NextFree, // ascii_to_u64
    #[error("Entry type")]
    EntryType, // f or n
    #[error("Entry count")]
    EntryCount, // ascii_to_usize
    #[error("Parse as i128")]
    ParseIntError, // ascii_to_i128
    #[error("Parse as f64")]
    ParseFloatError, // ascii_to_f64
    //
    #[error("Object: {0}")]
    Object(#[from] ObjectErr),
}

impl DisplayUsingBuffer for ParseErrorCode<'_> {
    fn display_using_buffer(&self, buffer: &[Byte]) -> String {
        match self {
            Self::RecNotFound(code) => {
                format!("Not found. Parse: {}", code.display_using_buffer(buffer))
            }
            Self::RecMissingSubobject(key, code) => {
                format!(
                    "Missing subobject: {}. Parse: {}",
                    key,
                    code.display_using_buffer(buffer)
                )
            }
            Self::RecMissingValue(key, code) => {
                format!(
                    "Missing value. Key: {}. Parse: {}",
                    debug_bytes(key),
                    code.display_using_buffer(buffer)
                )
            }
            Self::RecMissingClosing(code) => format!(
                "Missing closing. Parse: {}",
                code.display_using_buffer(buffer)
            ),
            Self::RecSubsectionEntry {
                index,
                first_object_number,
                entry_count,
                code,
            } => format!(
                "Entry number {} in subsection {} {}. Parse: {}",
                index,
                first_object_number,
                entry_count,
                code.display_using_buffer(buffer)
            ),

            Self::Object(err) => err.display_using_buffer(buffer),
            _ => self.to_string(),
        }
    }
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
    use crate::impl_from_ref;

    impl_from_ref!('buffer, ParseFailure<'buffer>, Failure, ParseErr<'buffer>);
    impl_from_ref!('buffer, ParseRecoverable<'buffer>, Recoverable, ParseErr<'buffer>);

    // impl_from_ref!('buffer, ObjectErr<'buffer>, Object, ParseErrorCode<'buffer>);

    impl<'buffer, const RECOVERABLE: bool> ParseError<'buffer, RECOVERABLE> {
        pub fn new(
            buffer: &'buffer [Byte],
            object: &'static str,
            code: ParseErrorCode<'buffer>,
        ) -> Self {
            Self {
                buffer,
                object,
                code,
            }
        }
    }

    impl<'buffer> ParseErr<'buffer> {
        pub fn buffer(&self) -> &'buffer [Byte] {
            match self {
                Self::Recoverable(err) => err.buffer,
                Self::Failure(err) => err.buffer,
            }
        }

        pub fn code(self) -> ParseErrorCode<'buffer> {
            match self {
                Self::Recoverable(err) => err.code,
                Self::Failure(err) => err.code,
            }
        }
    }
}
