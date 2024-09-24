use self::error::PngError;
use super::PredictorParms;
use crate::process::error::ProcessResult;
use crate::process::filter::Filter;
use crate::Byte;

/// REFERENCE:
/// - [7.4.4.4 LZW and Flate predictor functions, p41]
/// - [[https://www.w3.org/TR/PNG-Filters.html]]
/// The World Wide Web Consortium’s Portable Network Graphics filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::process::filter) struct Png {
    algorithm: PngAlgorithm,
    parms: PredictorParms,
}

impl Filter for Png {
    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html]]
    fn filter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        let bytes = bytes.as_ref();
        // REFERENCE:
        // [[https://www.w3.org/TR/png/#4Concepts.EncodingScanlineAbs]
        // 4.6.2 Scanline serialization]
        let bits_per_scanline =
            *self.parms.columns * *self.parms.colors * *self.parms.bits_per_component;
        // REFERENCE: [7.4.4.4 LZW and Flate predictor functions, p42]
        // The number of bits per scanline needs to be rounded up
        let bytes_per_scanline = if bits_per_scanline % 8 == 0 {
            bits_per_scanline / 8
        } else {
            bits_per_scanline / 8 + 1
        };
        // The total number of bytes per scanline in the filtered data is one
        // more than the number of bytes per scanline to account for the filter
        // type byte
        let encoded_bytes_per_scanline = bytes_per_scanline + 1;

        let mut filtered =
            Vec::with_capacity(bytes.len() * encoded_bytes_per_scanline / bytes_per_scanline);

        let bits_per_pixel = *self.parms.bits_per_component * *self.parms.colors;
        // REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html]]
        // The number of bits per scanline needs to be rounded up
        let bytes_per_pixel = if bits_per_pixel % 8 == 0 {
            bits_per_pixel / 8
        } else {
            bits_per_pixel / 8 + 1
        };

        let diff_fn = self.algorithm.diff_fn()?;
        // REFERENCE:
        // - [7.4.4.4 LZW and Flate predictor functions, p42]
        // - [[https://www.w3.org/TR/PNG-Filters.html]]
        let mut prior_scanline = vec![0; bytes_per_scanline];
        let mut diff = vec![0; encoded_bytes_per_scanline];
        // Add filter type byte
        diff[0] = *self.algorithm;

        let scanlines = bytes.chunks_exact(bytes_per_scanline);
        if !scanlines.remainder().is_empty() {
            return Err(PngError::NumBytes(bytes.len(), bytes_per_scanline).into());
        }
        scanlines.for_each(|scanline| {
            diff_fn(&mut diff, &mut prior_scanline, scanline, bytes_per_pixel);
            filtered.extend_from_slice(&diff);
        });

        Ok(filtered)
    }

    fn defilter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        let bytes = bytes.as_ref();

        let bits_per_scanline =
            *self.parms.columns * *self.parms.colors * *self.parms.bits_per_component;
        let bytes_per_scanline = if bits_per_scanline % 8 == 0 {
            bits_per_scanline / 8
        } else {
            bits_per_scanline / 8 + 1
        };
        // The total number of bytes per scanline in the filtered data is one
        // more than the number of bytes per scanline to account for the filter
        // type byte
        let encoded_bytes_per_scanline = bytes_per_scanline + 1;

        let mut defiltered =
            Vec::with_capacity(bytes.len() * bytes_per_scanline / encoded_bytes_per_scanline);

        let bits_per_pixel = *self.parms.bits_per_component * *self.parms.colors;
        let bytes_per_pixel = if bits_per_pixel % 8 == 0 {
            bits_per_pixel / 8
        } else {
            bits_per_pixel / 8 + 1
        };

        // REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html]]
        let mut prior_scanline = vec![0; bytes_per_scanline];
        let mut scanline = vec![0; bytes_per_scanline];

        let diffs = bytes.chunks_exact(encoded_bytes_per_scanline);
        if !diffs.remainder().is_empty() {
            return Err(PngError::EncodedNumBytes(bytes.len(), encoded_bytes_per_scanline).into());
        }
        if let PngAlgorithm::Optimum = self.algorithm {
            for diff in diffs {
                let filter_type = diff[0];
                let algorithm = PngAlgorithm::try_from(filter_type)?;
                let rev_fn = algorithm.rev_fn()?;
                rev_fn(&mut scanline, &mut prior_scanline, diff, bytes_per_pixel);
                defiltered.extend_from_slice(&scanline);
            }
        } else {
            let rev_fn = self.algorithm.rev_fn()?;
            for diff in diffs {
                // REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html]]
                let filter_type = diff[0];
                let algorith = PngAlgorithm::try_from(filter_type)?;
                // REFERENCE: [7.4.4.4 LZW and Flate predictor functions, p42]
                // FIXME The optimum apply different filters to different
                // scanlines
                if algorith != self.algorithm {
                    // TODO Convert into a warning and rollback to the optimum
                    // algorithm
                    return Err(PngError::MismatchingAlgorithm(*self.algorithm, filter_type).into());
                }
                // let rev_fn = algorithm.rev_fn();
                rev_fn(&mut scanline, &mut prior_scanline, diff, bytes_per_pixel);
                defiltered.extend_from_slice(&scanline);
            }
        }

        Ok(defiltered)
    }
}

