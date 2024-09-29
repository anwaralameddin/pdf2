use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmrResult;

use self::error::A85ErrorCode;
use super::Filter;
use crate::parse::character_set::is_white_space;
use crate::process::filter::error::FilterResult;
use crate::Byte;

/// ASCII base-85 filter.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct A85;

impl<'buffer> Filter<'buffer> for A85 {
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        let bytes = bytes.as_ref();
        let mut encoded = Vec::with_capacity(bytes.len() * 5 / 4 + 1);
        let mut prev = Bytes::<4>::default();
        for &byte in bytes.iter() {
            let value = if let Some(data) = prev.push(byte) {
                u32::from_be_bytes(data)
            } else {
                continue;
            };

            if value == 0 {
                encoded.push(b'z');
            } else {
                for i in (0u32..5).rev() {
                    encoded.push((value / 85u32.pow(i) % 85 + u32::from(b'!')) as Byte);
                }
            }
        }
        if prev.pos != 0 {
            let value = u32::from_be_bytes(prev.data);
            for i in (0u32..5).rev().take(prev.pos + 1) {
                encoded.push((value / 85u32.pow(i) % 85 + u32::from(b'!')) as Byte);
            }
        }
        // Add the EOD marker
        encoded.extend(b"~>");
        Ok(encoded)
    }

    /// REFERENCE: [7.4.3 ASCII85Decode filter, p37-39]
    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        let bytes = bytes.as_ref();
        let mut defiltered = Vec::with_capacity(bytes.len() * 4 / 5 + 1);
        let mut prev = Bytes::<5>::default();
        let mut tilde = false;
        let mut eod = false;
        for &byte in bytes.iter() {
            if is_white_space(byte) {
                continue;
            }
            if eod {
                return Err(A85ErrorCode::AfterEod(char::from(byte)).into());
            }
            if tilde {
                if byte == b'>' {
                    eod = true;
                    continue;
                } else {
                    return Err(A85ErrorCode::CorruptEod(char::from(byte)).into());
                }
            }
            if byte == b'~' {
                tilde = true;
                continue;
            }
            if byte == b'z' {
                if prev.pos == 0 {
                    defiltered.extend(&[0; 4]);
                    continue;
                } else {
                    return Err(A85ErrorCode::ZInMiddle(prev).into());
                }
            }
            if !(b'!'..=b'u').contains(&byte) {
                return Err(A85ErrorCode::InvalidBase85Digit(char::from(byte)).into());
            }
            let data = if let Some(data) = prev.push(byte) {
                data
            } else {
                continue;
            };
            // TODO Consider a more efficient algorithm
            let mut value = 0u32;
            for &byte in data.iter() {
                value = value
                    .checked_mul(85)
                    .ok_or(A85ErrorCode::ValueTooLarge(prev))?
                    .checked_add(u32::from(byte - b'!'))
                    .ok_or(A85ErrorCode::ValueTooLarge(prev))?;
            }
            defiltered.extend_from_slice(&value.to_be_bytes());
        }
        if prev.pos != 0 {
            if prev.pos == 1 {
                return Err(A85ErrorCode::FinalPartialGroup(prev).into());
            }
            let mut value = 0u32;

            for &byte in prev.data.iter().chain(&vec![b'u'; 5 - prev.pos]) {
                value = value
                    .checked_mul(85)
                    .ok_or(A85ErrorCode::ValueTooLarge(prev))?
                    .checked_add(u32::from(byte - b'!'))
                    .ok_or(A85ErrorCode::ValueTooLarge(prev))?;
            }
            defiltered.extend_from_slice(&value.to_be_bytes()[..prev.pos - 1]);
        }

        Ok(defiltered)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bytes<const N: usize> {
    data: [Byte; N],
    pos: usize,
}

impl<const N: usize> Default for Bytes<N> {
    fn default() -> Self {
        Self {
            data: [0; N],
            pos: 0,
        }
    }
}

impl<const N: usize> Display for Bytes<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmrResult {
        for &byte in self.data.iter().take(self.pos) {
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}

impl<const N: usize> Bytes<N> {
    fn push(&mut self, byte: Byte) -> Option<[Byte; N]> {
        if let Some(b) = self.data.get_mut(self.pos) {
            *b = byte;
            self.pos += 1;
            None
        } else {
            let data = self.data;
            self.data = [0; N];
            self.data[0] = byte;
            self.pos = 1;
            Some(data)
        }
    }
}

pub(in crate::process::filter) mod error {
    use ::thiserror::Error;

    use super::Bytes;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum A85ErrorCode {
        #[error("Invalid ASCII base-85 digit: {0}")]
        InvalidBase85Digit(char),
        #[error("Unexpected character after the EOD marker: {0}")]
        AfterEod(char),
        #[error("Value is is greater than 2^32 - 1. {0}")]
        ValueTooLarge(Bytes<5>),
        #[error("A z character occurs in the middle of a group: {0}")]
        ZInMiddle(Bytes<5>),
        #[error("A final partial group contains only one character: {0}")]
        FinalPartialGroup(Bytes<5>),
        #[error("Corrupt EOD marker. Expected '>' after '~'. Found: {0}")]
        CorruptEod(char),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_err_eq;

    #[test]
    fn ascii_85_valid() {
        // Synthetic tests

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(~>";
        let defiltered = A85.defilter(filtered).unwrap();
        let expected = b"A Hexadecimal String";
        assert_eq!(defiltered, expected);
        let refiltered = A85.filter(defiltered).unwrap();
        let filtered = filtered
            .iter()
            .filter(|&&byte| !is_white_space(byte))
            .copied()
            .collect::<Vec<Byte>>();
        assert_eq!(refiltered, filtered);

        // TODO Add tests
    }

    #[test]
    fn ascii_85_invalid() {
        // Synthetic tests

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(v~>";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = A85ErrorCode::InvalidBase85Digit('v');
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(~> a";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = A85ErrorCode::AfterEod('a');
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0 G[Y,o @qfdg C`lYu EbT/a uuuuu~>";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = A85ErrorCode::ValueTooLarge(Bytes {
            data: *b"uuuuu",
            pos: 5,
        });
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(/cz~>";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = A85ErrorCode::ZInMiddle(Bytes {
            data: *b"/c\0\0\0",
            pos: 2,
        });
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(/";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = A85ErrorCode::FinalPartialGroup(Bytes {
            data: *b"/\0\0\0\0",
            pos: 1,
        });
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(~ a";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = A85ErrorCode::CorruptEod('a');
        assert_err_eq!(defiltered_result, expected_error);
    }
}
