pub(super) mod ascii_85;
pub(crate) mod ascii_hex;
pub(super) mod ccitt_fax;
pub(super) mod crypt;
pub(super) mod dct;
pub(crate) mod error;
pub(super) mod flate;
pub(super) mod jbig2;
pub(super) mod jpx;
pub(super) mod lzw;
pub(super) mod predictor;
pub(super) mod run_length;

use ::std::ops::Deref;

use self::ascii_85::A85;
use self::ascii_hex::AHx;
use self::ccitt_fax::Ccf;
use self::crypt::Crypt;
use self::dct::Dct;
use self::error::FilterErr;
use self::error::FilterErrorCode;
use self::error::FilterResult;
use self::flate::Fl;
use self::jbig2::JBiG2;
use self::jpx::Jpx;
use self::lzw::Lzw;
use self::run_length::RL;
use crate::object::direct::dictionary::Dictionary;
use crate::object::direct::name::Name;
use crate::object::indirect::stream::KEY_DECODEPARMS;
use crate::object::indirect::stream::KEY_F;
use crate::object::indirect::stream::KEY_FDECODEPARMS;
use crate::object::indirect::stream::KEY_FFILTER;
use crate::object::indirect::stream::KEY_FILTER;
use crate::Byte;

pub(crate) trait Filter<'buffer> {
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>>;

    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>>;
}

#[derive(Debug, PartialEq)]
pub(crate) struct FilteringChain(Vec<Filtering>);

impl<'buffer> Filter<'buffer> for FilteringChain {
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        // The filters in the stream dictionary are in the order they need to
        // be applied to defilter the data. Filter the data by applying the
        // filters in the reverse order.
        let mut filtered: Vec<_>;
        if let [rest @ .., last] = self.0.as_slice() {
            filtered = last.filter(bytes)?;
            for filtering in rest.iter().rev() {
                filtered = filtering.filter(filtered)?;
            }
        } else {
            filtered = bytes.into();
        }
        Ok(filtered)
    }

    /// REFERENCE: [7.3.8.2 Stream extent, p31-33] and [7.4 Filters, p34]
    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        // Multiple filters can be provided in the Filter array in the order
        // they need to be applied to defilter the encoded data.

        // The below ad hoc approach is there to avoid the unnecessary
        // conversion to Vec<Byte> in the first iteration
        // self.0.iter().try_fold(
        //     bytes.into(),
        //     |bytes, filtering| -> ProcessResult<Vec<Byte>> { filtering.defilter(bytes) },
        // )
        let mut defiltered: Vec<_>;
        if let [first, rest @ ..] = self.0.as_slice() {
            defiltered = first.defilter(bytes)?;
            for filtering in rest {
                defiltered = filtering.defilter(defiltered)?;
            }
        } else {
            defiltered = bytes.into();
        }
        Ok(defiltered)
    }
}

/// REFERENCE: [Table 6: Standard filters, p35-36]
/// NOTE: This structure is named `Filtering` to avoid conflicts with the
/// `Filter` trait. Also, this has the side effect of having consistent naming
/// with the `Encoding` structure and the `Encoder` trait.
#[derive(Debug, PartialEq, Clone, Copy)]
enum Filtering {
    None,
    AHx(AHx),
    A85(A85),
    Lzw(Lzw),
    Fl(Fl),
    RL(RL),
    Ccf(Ccf),
    JBiG2(JBiG2),
    Dct(Dct),
    Jpx(Jpx),
    Crypt(Crypt),
}

impl Filtering {
    pub(super) fn new<'buffer>(
        name: &'buffer Name,
        decode_parms: Option<&'buffer Dictionary>,
    ) -> FilterResult<'buffer, Self> {
        // REFERENCE: [Table 92 — Additional abbreviations in an inline image
        // object, p269]
        match *name.deref() {
            b"AHx" | b"ASCIIHexDecode" => Ok(Self::AHx(AHx)),
            b"A85" | b"ASCII85Decode" => Ok(Self::A85(A85)),
            b"LZW" | b"LZWDecode" => Ok(Self::Lzw(Lzw::new(decode_parms)?)),
            b"Fl" | b"FlateDecode" => Ok(Self::Fl(Fl::new(decode_parms)?)),
            b"RL" | b"RunLengthDecode" => Ok(Self::RL(RL)),
            b"CCF" | b"CCITTFaxDecode" => Ok(Self::Ccf(Ccf::new(decode_parms)?)),
            b"JBIG2Decode" => Ok(Self::JBiG2(JBiG2::new(decode_parms)?)),
            b"DCT" | b"DCTDecode" => Ok(Self::Dct(Dct::new(decode_parms)?)),
            b"JPXDecode" => Ok(Self::Jpx(Jpx)),
            b"Crypt" => Ok(Self::Crypt(Crypt::new(decode_parms)?)),
            _ => Err(FilterErr::new(
                stringify!(Filtering),
                FilterErrorCode::Unsupported(name),
            )),
        }
    }
}

