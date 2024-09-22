use ::nom::error::ErrorKind;
use ::std::str::Utf8Error;
use ::thiserror::Error;

use crate::fmt::debug_bytes;
use crate::object::direct::array::error::ArrayFailure;
use crate::object::direct::array::error::ArrayRecoverable;
use crate::object::direct::boolean::error::BooleanRecoverable;
use crate::object::direct::dictionary::error::DataTypeError;
use crate::object::direct::dictionary::error::DictionaryFailure;
use crate::object::direct::dictionary::error::DictionaryRecoverable;
use crate::object::direct::error::DirectValueRecoverable;
use crate::object::direct::name::error::NameRecoverable;
use crate::object::direct::null::error::NullRecoverable;
use crate::object::direct::numeric::error::NumericRecoverable;
use crate::object::direct::numeric::integer::error::IntegerRecoverable;
use crate::object::direct::numeric::real::error::RealFailure;
use crate::object::direct::numeric::real::error::RealRecoverable;
use crate::object::direct::string::error::StringRecoverable;
use crate::object::direct::string::hexadecimal::error::HexadecimalFailure;
use crate::object::direct::string::hexadecimal::error::HexadecimalRecoverable;
use crate::object::direct::string::literal::error::LiteralFailure;
use crate::object::direct::string::literal::error::LiteralRecoverable;
use crate::object::indirect::error::IndirectValueRecoverable;
use crate::object::indirect::id::error::IdRecoverable;
use crate::object::indirect::object::error::IndirectObjectFailure;
use crate::object::indirect::object::error::IndirectObjectRecoverable;
use crate::object::indirect::reference::error::ReferenceRecoverable;
use crate::object::indirect::stream::error::StreamFailure;
use crate::object::indirect::stream::error::StreamRecoverable;
use crate::xref::increment::error::IncrementCode;
use crate::xref::increment::section::entry::error::EntryCode;
use crate::xref::increment::section::error::SectionCode;
use crate::xref::increment::section::subsection::error::SubsectionCode;
use crate::xref::pretable::error::PreTableCode;
use crate::xref::startxref::error::StartXRefCode;
use crate::Byte;