/// REFERENCE: [Table 10 — Predictor values. p42]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::process::filter::predictor) enum PngAlgorithm {
    None,
    Sub,
    Up,
    Average,
    Paeth,
    Optimum,
}

mod process {

    use super::*;

    type PngDiffFn = fn(&mut [Byte], &mut [Byte], &[Byte], usize);
    type PngRevFn = fn(&mut [Byte], &mut [Byte], &[Byte], usize);

    impl PngAlgorithm {
        #[inline]
        pub(super) fn diff_fn(&self) -> Result<PngDiffFn, PngError> {
            match self {
                Self::None => Ok(diff_none),
                Self::Sub => Ok(diff_sub),
                Self::Up => Ok(diff_up),
                Self::Average => Ok(diff_average),
                Self::Paeth => Ok(diff_paeth),
                Self::Optimum => todo!("Implement the Optimum filter"),
            }
        }

        pub(super) fn rev_fn(&self) -> Result<PngRevFn, PngError> {
            match self {
                Self::None => Ok(rev_none),
                Self::Sub => Ok(rev_sub),
                Self::Up => Ok(rev_up),
                Self::Average => Ok(rev_average),
                Self::Paeth => Ok(rev_paeth),
                Self::Optimum => todo!("Implement the Optimum filter"),
            }
        }
    }

    #[inline]
    fn diff_none(
        _diff: &mut [Byte],
        _prior_scanline: &mut [Byte],
        _scanline: &[Byte],
        _bytes_per_pixel: usize,
    ) {
    }

