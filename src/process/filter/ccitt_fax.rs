use super::Filter;
use crate::object::direct::dictionary::Dictionary;
use crate::process::error::ProcessResult;
use crate::Byte;

/// The CCITT facsimile standard filter.
#[derive(Debug)]
pub(super) struct Ccf {}

impl Ccf {
    pub(super) fn new(_decode_parms: Option<&Dictionary>) -> ProcessResult<Self> {
        todo!("Implement Ccf::new")
    }
}

impl Filter for Ccf {
    fn filter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Ccf::filter")
    }

    fn defilter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Ccf::defilter")
    }
}
