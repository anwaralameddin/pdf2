use self::error::TiffError;
use super::bits_per_component::BitsPerComponent;
use super::PredictorParms;
use crate::process::error::ProcessResult;
use crate::process::filter::Filter;
use crate::Byte;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::process::filter) struct Tiff {
    parms: PredictorParms,
}

impl Filter for Tiff {
    /// REFERENCE: [7.4.4.4 LZW and Flate predictor functions, p41]
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[crate::Byte]>,
    ) -> ProcessResult<Vec<Byte>> {
        let bytes = bytes.as_ref();

        // TODO This predictor can be implemented in place without the additional buffer
        let mut filtered = Vec::with_capacity(bytes.len());

        let mut bit_reader = BitsReader::new(bytes, self.parms);
        let mut bit_writer = BitsWriter::new(&mut filtered, self.parms);

        // REFERENCE: [[TIFF 6.0 Specification] Section 14: Differencing
        // Predictor, p64]
        while let Some(row) = bit_reader.next_row()? {
            let mut prev_colors = vec![0; self.parms.colors as usize];
            for colors in row.into_iter() {
                for (prev_component, component) in prev_colors.iter().zip(colors.iter()) {
                    if prev_colors.len() != colors.len() {
                        return Err(TiffError::MismatchingComponents(
                            prev_colors.len(),
                            colors.len(),
                        )
                        .into());
                    }
                    // FIXME Verify that the operation is defined modulo 256
                    let diff = component.wrapping_sub(*prev_component)
                        & ((1 << *self.parms.bits_per_component) - 1) as Byte;
                    bit_writer.write_component(diff)?;
                }
                prev_colors = colors
            }
            bit_writer.flush_row()?;
        }

        Ok(filtered)
    }

    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[crate::Byte]>,
    ) -> ProcessResult<Vec<Byte>> {
        let bytes = bytes.as_ref();

        let mut defiltered = Vec::with_capacity(bytes.len());

        let mut bit_reader = BitsReader::new(bytes, self.parms);
        let mut bit_writer = BitsWriter::new(&mut defiltered, self.parms);

        while let Some(row) = bit_reader.next_row()? {
            let mut prev_colors = vec![0; self.parms.colors as usize];
            for diffs in row.into_iter() {
                let mut colors = Vec::with_capacity(diffs.len());
                for (prev_component, diff) in prev_colors.iter().zip(diffs.iter()) {
                    if prev_colors.len() != diffs.len() {
                        return Err(TiffError::MismatchingComponents(
                            prev_colors.len(),
                            diffs.len(),
                        )
                        .into());
                    }
                    // FIXME Verify that the operation is defined modulo 256
                    let component = diff.wrapping_add(*prev_component)
                        & ((1 << *self.parms.bits_per_component) - 1) as Byte;
                    bit_writer.write_component(component)?;
                    colors.push(component);
                }
                prev_colors = colors;
            }
            bit_writer.flush_row()?;
        }

        Ok(defiltered)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BitsReader<'a> {
    bytes: &'a [Byte],
    parms: PredictorParms,
    location: usize,
}

impl<'a> BitsReader<'a> {
    fn next_bits(&mut self) -> ProcessResult<Option<Byte>> {
        let byte_location = self.location / 8;
        let bit_location = self.location % 8;

        let byte = if let Some(byte) = self.bytes.get(byte_location) {
            *byte
        } else {
            return Ok(None);
        };

        if bit_location + *self.parms.bits_per_component > 8 {
            return Err(TiffError::BitsOutOfBound(
                *self.parms.bits_per_component,
                8 - bit_location,
            )
            .into());
        }

        if let BitsPerComponent::Eight = self.parms.bits_per_component {
            self.location += 8;
            return Ok(Some(byte));
        }

        let bits = (byte >> (8 - bit_location - *self.parms.bits_per_component))
            & (((1 << *self.parms.bits_per_component) - 1) as Byte);

        self.location += *self.parms.bits_per_component;

        Ok(Some(bits))
    }

    fn next_colors(&mut self) -> ProcessResult<Option<Vec<Byte>>> {
        if self.location == self.bytes.len() * 8 {
            return Ok(None);
        }

        let mut colors = Vec::with_capacity(*self.parms.colors);

        for i in 0..*self.parms.colors {
            let component = self
                .next_bits()?
                .ok_or_else(|| TiffError::ColorsOutOfBound(*self.parms.colors, i))?;
            colors.push(component);
        }

        Ok(Some(colors))
    }

