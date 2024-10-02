use ::std::str::Utf8Error;
use ::thiserror::Error;

use super::Encoding;
use crate::fmt::debug_bytes;
use crate::process::escape::error::EscapeErrorCode;
use crate::process::filter::error::FilterErr;
use crate::Byte;

pub(crate) type EncodingResult<'buffer, T> = Result<T, EncodingErr<'buffer>>;

#[derive(Debug, Error, PartialEq, Clone)]
#[error("Escape. Error: {code}. Buffer: {}", debug_bytes(.buffer))]
pub struct EncodingErr<'buffer> {
    buffer: &'buffer [Byte],
    code: EncodingErrorCode,
}

// FilterErrorCode and EscapeErrorCode do not implement Copy
#[derive(Debug, Error, PartialEq, Clone)]
pub enum EncodingErrorCode {
    #[error("Wrong byte order marker for {0:?}-encoded")]
    ByteOrderMarker(Encoding),
    #[error("Invalid {0:?}-encoded")]
    Invalid(Encoding),
    //
    #[error("Escape. Error {0}")]
    Escape(#[from] EscapeErrorCode),
    #[error("Filter: {0}")]
    Filter(#[from] FilterErr),
    #[error("Utf8: {0}")]
    Utf8(#[from] Utf8Error),
}

mod convert {
    use super::*;
    use crate::process::escape::error::EscapeErr;

    impl<'buffer> EncodingErr<'buffer> {
        pub fn new(buffer: &'buffer [Byte], code: EncodingErrorCode) -> Self {
            Self { buffer, code }
        }
    }

    impl<'buffer> From<EscapeErr<'buffer>> for EncodingErr<'buffer> {
        fn from(err: EscapeErr<'buffer>) -> Self {
            Self {
                buffer: err.buffer,
                code: EncodingErrorCode::Escape(err.code),
            }
        }
    }
}
