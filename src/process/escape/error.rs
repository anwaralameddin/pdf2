use ::thiserror::Error;

use crate::fmt::debug_bytes;
use crate::process::filter::error::FilterErrorCode;
use crate::Byte;

pub(crate) type EscapeResult<'buffer, T> = Result<T, EscapeErr<'buffer>>;

#[derive(Debug, Error, PartialEq, Clone)]
#[error("Escape. Error {code}. Buffer: {}", debug_bytes(.buffer))]
pub struct EscapeErr<'buffer> {
    pub(crate) buffer: &'buffer [Byte],
    pub(crate) code: EscapeErrorCode,
}

// FilterErrorCode does not implement Copy
#[derive(Debug, Error, PartialEq, Clone)]
pub enum EscapeErrorCode {
    #[error("Hexadecimal string. Error: {0}")]
    Hexadecimal(FilterErrorCode),
    #[error("Name. A non hexadecimal character following the number sign:  {0}")]
    InvalidHexDigit(char),
    #[error("Name. Incomplete hexadecimal code: #{0:02X}. Followed by: {1}")]
    IncompleteHexCode(Byte, char),
    #[error("Name. Trailing number sign")]
    TraillingNumberSign,
    #[error("Name. Trailing hexadecimal digit: #{0:02X}")]
    TraillingHexDigit(Byte),
}

mod convert {
    use super::*;

    impl<'buffer> EscapeErr<'buffer> {
        pub fn new(buffer: &'buffer [Byte], code: EscapeErrorCode) -> Self {
            Self { buffer, code }
        }
    }
}
