use ::thiserror::Error;

use super::ascii_85::error::A85ErrorCode;
use super::ascii_hex::error::AHxErrorCode;
use super::flate::error::FlErrorCode;
use super::lzw::error::LzwErrorCode;
use super::predictor::png::error::PngErrorCode;
use super::predictor::tiff::error::TiffErrorCode;
use crate::object::direct::name::Name;
use crate::object::direct::DirectValue;

pub(crate) type FilterResult<'buffer, T> = Result<T, FilterErr<'buffer>>;

#[derive(Debug, Error, PartialEq, Clone)]
#[error("{object}. Error: {code}")]
pub struct FilterErr<'buffer> {
    pub(crate) object: &'static str,
    pub(crate) code: FilterErrorCode<'buffer>,
}

// FlateError does not implement Copy
#[derive(Debug, Error, PartialEq, Clone)]
pub enum FilterErrorCode<'buffer> {
    #[error("Mismatching number of filters {0} and decode parameters {1}")]
    Mismatch(usize, usize),
    // TODO Replace &Name with Span
    #[error("Unsupported. Input: {0}")]
    Unsupported(&'buffer Name<'buffer>),
    #[error("Unsupport Parameter. Input: {0}")]
    UnsupportedParameter(&'buffer i128),
    // TODO Replace &DirectValue with Span
    #[error("Wrong value type. Expected {0}. Input {1}")]
    ValueType(&'static str, &'buffer DirectValue<'buffer>),
    #[error("Missing required parameter: {0}")]
    MissingRequiredParameter(&'static str),
    //
    #[error("Tiff: {0}")]
    Tiff(TiffErrorCode),
    #[error("Png: {0}")]
    Png(PngErrorCode),
    #[error("ASCII85: {0}")]
    A85(A85ErrorCode),
    #[error("ASCIIHex: {0}")]
    AHx(AHxErrorCode),
    #[error("LZW: {0}")]
    Lzw(LzwErrorCode),
    #[error("Flate: {0}")]
    Fl(FlErrorCode),
}

mod convert {
    use super::*;
    impl<'buffer> FilterErr<'buffer> {
        pub fn new(object: &'static str, code: FilterErrorCode<'buffer>) -> Self {
            Self { object, code }
        }
    }

    macro_rules! filter_err_from {
        ($from:ty, $object:ident) => {
            impl From<$from> for FilterErr<'_> {
                fn from(code: $from) -> Self {
                    Self::new(stringify!($object), FilterErrorCode::$object(code))
                }
            }
        };
    }

    filter_err_from!(TiffErrorCode, Tiff);
    filter_err_from!(PngErrorCode, Png);
    filter_err_from!(A85ErrorCode, A85);
    filter_err_from!(AHxErrorCode, AHx);
    filter_err_from!(LzwErrorCode, Lzw);
    filter_err_from!(FlErrorCode, Fl);
}
