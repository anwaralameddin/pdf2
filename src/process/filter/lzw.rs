use ::std::collections::HashMap;

use self::error::LzwErrorCode;
use super::predictor::Predictor;
use super::Filter;
use crate::process::filter::error::FilterResult;
use crate::Byte;
use crate::DECODED_LIMIT;

const KEY_EARLY_CHANGE: &str = "EarlyChange";
const MIN_CODE_SIZE: usize = 9;
const MAX_CODE_SIZE: usize = 12;
const CLEAR_CODE: u16 = 256;
const EOD_CODE: u16 = 257;
const FIRST_CODE: u16 = 258;

/// REFERENCE: [7.4.4 LZWDecode and FlateDecode filters, p38] and [[Adobe TIFF
/// Revision 6.0; Final (TIFF)] 7.4.4.2 "Details of LZW encoding"]
/// The LZW (Lempel-Ziv-Welch) adaptive compression filter.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub(super) struct Lzw {
    predictor: Predictor,
    early_change: EarlyChange,
}

impl<'buffer> Filter<'buffer> for Lzw {
    /// REFERENCE: [7.4.4.2 Details of LZW encoding, p38-40]
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        let bytes = self.predictor.filter(bytes)?;

        let mut filter = LzwFilter::default();
        // TODO The choice of capacity is not optimal
        let mut filtered = Vec::with_capacity(bytes.len() / 2);

        let mut code = CLEAR_CODE;
        filter.output_code(CLEAR_CODE, &mut filtered);

        for &byte in bytes.iter() {
            filter.seq.push(byte);
            if let [b] = filter.seq.as_slice() {
                code = u16::from(*b);
                continue;
            }
            if let Some(&c) = filter.table.get(&filter.seq) {
                code = c;
                continue;
            }

            if *self.early_change && filter.len == (1 << *filter.code_size) {
                filter.code_size.increment();
            }
            filter.output_code(code, &mut filtered);
            if !*self.early_change && filter.len == (1 << *filter.code_size) {
                filter.code_size.increment();
            }

            filter.update_table(self.early_change, &mut filtered);
            filter.seq = vec![byte];
            code = u16::from(byte);
        }

        if !filter.seq.is_empty() {
            filter.output_code(code, &mut filtered);
        }

        filter.output_code(EOD_CODE, &mut filtered);

        if filter.leftover_bits > 0 {
            filtered.push((filter.leftover_code << (8 - filter.leftover_bits)) as Byte);
        }

        Ok(filtered)
    }

    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        let bytes = bytes.as_ref();

        let mut defilter = LzwDefilter::default();
        // TODO The choice of capacity is not optimal
        let mut defiltered = Vec::with_capacity(bytes.len() * 2);

        for (i, &byte) in bytes.iter().enumerate() {
            if defilter.eod {
                return Err(LzwErrorCode::ByteAfterEod(byte).into());
            }

            if *self.early_change && defilter.len == (1 << *defilter.code_size) {
                defilter.code_size.increment();
            }
            // As code_size > 8, depending on the leftover bits, the read byte
            // might not be enough to form a code, hence the need to read more
            // bytes
            if *defilter.code_size > 8 + defilter.leftover_bits {
                defilter.prev_byte <<= 8;
                defilter.prev_byte |= u16::from(byte);
                defilter.leftover_bits += 8;
                continue;
            }
            let code = defilter.get_code(byte);
            if !*self.early_change && defilter.len == (1 << *defilter.code_size) {
                defilter.code_size.increment();
            }

            if code == CLEAR_CODE {
                defilter = LzwDefilter {
                    prev_byte: defilter.prev_byte,
                    leftover_bits: defilter.leftover_bits,
                    eod: defilter.eod,
                    ..LzwDefilter::default()
                };
                continue;
            }

            if code == EOD_CODE {
                defilter.eod = true;
                continue;
            }

            if defilter.len > (1 << MAX_CODE_SIZE) {
                return Err(LzwErrorCode::FullTable.into());
            }

            if code >= defilter.len {
                return Err(LzwErrorCode::OutOfBounds {
                    code,
                    len: defilter.len,
                }
                .into());
            }

            let first = if code < CLEAR_CODE {
                code as Byte
            } else if let [first, ..] = defilter.table[usize::from(code - FIRST_CODE)].as_slice() {
                *first
            } else {
                unreachable!(
                    "update_table is always passed a non-empty output, guaranteeing that the \
                     table entry is never empty"
                );
            };
            // Complete the table's previous entry
            if let Some(prev) = defilter.table.last_mut() {
                prev.push(first);
            }

            let output = if code < CLEAR_CODE {
                vec![code as Byte]
            } else {
                defilter.table[usize::from(code - FIRST_CODE)].clone()
            };

            defiltered.extend_from_slice(&output);
            defilter.update_table(output);

            if defiltered.len() > DECODED_LIMIT {
                return Err(LzwErrorCode::TooLarge(i, bytes.len()).into());
            }
        }

        // Ensure no non-zero leftover bits
        let code = defilter.prev_byte & ((1 << defilter.leftover_bits) - 1);
        if code != 0 {
            if defilter.eod {
                return Err(LzwErrorCode::CodeAfterEod(code).into());
            }
            return Err(LzwErrorCode::LeftoverBits(code).into());
        }

        let defiltered = self.predictor.defilter(defiltered)?;

        Ok(defiltered)
    }
}

