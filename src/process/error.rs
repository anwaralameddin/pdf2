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
use crate::xref::error::TableError;
use crate::xref::increment::stream::entry::error::EntryError;
use crate::xref::increment::stream::error::XRefStreamError;

pub(crate) type ProcessResult<T> = Result<T, ProcessErr>;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ProcessErr {
    #[error("Encoding: {0}")]
    Encoding(#[from] EncodingError),
    #[error("Escape: {0}")]
    Escape(#[from] EscapeError),
    #[error("Filter: {0}")]
    Filter(#[from] FilterError),
    #[error("Predictor: {0}")]
    Predictor(#[from] PredictorError),
    #[error("JBIG2: {0}")]
    Jbig2(#[from] Jbig2Error),
    #[error("ASCIIHex: {0}")]
    ASCIIHex(#[from] ASCIIHexError),
    #[error("ASCII85: {0}")]
    ASCII85(#[from] ASCII85Error),
    #[error("LZW: {0}")]
    Lzw(#[from] LzwError),
    #[error("Flate: {0}")]
    Flate(#[from] FlateError),
    #[error("Tiff: {0}")]
    Tiff(#[from] TiffError),
    #[error("Png: {0}")]
    Png(#[from] PngError),

    #[error("XRefStream: {0}")]
    XRefStream(#[from] XRefStreamError),
    #[error("Table: {0}")]
    Table(#[from] TableError),
    #[error("Entry: {0}")]
    Entry(#[from] EntryError),

    #[error("Utf8: {0}")]
    Utf8(#[from] Utf8Error),
}
