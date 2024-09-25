use self::error::ASCII85Error;
use super::Filter;
use crate::fmt::debug_bytes;
use crate::parse::character_set::is_white_space;
use crate::process::error::ProcessResult;
use crate::Byte;

/// ASCII base-85 filter.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct A85;

impl Filter for A85 {
    fn filter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        let bytes = bytes.as_ref();
        let mut encoded = Vec::with_capacity(bytes.len() * 5 / 4 + 1);
        let mut prev = vec![];
        for &byte in bytes.iter() {
            prev.push(byte);
            if prev.len() < 4 {
                continue;
            }
            let mut value = 0u32;
            for &byte in prev.iter() {
                value = (value << 8).checked_add(u32::from(byte)).ok_or(
                    ASCII85Error::ValueTooLarge(stringify!(Byte), debug_bytes(&prev)),
                )?;
            }
            if value == 0 {
                encoded.push(b'z');
            } else {
                for i in (0u32..5).rev() {
                    encoded.push((value / 85u32.pow(i) % 85 + u32::from(b'!')) as Byte);
                }
            }
            prev.clear();
        }
        if !prev.is_empty() {
            let mut value = 0u32;
            for &byte in prev.iter().chain(&vec![0; 4 - prev.len()]) {
                value = (value << 8).checked_add(u32::from(byte)).ok_or(
                    ASCII85Error::ValueTooLarge(stringify!(Byte), debug_bytes(&prev)),
                )?;
            }
            for i in (0u32..5).rev().take(prev.len() + 1) {
                encoded.push((value / 85u32.pow(i) % 85 + u32::from(b'!')) as Byte);
            }
        }
        // Add the EOD marker
        encoded.extend(b"~>");
        Ok(encoded)
    }

    /// REFERENCE: [7.4.3 ASCII85Decode filter, p37-39]
    fn defilter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        let bytes = bytes.as_ref();
        let mut defiltered = Vec::with_capacity(bytes.len() * 4 / 5 + 1);
        let mut prev = vec![];
        let mut tilde = false;
        let mut eod = false;
        for &byte in bytes.iter() {
            if is_white_space(byte) {
                continue;
            }
            if eod {
                return Err(ASCII85Error::AfterEod(char::from(byte)).into());
            }
            if tilde {
                if byte == b'>' {
                    eod = true;
                    continue;
                } else {
                    return Err(ASCII85Error::CorruptEod(char::from(byte)).into());
                }
            }
            if byte == b'~' {
                tilde = true;
                continue;
            }
            if byte == b'z' {
                if prev.is_empty() {
                    defiltered.extend(&[0, 0, 0, 0]);
                    continue;
                } else {
                    return Err(ASCII85Error::ZInMiddle(debug_bytes(&prev)).into());
                }
            }
            if !(b'!'..=b'u').contains(&byte) {
                return Err(ASCII85Error::InvalidBase85Digit(char::from(byte)).into());
            }
            prev.push(byte);
            if prev.len() < 5 {
                continue;
            }
            // TODO Consider a more efficient algorithm
            let mut value = 0u32;
            for &byte in prev.iter() {
                value = value
                    .checked_mul(85)
                    .ok_or(ASCII85Error::ValueTooLarge("Base85", debug_bytes(&prev)))?
                    .checked_add(u32::from(byte - b'!'))
                    .ok_or(ASCII85Error::ValueTooLarge("Base85", debug_bytes(&prev)))?;
            }
            defiltered.extend_from_slice(&value.to_be_bytes());
            prev.clear();
        }
        if !prev.is_empty() {
            if prev.len() == 1 {
                return Err(ASCII85Error::FinalPartialGroup(debug_bytes(&prev)).into());
            }
            let mut value = 0u32;

            for &byte in prev.iter().chain(&vec![b'u'; 5 - prev.len()]) {
                value = value
                    .checked_mul(85)
                    .ok_or(ASCII85Error::ValueTooLarge("Base85", debug_bytes(&prev)))?
                    .checked_add(u32::from(byte - b'!'))
                    .ok_or(ASCII85Error::ValueTooLarge("Base85", debug_bytes(&prev)))?;
            }
            defiltered.extend_from_slice(&value.to_be_bytes()[..prev.len() - 1]);
        }

        Ok(defiltered)
    }
}

pub(in crate::process) mod error {
    use ::thiserror::Error;

    // FIXME (TEMP) Restrict the use of debug_bytes to error display
    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum ASCII85Error {
        #[error("Invalid ASCII base-85 digit: {0}")]
        InvalidBase85Digit(char),
        #[error("Unexpected character after the EOD marker: {0}")]
        AfterEod(char),
        #[error("Value is is greater than 2^32 - 1. {0} group: {1}")]
        ValueTooLarge(&'static str, String),
        #[error("A z character occurs in the middle of a group: {0}")]
        ZInMiddle(String),
        #[error("A final partial group contains only one character: {0}")]
        FinalPartialGroup(String),
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
        let expected_error = ASCII85Error::InvalidBase85Digit('v');
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(~> a";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = ASCII85Error::AfterEod('a');
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0 G[Y,o @qfdg C`lYu EbT/a uuuuu~>";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = ASCII85Error::ValueTooLarge("Base85", "uuuuu".to_string());
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(/cz~>";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = ASCII85Error::ZInMiddle("/c".to_string());
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(/";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = ASCII85Error::FinalPartialGroup("/".to_string());
        assert_err_eq!(defiltered_result, expected_error);

        let filtered = b"5p/^0G[Y,o@qfdgC`lYuEbTE(~ a";
        let defiltered_result = A85.defilter(filtered);
        let expected_error = ASCII85Error::CorruptEod('a');
        assert_err_eq!(defiltered_result, expected_error);
    }
}
