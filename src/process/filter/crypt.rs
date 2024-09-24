use super::Filter;
use crate::object::direct::dictionary::Dictionary;
use crate::process::error::ProcessResult;
use crate::Byte;

#[derive(Debug)]
pub(super) struct Crypt {}

impl Crypt {
    pub(super) fn new(_decode_parms: Option<&Dictionary>) -> ProcessResult<Self> {
        todo!("Implement Crypt::new")
    }
}
impl Filter for Crypt {
    fn filter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Crypt::filter")
    }

    fn defilter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Crypt::defilter")
    }
}
