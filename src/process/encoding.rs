use ::std::ffi::OsString;

use self::error::EncodingError;
use super::error::ProcessResult;
use crate::Byte;

pub(crate) trait Decoder {
    fn encode(&self, string: &OsString) -> ProcessResult<Vec<Byte>>;

    fn decode(&self, bytes: &[Byte]) -> ProcessResult<OsString>;
}
/// REFERENCE: [7.2 Lexical conventions, p21] and [7.9.2 String object types,
/// p115]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Encoding {
    Ascii,
    PdfDoc,
    Utf8,
    Utf16BE,
    Glyph, // Custom encoding
}

impl Decoder for Encoding {
    fn encode(&self, _string: &OsString) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Encoding::encode")
    }

    /// REFERENCE: [7.3.4 String objects, p25] and [7.9.2 String object types]
    fn decode(&self, bytes: &[Byte]) -> ProcessResult<OsString> {
        match self {
            Self::Ascii => {
                todo!("Implement Encoding::decode for Ascii")
            }
            Self::PdfDoc => {
                todo!("Implement Encoding::decode for PdfDoc")
                // Ok(self.0.iter().map(|&b| char::from(b)).collect())
            }
            Self::Utf8 => {
                // REFERENCE: [7.9.2.2.1 General, p116]
                // let order_marker = [0xEF, 0xBB, 0xBF];
                if let [0xEF, 0xBB, 0xBF, rest @ ..] = bytes {
                    return ::std::str::from_utf8(rest)
                        .map(Into::into)
                        .map_err(Into::into);
                }
                Err(EncodingError::ByteOrderMarker(*self, bytes.into()).into())
            }

            Self::Utf16BE => {
                // REFERENCE: [7.9.2.2.1 General, p116]
                // let order_marker = [0xFE, 0xFF];
                if let [0xFE, 0xFF, _rest @ ..] = bytes {
                    todo!("Implement Encoding::decode for Utf16")
                }
                Err(EncodingError::ByteOrderMarker(*self, bytes.into()).into())
            }
            Self::Glyph => {
                todo!("Implement Encoding::decode for Glyph")
            }
        }
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    use crate::Bytes;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum EncodingError {
        #[error("Wrong byte order marker for {0:?}-encoded buffer {1:?}")]
        ByteOrderMarker(super::Encoding, Bytes),
        #[error("Invalid {0:?}-encoded buffer {1:?}")]
        Invalid(super::Encoding, Bytes),
    }
}