pub(crate) type NewParseResult<'buffer, T> = Result<T, NewParseErr<'buffer>>;
pub(crate) type ParseResult<T> = Result<T, ParseErr>;
/// Recoverable parsing error
/// This error is used when the parser is unable to determine the value type
/// and the buffer needs to be reprocessed with a different parser
pub(crate) type NewParseRecoverable<'buffer> = ParseError<'buffer, true>;
/// Unrecoverable parsing error
/// This error is used when the parser is able to determine the value type
/// but fails to parse it completely
pub(crate) type NewParseFailure<'buffer> = ParseError<'buffer, false>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum NewParseErr<'buffer> {
    #[error("Parse Recoverable: {0}")]
    Recoverable(NewParseRecoverable<'buffer>),
    #[error("Parse Failure: {0}")]
    Failure(NewParseFailure<'buffer>),
    // TODO(TEMP)
    #[error("Old Parse Error: {0}")]
    Old(#[from] ParseErr),
}

#[derive(Debug, Error, PartialEq, Clone)]
#[error("Code: {code}. Buffer: {}", debug_bytes(.buffer))]
pub struct ParseError<'buffer, const RECOVERABLE: bool> {
    pub(crate) buffer: &'buffer [Byte],
    pub(crate) code: ParseErrorCode,
}

// XRefStreamCode, SubsectionCode and SectionCode do not implement Copy
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseErrorCode {
    #[error("{0}. Not found. {}", if let Some(kind) = .1 { kind.description() } else { "" })]
    NotFound(&'static str, Option<ErrorKind>),
    #[error("{0}. Missing closing. Error: {}", .1.description() )]
    MissingClosing(&'static str, ErrorKind),
    #[error("{0}. Offset")]
    OffSet(&'static str),
    #[error("Increment. {0}")]
    Increment(#[from] IncrementCode),
    #[error("StartXRef. {0}")]
    StartXRef(#[from] StartXRefCode),
    #[error("PreTable. {0}")]
    PreTable(#[from] PreTableCode),
    #[error("Entry. {0}")]
    Entry(#[from] EntryCode),
    #[error("Subsection. {0}")]
    Subsection(#[from] SubsectionCode),
    #[error("Section. {0}")]
    Section(#[from] SectionCode),
}

// TODO (TEMP)
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseErr {
    #[error("Parse Recoverable: {0}")]
    Error(#[from] ParseRecoverable),
    #[error("Parse Failure: {0}")]
    Failure(#[from] ParseFailure),
}

// TODO (TEMP)
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseRecoverable {
    #[error("Null: {0}")]
    Null(#[from] NullRecoverable),
    #[error("Boolean: {0}")]
    Boolean(#[from] BooleanRecoverable),
    #[error("Name: {0}")]
    Name(#[from] NameRecoverable),
    #[error("Integer: {0}")]
    Integer(#[from] IntegerRecoverable),
    #[error("Real: {0}")]
    Real(#[from] RealRecoverable),
    #[error("Numeric: {0}")]
    Numeric(#[from] NumericRecoverable),
    #[error("Hexadecimal: {0}")]
    Hexadecimal(#[from] HexadecimalRecoverable),
    #[error("Literal: {0}")]
    Literal(#[from] LiteralRecoverable),
    #[error("String: {0}")]
    String(#[from] StringRecoverable),
    #[error("Array: {0}")]
    Array(#[from] ArrayRecoverable),
    #[error("Dictionary: {0}")]
    Dictionary(#[from] DictionaryRecoverable),
    #[error("Direct Value: {0}")]
    DirectValue(#[from] DirectValueRecoverable),
    #[error("Id: {0}")]
    Id(#[from] IdRecoverable),
    #[error("Reference: {0}")]
    Reference(#[from] ReferenceRecoverable),
    #[error("Indirect Object: {0}")]
    IndirectObject(#[from] IndirectObjectRecoverable),
    #[error("Indirect Value: {0}")]
    IndirectValue(#[from] IndirectValueRecoverable),
    #[error("Stream: {0}")]
    Stream(#[from] StreamRecoverable),
}

// TODO (TEMP)
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseFailure {
    // TODO Remove Utf8Error from ParseFailure
    // Utf8Error should not happen, as ::std::str::from_utf8 is only called
    // for bytes that are already validated as UTF-8
    #[error("Utf8Error: Failed to parse as a string slice: {1}. Buffer: {0}")]
    Utf8Error(String, Utf8Error),
    #[error("Real: {0}")]
    Real(#[from] RealFailure),
    #[error("Hexadecimal: {0}")]
    Hexadecimal(#[from] HexadecimalFailure),
    #[error("Literal: {0}")]
    Literal(#[from] LiteralFailure),
    #[error("Array: {0}")]
    Array(#[from] ArrayFailure),
    #[error("Dictionary: {0}")]
    Dictionary(#[from] DictionaryFailure),
    #[error("Indirect Object: {0}")]
    IndirectObject(#[from] IndirectObjectFailure),
    #[error("Stream: {0}")]
    Stream(#[from] StreamFailure),
    // TODO Refactor if DataTypeErr is only used in ParseFailure
    #[error("DataType: {0}")]
    DataType(#[from] DataTypeError<'static>),
}

impl NewParseErr<'_> {
    pub fn code(&self) -> &ParseErrorCode {
        match self {
            Self::Recoverable(err) => &err.code,
            Self::Failure(err) => &err.code,
            Self::Old(_) => unimplemented!(),
        }
    }
}

// TODO(TEMP)
impl ParseErr {
    pub fn code(&self) -> &ParseErrorCode {
        unimplemented!()
    }
}

#[macro_export]
macro_rules! new_parse_failure {
    ($e:ident, $failure:expr) => {
        |err| match err {
            NomErr::Incomplete(_) => unreachable!(
                "::nom::complete functions do not return the Incomplete error variant."
            ),
            NomErr::Error($e) | NomErr::Failure($e) => NewParseErr::Failure($failure.into()),
        }
    };
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
            NomErr::Error($e) | NomErr::Failure($e) => NewParseErr::Recoverable($error),
        }
    };
}

#[macro_export]
macro_rules! parse_error {
    ($e:ident, $error:expr) => {
        |err| match err {
            NomErr::Incomplete(_) => unreachable!(
                "::nom::complete functions do not return the Incomplete error variant."
            ),
            NomErr::Error($e) | NomErr::Failure($e) => ParseErr::Error($error.into()),
        }
    };
}

mod convert {
    use super::*;

    // TODO(TEMP)
    impl<'buffer> From<ParseFailure> for NewParseErr<'buffer> {
        fn from(err: ParseFailure) -> Self {
            NewParseErr::Old(err.into())
        }
    }

    impl<'buffer> From<NewParseFailure<'buffer>> for NewParseErr<'buffer> {
        fn from(err: NewParseFailure<'buffer>) -> Self {
            NewParseErr::Failure(err)
        }
    }

    impl<'buffer> From<NewParseRecoverable<'buffer>> for NewParseErr<'buffer> {
        fn from(err: NewParseRecoverable<'buffer>) -> Self {
            NewParseErr::Recoverable(err)
        }
    }
}
