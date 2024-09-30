use super::Filter;
use crate::process::filter::error::FilterResult;
use crate::Byte;

const KEY_JBIG2_GLOBALS: &[Byte] = b"JBIG2Globals";

/// REFERENCE: [Table 12 — Optional parameter for the JBIG2Decode filter. p46]
#[derive(Debug, PartialEq, Clone, Copy)]
struct Jbig2Globals(());

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct JBiG2 {
    jbig2_globals: Jbig2Globals,
}

impl<'buffer> Filter<'buffer> for JBiG2 {
    fn filter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement JBiG2::filter")
    }

    fn defilter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement JBiG2::defilter")
    }
}

mod convert {
    use super::*;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::DirectValue;
    use crate::process::filter::error::FilterErr;
    use crate::process::filter::error::FilterErrorCode;

    impl JBiG2 {
        pub(in crate::process::filter) fn new<'buffer>(
            decode_parms: Option<&'buffer Dictionary>,
        ) -> FilterResult<'buffer, Self> {
            if let Some(decode_parms) = decode_parms {
                let jbig2_globals = decode_parms
                    .required_get(KEY_JBIG2_GLOBALS)
                    .map_err(|_| {
                        FilterErr::new(
                            stringify!(JBiG2),
                            FilterErrorCode::MissingRequiredParameter(stringify!(Jbig2Globals)),
                        )
                    })
                    .and_then(Jbig2Globals::try_from)?;

                Ok(Self { jbig2_globals })
            } else {
                Err(FilterErr::new(
                    stringify!(JBiG2),
                    FilterErrorCode::MissingRequiredParameter(stringify!(Jbig2Globals)),
                ))
            }
        }
    }

    impl<'buffer> TryFrom<&'buffer DirectValue<'buffer>> for Jbig2Globals {
        type Error = FilterErr<'buffer>;

        fn try_from(_value: &'buffer DirectValue<'buffer>) -> Result<Self, Self::Error> {
            todo!("Implement TryFrom<&DirectValue> for Jbig2Globals")
        }
    }
}
