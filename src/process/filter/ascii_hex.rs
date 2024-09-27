use self::error::AHxErrorCode;
use super::Filter;
use crate::parse::character_set::is_white_space;
use crate::parse::num::hex_val;
use crate::process::filter::error::FilterResult;
use crate::Byte;

/// ASCII hexadecimal filter.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct AHx;

impl<'buffer> Filter<'buffer> for AHx {
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        let encoded = bytes
            .as_ref()
            .iter()
            .flat_map(|&byte| {
                let high = byte >> 4;
                let low = byte & 0x0F;
                vec![
                    if high < 10 {
                        b'0' + high
                    } else {
                        b'A' + high - 10
                    },
                    if low < 10 {
                        b'0' + low
                    } else {
                        b'A' + low - 10
                    },
                ]
            })
            .collect();
        Ok(encoded)
    }

    /// REFERENCE: [7.4.2 ASCIIHexDecode filter, p37]
    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        let bytes = bytes.as_ref();
        let mut defiltered = Vec::with_capacity(bytes.len() / 2 + 1);
        let mut eod = false;
        let mut prev = None;
        // TODO(QUESTION): Is the EOD required?
        for &byte in bytes.iter() {
            if is_white_space(byte) {
                continue;
            }
            if eod {
                return Err(AHxErrorCode::AfterEod(char::from(byte)).into());
            }
            if byte == b'>' {
                eod = true;
                continue;
            }
            if let Some(a) = prev {
                defiltered.push(
                    hex_val(a).ok_or_else(|| AHxErrorCode::InvalidHexDigit(char::from(a)))? << 4
                        | hex_val(byte)
                            .ok_or_else(|| AHxErrorCode::InvalidHexDigit(char::from(byte)))?,
                );

                prev = None;
            } else {
                prev = Some(byte);
            }
        }
        if let Some(a) = prev {
            defiltered.push(hex_val(a).ok_or_else(|| AHxErrorCode::AfterEod(char::from(a)))? << 4);
        }

        Ok(defiltered)
    }
}

pub(in crate::process::filter) mod error {
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum AHxErrorCode {
        #[error("Invalid ASCII hexadecimal digit: {0}")]
        InvalidHexDigit(char),
        #[error("Unexpected character after the EOD marker: {0}")]
        AfterEod(char),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assert_err_eq;

    #[test]
    fn ascii_hex_valid() {
        // Synthetic tests

        let filtered = b"412048657861646563696D616C20537472696E67";
        let defiltered = AHx.defilter(filtered).unwrap();
        let expected = b"A Hexadecimal String";
        assert_eq!(defiltered, expected);
        let refiltered = AHx.filter(defiltered).unwrap();
        assert_eq!(refiltered, filtered);

        let filtered = b"41 20 48";
        let defiltered = AHx.defilter(filtered).unwrap();
        let expected = b"\x41\x20\x48";
        assert_eq!(defiltered, expected);
        let refiltered = AHx.filter(defiltered).unwrap();
        let filtered = filtered
            .iter()
            .filter(|&&b| !is_white_space(b))
            .cloned()
            .collect::<Vec<Byte>>();
        assert_eq!(refiltered, filtered);

        let filtered = b"41 20 4";
        let defiltered = AHx.defilter(filtered).unwrap();
        let expected = b"\x41\x20\x40";
        assert_eq!(defiltered, expected);
        let refiltered = AHx.filter(defiltered).unwrap();
        let filtered = filtered
            .iter()
            .filter(|&&b| !is_white_space(b))
            .cloned()
            .collect::<Vec<Byte>>();
        assert_eq!(refiltered[..5], filtered);
        assert_eq!(refiltered[5], b'0');

        let filtered = b"41 20 4> ";
        let defiltered = AHx.defilter(filtered).unwrap();
        let expected = b"\x41\x20\x40";
        assert_eq!(defiltered, expected);
    }

    #[test]
    fn ascii_hex_invalid() {
        // Synthetic tests

        // Invalid ASCII hexadecimal digit
        let filtered = b"41204X";
        let defiltered_result = AHx.defilter(filtered);
        let expected_error = AHxErrorCode::InvalidHexDigit('X');
        assert_err_eq!(defiltered_result, expected_error);

        // Unexpected character after the EOD marker
        let filtered = b"41204>1";
        let defiltered_result = AHx.defilter(filtered);
        let expected_error = AHxErrorCode::AfterEod('1');
        assert_err_eq!(defiltered_result, expected_error);
    }
}