    fn next_row(&mut self) -> ProcessResult<Option<Vec<Vec<Byte>>>> {
        if self.location == self.bytes.len() * 8 {
            return Ok(None);
        }

        let mut row = Vec::with_capacity(*self.parms.columns);

        for i in 0..*self.parms.columns {
            let colors = self
                .next_colors()?
                .ok_or_else(|| TiffError::ColumnsOutOfBound(*self.parms.columns, i))?;
            row.push(colors);
        }

        // REFERENCE: [7.4.4.4 LZW and Flate predictor functions, p42]
        // The number of bits per row needs to be rounded up to the nearest byte.
        if self.location % 8 != 0 {
            let padding = 8 - self.location % 8;
            if let Some(byte) = self.bytes.get(self.location / 8) {
                let remaining = byte & ((1 << padding) - 1);
                if remaining != 0 {
                    return Err(TiffError::RowMissingPadding(padding, *byte).into());
                }
            }
            self.location += padding;
        }

        Ok(Some(row))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BitsWriter<'a> {
    buffer: &'a mut Vec<Byte>,
    parms: PredictorParms,
    cache: Byte,
    bit_location: usize,
}

impl<'a> BitsWriter<'a> {
    fn write_component(&mut self, component: Byte) -> ProcessResult<()> {
        if self.bit_location + *self.parms.bits_per_component > 8 {
            return Err(TiffError::BitsOutOfBound(
                *self.parms.bits_per_component,
                8 - self.bit_location,
            )
            .into());
        }

        if let BitsPerComponent::Eight = self.parms.bits_per_component {
            self.buffer.push(component);
            self.bit_location = 0;
            return Ok(());
        }
        if component > ((1 << *self.parms.bits_per_component) - 1) as Byte {
            return Err(
                TiffError::CorruptComponent(component, *self.parms.bits_per_component).into(),
            );
        }

        self.cache = (self.cache << *self.parms.bits_per_component) | component;
        self.bit_location += *self.parms.bits_per_component;

        if self.bit_location == 8 {
            self.buffer.push(self.cache);
            self.cache = 0;
            self.bit_location = 0;
        }

        Ok(())
    }

    fn flush_row(&mut self) -> ProcessResult<()> {
        if self.bit_location == 0 {
            return Ok(());
        }

        self.cache <<= 8 - self.bit_location;

        self.buffer.push(self.cache);
        self.cache = 0;
        self.bit_location = 0;

        Ok(())
    }
}

mod convert {
    use super::*;

    impl Tiff {
        pub(in crate::process::filter::predictor) fn new(parms: PredictorParms) -> Self {
            Self { parms }
        }
    }

    impl<'a> BitsReader<'a> {
        pub(super) fn new(bytes: &'a [Byte], parms: PredictorParms) -> Self {
            Self {
                bytes,
                parms,
                location: 0,
            }
        }
    }

    impl<'a> BitsWriter<'a> {
        pub(super) fn new(buffer: &'a mut Vec<Byte>, parms: PredictorParms) -> Self {
            Self {
                buffer,
                parms,
                cache: 0,
                bit_location: 0,
            }
        }
    }
}

pub(in crate::process) mod error {
    use ::thiserror::Error;

    use super::*;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum TiffError {
        #[error("Bits out of bound: Requested {0}. Found {1}")]
        BitsOutOfBound(usize, usize),
        #[error("Colors out of bound: Requested {0}. Found {1}")]
        ColorsOutOfBound(usize, usize),
        #[error("Columns out of bound: Requested {0}. Found {1}")]
        ColumnsOutOfBound(usize, usize),
        #[error("Mismatching number of components: Expected {0}. Found {1}")]
        MismatchingComponents(usize, usize),
        #[error(
            "Row missing padding: The least significant {0} bits of the byte {1:02X} are non-zero"
        )]
        RowMissingPadding(usize, Byte),
        #[error("Corrupt component: {0:08b} does not fit in {1} bits")]
        CorruptComponent(Byte, usize),
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_err_eq;
    use crate::process::filter::predictor::bits_per_component::BitsPerComponent;
    use crate::process::filter::predictor::colors::Colors;
    use crate::process::filter::predictor::columns::Columns;
    use crate::process::filter::predictor::tiff::error::TiffError;
    use crate::process::filter::predictor::tiff::BitsReader;
    use crate::process::filter::predictor::tiff::BitsWriter;
    use crate::process::filter::predictor::PredictorParms;
    // use crate::process::filter::tests::lax_stream_defilter_filter;
    // use crate::Byte;