/// REFERENCE: [Table 8 — Optional parameters for LZWDecode and FlateDecode
/// filters, p40]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EarlyChange(bool);

impl Default for EarlyChange {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug)]
struct LzwFilter {
    table: HashMap<Vec<Byte>, u16>,
    len: u16,
    seq: Vec<Byte>,
    code_size: CodeSize,
    leftover_bits: usize,
    leftover_code: u16,
}

impl Default for LzwFilter {
    fn default() -> Self {
        Self {
            table: HashMap::with_capacity((1 << MAX_CODE_SIZE) - usize::from(FIRST_CODE)),
            len: FIRST_CODE,
            seq: Default::default(),
            code_size: Default::default(),
            leftover_bits: Default::default(),
            leftover_code: Default::default(),
        }
    }
}

impl LzwFilter {
    fn output_code(&mut self, code: u16, filtered: &mut Vec<Byte>) {
        let new_leftover_bits = (*self.code_size - 8) + self.leftover_bits;
        filtered.push(
            (self.leftover_code << (8 - self.leftover_bits)) as Byte
                | (code >> new_leftover_bits) as Byte,
        );
        self.leftover_bits = new_leftover_bits;
        self.leftover_code = code & ((1 << self.leftover_bits) - 1);
        while self.leftover_bits >= 8 {
            filtered.push((self.leftover_code >> (self.leftover_bits - 8)) as Byte);
            self.leftover_bits -= 8;
            self.leftover_code &= (1 << self.leftover_bits) - 1;
        }
    }

    fn update_table(&mut self, early_change: EarlyChange, filtered: &mut Vec<Byte>) {
        // HACK The below does not seem to fit perfectly with the specification, but it works
        if *early_change && self.len + 1 == (1 << MAX_CODE_SIZE) {
            self.table.clear();
            // While increasing the length does not initially make sense here,
            // it is needed to reach the following branch
            self.len += 1;
        } else if self.len == (1 << MAX_CODE_SIZE) {
            self.output_code(CLEAR_CODE, filtered);

            self.table = HashMap::with_capacity((1 << MAX_CODE_SIZE) - usize::from(FIRST_CODE));
            self.len = FIRST_CODE;
            self.code_size = Default::default();
        } else {
            self.table.insert(self.seq.clone(), self.len);
            self.len += 1;
        }
    }
}

#[derive(Debug)]
struct LzwDefilter {
    table: Vec<Vec<Byte>>,
    len: u16,
    code_size: CodeSize,
    eod: bool,
    prev_byte: u16,
    leftover_bits: usize,
}

impl Default for LzwDefilter {
    fn default() -> Self {
        Self {
            table: Vec::with_capacity((1 << MAX_CODE_SIZE) - usize::from(FIRST_CODE)),
            len: FIRST_CODE,
            code_size: Default::default(),
            eod: Default::default(),
            prev_byte: Default::default(),
            leftover_bits: Default::default(),
        }
    }
}

impl LzwDefilter {
    fn get_code(&mut self, byte: Byte) -> u16 {
        // This is handled by the caller
        // if *self.code_size > 8 + self.leftover_bits {
        //     unreachable!("The caller should ensure this condition is unreachable here");
        // }
        let new_leftover_bits = 8 + self.leftover_bits - *self.code_size;
        let code = (((self.prev_byte) & ((1 << self.leftover_bits) - 1))
            << (*self.code_size - self.leftover_bits))
            | (u16::from(byte) >> (new_leftover_bits));
        self.leftover_bits = new_leftover_bits;
        self.prev_byte = u16::from(byte);
        code
    }

    fn update_table(&mut self, output: Vec<Byte>) {
        // The output will be completed in the following iteration
        self.table.push(output);
        self.len += 1;
    }
}

#[derive(Debug, Default)]
enum CodeSize {
    #[default]
    Nine,
    Ten,
    Eleven,
    Twelve,
}

