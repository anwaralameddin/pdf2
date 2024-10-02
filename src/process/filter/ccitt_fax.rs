use super::Filter;
use crate::object::direct::dictionary::Dictionary;
use crate::process::filter::error::FilterResult;
use crate::Byte;

/// The CCITT facsimile standard filter.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct Ccf {}

impl Ccf {
    pub(super) fn new(_decode_parms: Option<&Dictionary>) -> FilterResult<Self> {
        todo!("Implement Ccf::new")
    }
}

impl<'buffer> Filter<'buffer> for Ccf {
    fn filter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<Vec<Byte>> {
        todo!("Implement Ccf::filter")
    }

    fn defilter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<Vec<Byte>> {
        todo!("Implement Ccf::defilter")
    }
}