    // #[test]
    // fn tiff_valid() {
    // TODO
    // - Add tests
    // - Ensure tests cover the different cases for bits_per_component = 1, 2, 4
    // and 8
    // - Validate the below further after implementing XObject

    // lax_stream_defilter_filter(buffer, expected).unwrap();
    // }

    // TODO Add tests
    // #[test]
    // fn tiff_invalid() {
    // }

    #[test]
    fn predictor_read_bits() {
        // Synthetic tests
        let buffer = vec![0b1010_0101, 0b0100_0111];
        let parms = PredictorParms {
            bits_per_component: BitsPerComponent::Two,
            colors: Colors::Two,
            columns: Columns::new(2),
        };
        let mut reader = BitsReader::new(&buffer, parms);
        assert_eq!(reader.next_bits().unwrap(), Some(0b10));
        assert_eq!(reader.next_bits().unwrap(), Some(0b10));
        assert_eq!(reader.next_bits().unwrap(), Some(0b01));
        assert_eq!(reader.next_bits().unwrap(), Some(0b01));
        assert_eq!(reader.next_bits().unwrap(), Some(0b01));
        assert_eq!(reader.next_bits().unwrap(), Some(0b00));
        assert_eq!(reader.next_bits().unwrap(), Some(0b01));
        assert_eq!(reader.next_bits().unwrap(), Some(0b11));
        assert_eq!(reader.next_bits().unwrap(), None);
    }

    #[test]
    fn predictor_read_colors() {
        // Synthetic tests
        let buffer = vec![0b1010_0101, 0b0100_0111];
        let parms = PredictorParms {
            bits_per_component: BitsPerComponent::Two,
            colors: Colors::Three,
            columns: Columns::new(2),
        };
        let mut reader = BitsReader::new(&buffer, parms);
        assert_eq!(reader.next_colors().unwrap(), Some(vec![0b10, 0b10, 0b01]));
        assert_eq!(reader.next_colors().unwrap(), Some(vec![0b01, 0b01, 0b00]));
        let result = reader.next_colors();
        let expected_error = TiffError::ColorsOutOfBound(3, 2);
        assert_err_eq!(result, expected_error);
    }

    #[test]
    fn predictor_read_columns() {
        // Synthetic tests
        let buffer = vec![0b1010_0100, 0b0100_0111];
        let parms = PredictorParms {
            bits_per_component: BitsPerComponent::One,
            colors: Colors::Three,
            columns: Columns::new(2),
        };
        let mut reader = BitsReader::new(&buffer, parms);
        assert_eq!(
            reader.next_row().unwrap(),
            Some(vec![vec![0b1, 0b0, 0b1], vec![0b0, 0b0, 0b1]])
        );
        let result = reader.next_row();
        let expected_error = TiffError::RowMissingPadding(2, 0b0100_0111);
        assert_err_eq!(result, expected_error);
    }

    #[test]
    fn predictor_write_bits() {
        // Synthetic tests
        let mut buffer = Vec::new();
        let parms = PredictorParms {
            bits_per_component: BitsPerComponent::Two,
            colors: Colors::Two,
            columns: Columns::new(2),
        };
        let mut writer = BitsWriter::new(&mut buffer, parms);
        writer.write_component(0b10).unwrap();
        writer.write_component(0b10).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b00).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b11).unwrap();
        writer.flush_row().unwrap();
        assert_eq!(buffer, vec![0b1010_0101, 0b0100_0111]);

        let mut buffer = Vec::new();
        let mut writer = BitsWriter::new(&mut buffer, parms);
        writer.write_component(0b10).unwrap();
        writer.write_component(0b10).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b01).unwrap();
        writer.write_component(0b00).unwrap();
        writer.flush_row().unwrap();
        assert_eq!(buffer, vec![0b1010_0101, 0b0100_0000]);

        let mut buffer = Vec::new();
        let mut writer = BitsWriter::new(&mut buffer, parms);
        let result = writer.write_component(0b110);
        let expected_error = TiffError::CorruptComponent(0b110, 2);
        assert_err_eq!(result, expected_error);
    }
}
