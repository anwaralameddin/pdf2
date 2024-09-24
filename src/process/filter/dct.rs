use super::Filter;
use crate::object::direct::dictionary::OwnedDictionary;
use crate::process::error::ProcessResult;
use crate::Byte;

/// The DCT (discrete cosine transform) filter
#[derive(Debug)]
pub(super) struct Dct {}

impl Dct {
    pub(super) fn new(_decode_parms: Option<&OwnedDictionary>) -> ProcessResult<Self> {
        todo!("Implement Dct::new")
    }
}

impl Filter for Dct {
    fn filter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Dct::filter")
    }

    fn defilter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Dct::defilter")
    }
}
