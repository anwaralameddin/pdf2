mod bits_per_component;
mod colors;
mod columns;
pub(in crate::process) mod png;
pub(in crate::process) mod tiff;

use self::bits_per_component::BitsPerComponent;
use self::colors::Colors;
use self::columns::Columns;
use self::png::Png;
use self::tiff::Tiff;
use super::Filter;
use crate::process::filter::error::FilterResult;
use crate::Byte;

const KEY_PREDICTOR: &[Byte] = b"Predictor";
const KEY_BITS_PER_COMPONENT: &[Byte] = b"BitsPerComponent";
const KEY_COLORS: &[Byte] = b"Colors";
const KEY_COLUMNS: &[Byte] = b"Columns";

/// REFERENCE: [Table 8 — Optional parameters for LZWDecode and FlateDecode
/// filters, p40] and [Table 10 — Predictor values. p42]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Predictor {
    #[default]
    None,
    Tiff(Tiff),
    Png(Png),
}

impl<'buffer> Filter<'buffer> for Predictor {
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<Vec<Byte>> {
        match self {
            Self::None => Ok(bytes.into()),
            Self::Tiff(predictor) => predictor.filter(bytes),
            Self::Png(png) => png.filter(bytes),
        }
    }

    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<Vec<Byte>> {
        match self {
            Self::None => Ok(bytes.into()),
            Self::Tiff(predictor) => predictor.defilter(bytes),
            Self::Png(png) => png.defilter(bytes),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PredictorParms {
    bits_per_component: BitsPerComponent,
    colors: Colors,
    columns: Columns,
}

mod convert {
    use ::std::ops::Deref;

    use super::png::PngAlgorithm;
    use super::*;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::DirectValue;
    use crate::parse::ObjectParser;
    use crate::process::filter::error::FilterErr;
    use crate::process::filter::error::FilterErrorCode;

    impl Predictor {
        pub(in crate::process::filter) fn new(decode_parms: &Dictionary) -> FilterResult<Self> {
            let bits_per_component = decode_parms
                .opt_get(KEY_BITS_PER_COMPONENT)
                .map(BitsPerComponent::try_from)
                .transpose()?
                .unwrap_or_default();
            let colors = decode_parms
                .opt_get(KEY_COLORS)
                .map(Colors::try_from)
                .transpose()?
                .unwrap_or_default();
            let columns = decode_parms
                .opt_get(KEY_COLUMNS)
                .map(Columns::try_from)
                .transpose()?
                .unwrap_or_default();
            let parms = PredictorParms {
                bits_per_component,
                colors,
                columns,
            };

            match decode_parms.opt_get(KEY_PREDICTOR) {
                Some(DirectValue::Numeric(Numeric::Integer(value))) => match value.deref() {
                    1 => Ok(Self::None),
                    2 => Ok(Self::Tiff(Tiff::new(parms))),
                    10 => Ok(Self::Png(Png::new(PngAlgorithm::None, parms))),
                    11 => Ok(Self::Png(Png::new(PngAlgorithm::Sub, parms))),
                    12 => Ok(Self::Png(Png::new(PngAlgorithm::Up, parms))),
                    13 => Ok(Self::Png(Png::new(PngAlgorithm::Average, parms))),
                    14 => Ok(Self::Png(Png::new(PngAlgorithm::Paeth, parms))),
                    15 => Ok(Self::Png(Png::new(PngAlgorithm::Optimum, parms))),
                    _ => Err(FilterErr::new(
                        stringify!(Predictor),
                        FilterErrorCode::UnsupportedParameter(*value.deref()),
                    )),
                },
                Some(value) => Err(FilterErr::new(
                    stringify!(Predictor),
                    FilterErrorCode::ValueType(stringify!(Integer), value.span()),
                )),
                None => Ok(Self::None),
            }
        }
    }
}