    #[inline]
    fn rev_none(
        _scanline: &mut [Byte],
        _prior_scanline: &mut [Byte],
        _diff: &[Byte],
        _bytes_per_pixel: usize,
    ) {
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 6.3. Filter type 1:
    /// Sub]
    #[inline]
    fn diff_sub(
        diff: &mut [Byte],
        _prior_scanline: &mut [Byte],
        scanline: &[Byte],
        bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let left = scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            // The added one is needed to skip the filter type byte
            diff[x + 1] = scanline[x].wrapping_sub(*left);
        }
        // There is no need to update the previous scanline as it is not used in
        // this filter
        // prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 6.3. Filter type 1:
    /// Sub]
    #[inline]
    fn rev_sub(
        scanline: &mut [Byte],
        _prior_scanline: &mut [Byte],
        diff: &[Byte],
        bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let left = scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            scanline[x] = diff[x + 1].wrapping_add(*left);
        }
        // There is no need to update the previous scanline as it is not used in
        // this filter
        // FIXME This needs to be uncommented when the optimum filter is implemented
        // prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 6.4. Filter type 2:
    /// Up]
    #[inline]
    fn diff_up(
        diff: &mut [Byte],
        prior_scanline: &mut [Byte],
        scanline: &[Byte],
        _bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let up = prior_scanline.get(x).unwrap_or(&0);
            // The added one is needed to skip the filter type byte
            diff[x + 1] = scanline[x].wrapping_sub(*up);
        }
        prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 6.4. Filter type 2:
    /// Up]
    #[inline]
    fn rev_up(
        scanline: &mut [Byte],
        prior_scanline: &mut [Byte],
        diff: &[Byte],
        _bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let up = prior_scanline.get(x).unwrap_or(&0);
            scanline[x] = diff[x + 1].wrapping_add(*up);
        }
        prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 6.5. Filter type 3:
    /// Average]
    #[inline]
    fn diff_average(
        diff: &mut [Byte],
        prior_scanline: &mut [Byte],
        scanline: &[Byte],
        bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let left = scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            let up = prior_scanline.get(x).unwrap_or(&0);
            diff[x + 1] =
                scanline[x].wrapping_sub(((u16::from(*left) + u16::from(*up)) >> 1) as Byte);
        }
        prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 6.5. Filter type 3:
    /// Average]
    #[inline]
    fn rev_average(
        scanline: &mut [Byte],
        prior_scanline: &mut [Byte],
        diff: &[Byte],
        bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let left = scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            let up = prior_scanline.get(x).unwrap_or(&0);
            scanline[x] =
                diff[x + 1].wrapping_add(((u16::from(*up) + u16::from(*left)) >> 1) as Byte);
        }
        prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 66.6. Filter type
    /// 4: Paeth]
    #[inline]
    fn diff_paeth(
        diff: &mut [Byte],
        prior_scanline: &mut [Byte],
        scanline: &[Byte],
        bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let left = scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            let up = prior_scanline.get(x).unwrap_or(&0);
            let up_left = prior_scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            diff[x + 1] = scanline[x].wrapping_sub(paeth_predictor(*left, *up, *up_left));
        }
        prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 66.6. Filter type
    /// 4: Paeth]
    #[inline]
    fn rev_paeth(
        scanline: &mut [Byte],
        prior_scanline: &mut [Byte],
        diff: &[Byte],
        bytes_per_pixel: usize,
    ) {
        for x in 0..scanline.len() {
            let left = scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            let up = prior_scanline.get(x).unwrap_or(&0);
            let up_left = prior_scanline.get(x - bytes_per_pixel).unwrap_or(&0);
            scanline[x] = diff[x + 1].wrapping_add(paeth_predictor(*left, *up, *up_left));
        }
        prior_scanline.copy_from_slice(scanline);
    }

    /// REFERENCE: [[https://www.w3.org/TR/PNG-Filters.html] 66.6. Filter type
    /// 4: Paeth]
    fn paeth_predictor(left: Byte, up: Byte, up_left: Byte) -> Byte {
        let p = i16::from(left) + i16::from(up) - i16::from(up_left);
        let p_left = (p - i16::from(left)).abs();
        let p_up = (p - i16::from(up)).abs();
        let p_up_left = (p - i16::from(up_left)).abs();
        if p_left <= p_up && p_left <= p_up_left {
            left
        } else if p_up <= p_up_left {
            up
        } else {
            up_left
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl Png {
        pub(in crate::process::filter::predictor) fn new(
            algorithm: PngAlgorithm,
            parms: PredictorParms,
        ) -> Self {
            Self { algorithm, parms }
        }
    }

    impl TryFrom<Byte> for PngAlgorithm {
        type Error = PngError;

        fn try_from(value: Byte) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::None),
                1 => Ok(Self::Sub),
                2 => Ok(Self::Up),
                3 => Ok(Self::Average),
                4 => Ok(Self::Paeth),
                5 => Ok(Self::Optimum),
                _ => Err(PngError::Unsupported(value)),
            }
        }
    }

    impl Deref for PngAlgorithm {
        type Target = Byte;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::None => &0,
                Self::Sub => &1,
                Self::Up => &2,
                Self::Average => &3,
                Self::Paeth => &4,
                Self::Optimum => &5,
            }
        }
    }
}

pub(in crate::process) mod error {
    use ::thiserror::Error;

    use crate::Byte;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum PngError {
        #[error("Unsupported PNG filter. Type: {0}")]
        Unsupported(Byte),
        #[error("Number of bytes: {0} is not a multiple of {1} bytes per encoded scanline")]
        EncodedNumBytes(usize, usize),
        #[error("Number of bytes: {0} is not a multiple of {1} bytes per scanline")]
        NumBytes(usize, usize),
        // TODO The below error variant is temporary and should be replaced with
        // a warning
        #[error("Mismatching filter type. Expected: {0}, Found: {1}")]
        MismatchingAlgorithm(Byte, Byte),
    }
}

#[cfg(test)]
mod tests {

    use crate::process::filter::tests::lax_stream_defilter_filter;

    #[test]
    fn png_valid() {
        // PDF produced by Microsoft Word for Office 365
        let buffer = include_bytes!(
            "../../../../tests/data/A22408A93E27AD44B908FB70279DC7A0_xref_stream.bin"
        );
        let expected = include!("../../../../tests/code/A22408A93E27AD44B908FB70279DC7A0_data.rs");
        lax_stream_defilter_filter(buffer, expected).unwrap();
    }

    // TODO Add tests
    // #[test]
    // fn png_invalid() {
    // }
}
