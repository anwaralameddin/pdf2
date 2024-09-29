pub(crate) mod error;

use ::std::ffi::OsString;
use ::std::str::from_utf8;

use self::error::EncodingErr;
use self::error::EncodingErrorCode;
use self::error::EncodingResult;
use crate::Byte;

pub(crate) trait Decoder<'buffer> {
    fn encode(&self, string: &'buffer OsString) -> EncodingResult<'buffer, Vec<Byte>>;

    fn decode(
        &self,
        transfer: impl Fn(&'buffer [Byte]) -> EncodingResult<'buffer, Vec<Byte>>,
        bytes: &'buffer [Byte],
    ) -> EncodingResult<'buffer, OsString>;
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

impl<'buffer> Decoder<'buffer> for Encoding {
    fn encode(&self, _string: &'buffer OsString) -> EncodingResult<'buffer, Vec<Byte>> {
        todo!("Implement Encoding::encode")
    }

    /// REFERENCE: [7.3.4 String objects, p25] and [7.9.2 String object types]
    fn decode(
        &self,
        transfer: impl Fn(&'buffer [Byte]) -> EncodingResult<'buffer, Vec<Byte>>,
        bytes: &'buffer [Byte],
    ) -> EncodingResult<'buffer, OsString> {
        let transfered_bytes = transfer(bytes)?;
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
                if let [0xEF, 0xBB, 0xBF, rest @ ..] = transfered_bytes.as_slice() {
                    return from_utf8(rest)
                        .map(Into::into)
                        .map_err(|err| EncodingErr::new(bytes, EncodingErrorCode::Utf8(err)));
                }
                Err(EncodingErr::new(
                    bytes,
                    EncodingErrorCode::ByteOrderMarker(*self),
                ))
            }

            Self::Utf16BE => {
                // REFERENCE: [7.9.2.2.1 General, p116]
                // let order_marker = [0xFE, 0xFF];
                if let [0xFE, 0xFF, _rest @ ..] = transfered_bytes.as_slice() {
                    todo!("Implement Encoding::decode for Utf16")
                }
                Err(EncodingErr::new(
                    bytes,
                    EncodingErrorCode::ByteOrderMarker(*self),
                ))
            }
            Self::Glyph => {
                todo!("Implement Encoding::decode for Glyph")
            }
        }
    }
}
