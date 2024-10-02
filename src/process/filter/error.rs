use ::thiserror::Error;

use super::ascii_85::error::A85ErrorCode;
use super::ascii_hex::error::AHxErrorCode;
use super::flate::error::FlErrorCode;
use super::lzw::error::LzwErrorCode;
use super::predictor::png::error::PngErrorCode;
use super::predictor::tiff::error::TiffErrorCode;
use crate::error::DisplayUsingBuffer;
use crate::fmt::debug_bytes;
use crate::parse::Span;

pub(crate) type FilterResult<T> = Result<T, FilterErr>;

#[derive(Debug, Error, PartialEq, Clone)]
#[error("{object}. Error: {code}")]
pub struct FilterErr {
    pub(crate) object: &'static str,
    pub(crate) code: FilterErrorCode,
}

impl DisplayUsingBuffer for FilterErr {
    fn display_using_buffer(&self, buffer: &[u8]) -> String {
        format!(
            "{}. Error: {}",
            self.object,
            self.code.display_using_buffer(buffer)
        )
    }
}

// FlateError does not implement Copy
#[derive(Debug, Error, PartialEq, Clone)]
pub enum FilterErrorCode {
    #[error("Mismatching number of filters {0} and decode parameters {1}")]
    Mismatch(usize, usize),
    // TODO Replace &Name with Span
    #[error("Unsupported. Found: {0}")]
    Unsupported(Span), // &'buffer Name<'buffer>),
    #[error("Unsupport Parameter. Found: {0}")]
    UnsupportedParameter(i128),
    // TODO Replace &DirectValue with Span
    #[error("Wrong value type. Expected {0}. Found: {1}")]
    ValueType(&'static str, Span), // &'buffer DirectValue<'buffer>),
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

impl DisplayUsingBuffer for FilterErrorCode {
    fn display_using_buffer(&self, buffer: &[u8]) -> String {
        match self {
            FilterErrorCode::Unsupported(span) => {
                format!("Unsupported. Found: {}", debug_bytes(&buffer[*span]))
            }
            FilterErrorCode::ValueType(_, span) => {
                format!(
                    "Wrong value type. Expected: {}. Found: {}",
                    self,
                    debug_bytes(&buffer[*span])
                )
            }
            _ => self.to_string(),
        }
    }
}

mod convert {
    use super::*;
    impl FilterErr {
        pub fn new(object: &'static str, code: FilterErrorCode) -> Self {
            Self { object, code }
        }
    }

    macro_rules! filter_err_from {
        ($from:ty, $object:ident) => {
            impl From<$from> for FilterErr {
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