impl CodeSize {
    fn increment(&mut self) {
        match self {
            Self::Nine => *self = Self::Ten,
            Self::Ten => *self = Self::Eleven,
            Self::Eleven => *self = Self::Twelve,
            Self::Twelve => {}
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::DirectValue;
    use crate::process::filter::error::FilterErr;
    use crate::process::filter::error::FilterErrorCode;
    use crate::process::filter::error::FilterResult;

    impl Lzw {
        pub(in crate::process::filter) fn new<'buffer>(
            decode_parms: Option<&'buffer Dictionary>,
        ) -> FilterResult<'buffer, Self> {
            if let Some(decode_parms) = decode_parms {
                let predictor = Predictor::new(decode_parms)?;
                let early_change = decode_parms
                    .opt_get(KEY_EARLY_CHANGE)
                    .map(EarlyChange::try_from)
                    .transpose()?
                    .unwrap_or_default();
                Ok(Self {
                    predictor,
                    early_change,
                })
            } else {
                Ok(Self::default())
            }
        }
    }

    impl EarlyChange {
        pub(super) fn new(value: bool) -> Self {
            Self(value)
        }
    }

    impl<'buffer> TryFrom<&'buffer DirectValue<'buffer>> for EarlyChange {
        type Error = FilterErr<'buffer>;

        fn try_from(value: &'buffer DirectValue<'buffer>) -> Result<Self, Self::Error> {
            if let DirectValue::Numeric(Numeric::Integer(value)) = value {
                match value.deref() {
                    0 => Ok(Self(false)),
                    1 => Ok(Self(true)),
                    _ => Err(FilterErr::new(
                        stringify!(EarlyChange),
                        FilterErrorCode::UnsupportedParameter(value.deref()),
                    )),
                }
            } else {
                Err(FilterErr::new(
                    stringify!(EarlyChange),
                    FilterErrorCode::ValueType(stringify!(Integer), value),
                ))
            }
        }
    }

    impl Deref for EarlyChange {
        type Target = bool;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Deref for CodeSize {
        type Target = usize;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::Nine => &9,
                Self::Ten => &10,
                Self::Eleven => &11,
                Self::Twelve => &12,
            }
        }
    }
}

pub(in crate::process::filter) mod error {
    use ::thiserror::Error;

    use crate::Byte;
    use crate::DECODED_LIMIT;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum LzwErrorCode {
        #[error("Unexpected byte after the EOD marker: {0:02x}")]
        ByteAfterEod(Byte),
        #[error("Unexpected code after the EOD marker: {0}")]
        CodeAfterEod(u16),
        #[error("Out of bounds. Code: {code}. Table length: {len}")]
        OutOfBounds { code: u16, len: u16 },
        #[error("Leftover bits: {0:b}")]
        LeftoverBits(u16),
        #[error(
            "Too large: Exceeding the set limit of {} bytes while defiltering byte {0} out of {1}",
            DECODED_LIMIT
        )]
        TooLarge(usize, usize),
        #[error("Caution: Nonconforming encoder: The table was not cleared when full")]
        FullTable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_err_eq;
    use crate::object::indirect::stream::Stream;
    use crate::parse::Parser;
    // use crate::lax_stream_defilter_filter;
    use crate::strict_stream_defilter_filter;

    #[test]
    fn lzw_valid() {
        // TODO Replace with a longer test to invoke and test table clearing for full table
        // PDF produced by Acrobat 3.0 Import Plug-in
        let buffer =
            include_bytes!("../../../tests/data/0aa43346a9df321584b2181fa2a4b17d_lzw_671.bin");
        let expected =
            include_bytes!("../../../tests/code/0aa43346a9df321584b2181fa2a4b17d_lzw_671.data");
        strict_stream_defilter_filter!(buffer, expected);

        // TODO Add tests, especially those not requiring an early change
        let _filtering = Lzw {
            early_change: EarlyChange::new(false),
            ..Default::default()
        };

        // When the stream is filtered using both the LZWDecode and
        // ASCII85Decode, the latter may produce different equivalent results.
        // Hence, there is a need for a lax comparison.
        // lax_stream_defilter_filter(buffer, expected).unwrap();
    }

    #[test]
    fn lzw_invalid() {
        let filtering = Lzw {
            predictor: Predictor::default(),
            early_change: EarlyChange::new(false),
        };

        // TODO Regenerate the encoded stream
        // Too large decoded stream
        // In the "best case" scenario, the n-byte encoded string is decoded to
        // ~n^2/9 bytes for the first 3840 codes. This can be used to create
        // disturbingly large decoded streams. If not for the above decoding
        // checks, namely `FullTable` and `TooLarge`, the following ~400KiB the
        // encoded stream would decode to more than 1GiB.
        let buffer = include_bytes!("../../../tests/data/SYNTHETIC_lzw_malicious.bin");
        let defiltered_result = filtering.defilter(buffer);
        // let expected_error = LzwError::TooLarge(422068, 422070);
        let expected_error = LzwErrorCode::FullTable;
        assert_err_eq!(defiltered_result, expected_error);

        // TODO Add tests
    }
}