impl<'buffer> Filter<'buffer> for Filtering {
    fn filter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        match self {
            Self::None => Ok(bytes.into()),
            Self::AHx(filtering) => filtering.filter(bytes),
            Self::A85(filtering) => filtering.filter(bytes),
            Self::Lzw(filtering) => filtering.filter(bytes),
            Self::Fl(filtering) => filtering.filter(bytes),
            Self::RL(filtering) => filtering.filter(bytes),
            Self::Ccf(filtering) => filtering.filter(bytes),
            Self::JBiG2(filtering) => filtering.filter(bytes),
            Self::Dct(filtering) => filtering.filter(bytes),
            Self::Jpx(filtering) => filtering.filter(bytes),
            Self::Crypt(filtering) => filtering.filter(bytes),
        }
    }

    fn defilter(
        &self,
        bytes: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        match self {
            Self::None => Ok(bytes.into()),
            Self::AHx(filtering) => filtering.defilter(bytes),
            Self::A85(filtering) => filtering.defilter(bytes),
            Self::Lzw(filtering) => filtering.defilter(bytes),
            Self::Fl(filtering) => filtering.defilter(bytes),
            Self::RL(filtering) => filtering.defilter(bytes),
            Self::Ccf(filtering) => filtering.defilter(bytes),
            Self::JBiG2(filtering) => filtering.defilter(bytes),
            Self::Dct(filtering) => filtering.defilter(bytes),
            Self::Jpx(filtering) => filtering.defilter(bytes),
            Self::Crypt(filtering) => filtering.defilter(bytes),
        }
    }
}

mod convert {
    use super::error::FilterErr;
    use super::*;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::DirectValue;

    impl FilteringChain {
        /// REFERENCE: [7.3.8.2 Stream extent, p31-33]
        pub(crate) fn new<'buffer>(dictionary: &'buffer Dictionary) -> FilterResult<'buffer, Self> {
            // TODO Move this to a separate function that `parses` the stream
            // dictionary according to a specific schema.
            let filtering = if dictionary.opt_get(KEY_F).is_some() {
                dictionary.opt_get(KEY_FFILTER)
            } else {
                dictionary.opt_get(KEY_FILTER)
            };
            let decode_pars = if dictionary.opt_get(KEY_F).is_some() {
                dictionary.opt_get(KEY_FDECODEPARMS)
            } else {
                dictionary.opt_get(KEY_DECODEPARMS)
            };

            let filter_chain = match (filtering, decode_pars) {
                (
                    Some(DirectValue::Name(filtering)),
                    Some(DirectValue::Dictionary(decode_pars)),
                ) => {
                    vec![Filtering::new(filtering, Some(decode_pars))?]
                }
                (Some(DirectValue::Name(filtering)), None) => {
                    vec![Filtering::new(filtering, None)?]
                }
                (Some(DirectValue::Array(filterings)), Some(DirectValue::Array(decode_pars))) => {
                    if filterings.len() != decode_pars.len() {
                        return Err(FilterErr::new(
                            stringify!(FilteringChain),
                            FilterErrorCode::Mismatch(filterings.len(), decode_pars.len()),
                        ));
                    }
                    filterings
                        .iter()
                        .zip(decode_pars.iter())
                        .map(|(filtering, decode_pars)| match (filtering, decode_pars) {
                            (
                                DirectValue::Name(filtering),
                                DirectValue::Dictionary(decode_pars),
                            ) => Ok(Filtering::new(filtering, Some(decode_pars))?),
                            (DirectValue::Name(filtering), DirectValue::Null(_)) => {
                                Ok(Filtering::new(filtering, None)?)
                            }
                            (DirectValue::Name(_), _) => Err(FilterErr::new(
                                KEY_DECODEPARMS,
                                FilterErrorCode::ValueType(stringify!(Dictionary), decode_pars),
                            )),
                            _ => Err(FilterErr::new(
                                KEY_FILTER,
                                FilterErrorCode::ValueType(stringify!(Name), filtering),
                            )),
                        })
                        .collect::<FilterResult<_>>()?
                }
                (Some(DirectValue::Array(filtersing)), None) => filtersing
                    .iter()
                    .map(|filtering| -> FilterResult<Filtering> {
                        if let DirectValue::Name(filtering) = filtering {
                            Ok(Filtering::new(filtering, None)?)
                        } else {
                            Err(FilterErr::new(
                                KEY_FILTER,
                                FilterErrorCode::ValueType(stringify!(Name), filtering),
                            ))
                        }
                    })
                    .collect::<FilterResult<_>>()?,
                (None, _) => {
                    vec![Filtering::None]
                }
                (Some(filtering), _) => {
                    return Err(FilterErr::new(
                        KEY_FILTER,
                        FilterErrorCode::ValueType(stringify!(Name | Array), filtering),
                    ));
                }
            };

            Ok(Self(filter_chain))
        }
    }
}

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! strict_stream_defilter_filter {
        ($buffer:expr, $expected:expr) => {
            let (_, stream) = Stream::parse($buffer).unwrap();
            let defiltered = stream.defilter().unwrap();
            assert_eq!(defiltered, $expected);
            let refiltered = stream.filter_buffer(defiltered.as_slice()).unwrap();
            assert_eq!(refiltered, stream.data);
        };
    }

    /// Some filters like `FlateDecode` and `ÀSCII85Decode` do not necessarily
    /// produce the same output as the original stream. This is because the
    /// encoder may use a compression method different from the original PDF
    /// producer. Also, the encoder may not necessarily produce the same white
    /// space characters as the original producer in the case of
    /// `ÀSCII85Decode`.
    #[macro_export]
    macro_rules! lax_stream_defilter_filter {
        ($buffer:expr, $expected:expr) => {
            let (_, stream) = Stream::parse($buffer).unwrap();
            let defiltered = stream.defilter().unwrap();
            assert_eq!(defiltered, $expected);
            let refiltered = stream.filter_buffer(defiltered.as_slice()).unwrap();
            // assert_eq!(refiltered, stream.data);
            let redefiltered = stream.defilter_buffer(refiltered).unwrap();
            assert_eq!(redefiltered, defiltered);
        };
    }
}
