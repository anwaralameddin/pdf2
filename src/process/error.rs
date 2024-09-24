use ::std::str::Utf8Error;
use ::thiserror::Error;

use super::encoding::error::EncodingError;
use super::escape::error::EscapeError;
use super::filter::ascii_85::error::ASCII85Error;
use super::filter::ascii_hex::error::ASCIIHexError;
use super::filter::error::FilterError;
use super::filter::flate::error::FlateError;
use super::filter::jbig2::error::Jbig2Error;
use super::filter::lzw::error::LzwError;
use super::filter::predictor::error::PredictorError;
use super::filter::predictor::png::error::PngError;
use super::filter::predictor::tiff::error::TiffError;
use crate::object::direct::dictionary::error::DataTypeError;
use crate::object::direct::dictionary::error::MissingEntryError;
use crate::xref::error::XRefError;
use crate::xref::increment::error::IncrementError;
use crate::xref::increment::stream::entry::error::EntryError;
use crate::xref::increment::stream::error::XRefStreamError;

pub(crate) type NewProcessResult<T> = Result<T, NewProcessErr>;
pub(crate) type ProcessResult<T> = Result<T, ProcessErr>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum NewProcessErr {
    // TODO (TEMP) Remove this error variant after refactoring ProcessErr
    #[error("Old: {0}")]
    Old(#[from] ProcessErr),
    //
    #[error("DataType: {0}")]
    DataType(#[from] DataTypeError<'static>),
    #[error("MissingEntry: {0}")]
    MissingEntry(#[from] MissingEntryError),
    #[error("Entry: {0}")]
    Entry(#[from] EntryError),
    #[error("XRefStream: {0}")]
    XRefStream(#[from] XRefStreamError),
    #[error("Increment: {0}")]
    Increment(#[from] IncrementError),
    #[error("XRef: {0}")]
    XRef(#[from] XRefError),
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ProcessErr {
    #[error("Encoding: {0}")]
    Encoding(#[from] EncodingError),
    #[error("Escape: {0}")]
    Escape(#[from] EscapeError),
    // TODO (TEMP) Remove this error variant after refactoring escape and encoding
    #[error("Utf8: {0}")]
    Utf8(#[from] Utf8Error),
    // Filter errors
    #[error("Predictor: {0}")]
    Predictor(#[from] PredictorError),
    #[error("Tiff: {0}")]
    Tiff(#[from] TiffError),
    #[error("Png: {0}")]
    Png(#[from] PngError),
    #[error("Filter: {0}")]
    Filter(#[from] FilterError),
    #[error("ASCIIHex: {0}")]
    ASCIIHex(#[from] ASCIIHexError),
    #[error("ASCII85: {0}")]
    ASCII85(#[from] ASCII85Error),
    #[error("LZW: {0}")]
    Lzw(#[from] LzwError),
    #[error("Flate: {0}")]
    Flate(#[from] FlateError),
    #[error("JBIG2: {0}")]
    Jbig2(#[from] Jbig2Error),
}

#[macro_export]
macro_rules! process_err {
    ($e:ident, $error:expr) => {
        |err| match err {
            NomErr::Incomplete(_) => unreachable!(
                "::nom::complete functions do not return the Incomplete error variant."
            ),
            NomErr::Error($e) | NomErr::Failure($e) => $error,
        }
    };
}
