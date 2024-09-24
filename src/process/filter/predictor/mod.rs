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
use crate::process::error::ProcessResult;
use crate::Byte;

const KEY_PREDICTOR: &str = "Predictor";
const KEY_BITS_PER_COMPONENT: &str = "BitsPerComponent";
const KEY_COLORS: &str = "Colors";
const KEY_COLUMNS: &str = "Columns";

/// REFERENCE: [Table 8 — Optional parameters for LZWDecode and FlateDecode
/// filters, p40] and [Table 10 — Predictor values. p42]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Predictor {
    #[default]
    None,
    Tiff(Tiff),
    Png(Png),
}

impl Filter for Predictor {
    fn filter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        match self {
            Self::None => Ok(bytes.into()),
            Self::Tiff(predictor) => predictor.filter(bytes),
            Self::Png(png) => png.filter(bytes),
        }
    }

    fn defilter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
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
    use super::error::PredictorError;
    use super::png::PngAlgorithm;
    use super::*;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::OwnedDirectValue;

    impl Predictor {
        pub(in crate::process::filter) fn new(decode_parms: &OwnedDictionary) -> ProcessResult<Self> {
            let bits_per_component = decode_parms
                .get(KEY_BITS_PER_COMPONENT)
                .map(BitsPerComponent::try_from)
                .transpose()?
                .unwrap_or_default();
            let colors = decode_parms
                .get(KEY_COLORS)
                .map(Colors::try_from)
                .transpose()?
                .unwrap_or_default();
            let columns = decode_parms
                .get(KEY_COLUMNS)
                .map(Columns::try_from)
                .transpose()?
                .unwrap_or_default();
            let parms = PredictorParms {
                bits_per_component,
                colors,
                columns,
            };

            match decode_parms.get(KEY_PREDICTOR) {
                Some(OwnedDirectValue::Numeric(Numeric::Integer(value))) => match **value {
                    1 => Ok(Self::None),
                    2 => Ok(Self::Tiff(Tiff::new(parms))),
                    10 => Ok(Self::Png(Png::new(PngAlgorithm::None, parms))),
                    11 => Ok(Self::Png(Png::new(PngAlgorithm::Sub, parms))),
                    12 => Ok(Self::Png(Png::new(PngAlgorithm::Up, parms))),
                    13 => Ok(Self::Png(Png::new(PngAlgorithm::Average, parms))),
                    14 => Ok(Self::Png(Png::new(PngAlgorithm::Paeth, parms))),
                    15 => Ok(Self::Png(Png::new(PngAlgorithm::Optimum, parms))),
                    _ => Err(PredictorError::Unsupported(stringify!(Predictor), **value).into()),
                },
                Some(value) => {
                    Err(PredictorError::DataType(stringify!(Predictor), value.clone()).into())
                }
                None => Ok(Self::None),
            }
        }
    }
}

pub(in crate::process) mod error {
    use ::thiserror::Error;

    use crate::object::direct::OwnedDirectValue;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum PredictorError {
        #[error("{0}: Invalid data type. Input: {1}")]
        DataType(&'static str, OwnedDirectValue),
        #[error("{0}: Unsupport value. Input: {1}")]
        Unsupported(&'static str, i128),
    }
}
