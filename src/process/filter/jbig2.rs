use super::Filter;
use crate::object::direct::dictionary::OwnedDictionary;
use crate::object::indirect::stream::OwnedStream;
use crate::process::error::ProcessResult;
use crate::Byte;

const KEY_JBIG2_GLOBALS: &str = "JBIG2Globals";

/// REFERENCE: [Table 12 — Optional parameter for the JBIG2Decode filter. p46]
#[derive(Debug, Clone, PartialEq)]
struct Jbig2Globals(OwnedStream);

#[derive(Debug)]
pub(super) struct JBiG2 {
    jbig2_globals: Jbig2Globals,
}

impl Filter for JBiG2 {
    fn filter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement JBiG2::filter")
    }

    fn defilter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement JBiG2::defilter")
    }
}

mod convert {
    use super::error::Jbig2Error;
    use super::*;
    use crate::object::direct::OwnedDirectValue;
    use crate::process::error::ProcessErr;

    impl JBiG2 {
        pub(in crate::process::filter) fn new(
            decode_parms: Option<&OwnedDictionary>,
        ) -> ProcessResult<Self> {
            if let Some(decode_parms) = decode_parms {
                let jbig2_globals = decode_parms
                    .get(KEY_JBIG2_GLOBALS)
                    .map(Jbig2Globals::try_from)
                    .transpose()?
                    .ok_or_else(|| Jbig2Error::Missing(stringify!(Jbig2Globals)))?;

                Ok(Self { jbig2_globals })
            } else {
                Err(Jbig2Error::Missing(stringify!(Jbig2Globals)).into())
            }
        }
    }

    impl TryFrom<&OwnedDirectValue> for Jbig2Globals {
        type Error = ProcessErr;

        fn try_from(_value: &OwnedDirectValue) -> Result<Self, Self::Error> {
            todo!("Implement TryFrom<&DirectValue> for Jbig2Globals")
        }
    }
}

pub(in crate::process) mod error {
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum Jbig2Error {
        #[error("Missing parameter: {0}")]
        Missing(&'static str),
    }
}
