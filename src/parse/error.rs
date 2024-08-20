use ::nom::Needed;
use ::std::str::Utf8Error;
use ::thiserror::Error;

use crate::object::direct::array::error::ArrayFailure;
use crate::object::direct::array::error::ArrayRecoverable;
use crate::object::direct::boolean::error::BooleanRecoverable;
use crate::object::direct::dictionary::error::DataTypeErr;
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
use crate::xref::increment::error::IncrementRecoverable;
use crate::xref::increment::section::entry::error::EntryFailure;
use crate::xref::increment::section::error::SectionFailure;
use crate::xref::increment::section::error::SectionRecoverable;
use crate::xref::increment::section::subsection::error::SubsectionFailure;
use crate::xref::increment::section::subsection::error::SubsectionRecoverable;
use crate::xref::increment::trailer::error::TrailerFailure;
use crate::xref::pretable::error::PreTableFailure;
use crate::xref::startxref::error::StartXRefFailure;

pub(crate) type ParseResult<T> = Result<T, ParseErr>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseErr {
    #[error("Parse Recoverable: {0}")]
    Error(#[from] ParseRecoverable),
    #[error("Parse Failure: {0}")]
    Failure(#[from] ParseFailure),
}

/// Recoverable parsing error
/// This error is used when the parser is unable to determine the value type
/// and the buffer needs to be reprocessed with a different parser
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseRecoverable {
    #[error("Incomplete: {0:?}")]
    Incomplete(Needed),

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
    #[error("Subsection: {0}")]
    Subsection(#[from] SubsectionRecoverable),
    #[error("Section: {0}")]
    Section(#[from] SectionRecoverable),
    #[error("Increment: {0}")]
    Increment(#[from] IncrementRecoverable),
}

/// Unrecoverable parsing error
/// This error is used when the parser is able to determine the value type
/// but fails to parse it completely
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseFailure {
    #[error("Incomplete: {0:?}")]
    Incomplete(Needed),

    #[error("StartXRef: {0}")]
    StartXRef(#[from] StartXRefFailure),
    // TODO Remove Utf8Error from ParseFailure
    // Utf8Error should not happen, as ::std::str::from_utf8 is only called
    // for bytes that are already validated as UTF-8
    #[error("Utf8Error: Failed to parse as a string slice: {1}. Input: {0}")]
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
    #[error("Subsection: {0}")]
    Subsection(#[from] SubsectionFailure),
    #[error("Section: {0}")]
    Section(#[from] SectionFailure),
    #[error("Entry: {0}")]
    Entry(#[from] EntryFailure),
    #[error("Trailer: {0}")]
    Trailer(#[from] TrailerFailure),
    // TODO Refactor if DataTypeErr is only used in ParseFailure
    #[error("DataType: {0}")]
    DataType(#[from] DataTypeErr),
    #[error("PreTable: {0}")]
    PreTable(#[from] PreTableFailure),
}

#[macro_export]
macro_rules! parse_error {
    ($e:ident, $error:expr) => {
        |err| match err {
            NomErr::Incomplete(needed) => ParseRecoverable::Incomplete(needed).into(),
            NomErr::Error($e) | NomErr::Failure($e) => ParseErr::Error($error.into()),
        }
    };
}

#[macro_export]
macro_rules! parse_failure {
    ($e:ident, $failure:expr) => {
        |err| match err {
            NomErr::Incomplete(needed) => ParseFailure::Incomplete(needed).into(),
            NomErr::Error($e) | NomErr::Failure($e) => ParseErr::Failure($failure.into()),
        }
    };
}
