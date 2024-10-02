use super::Filter;
use crate::object::direct::dictionary::Dictionary;
use crate::process::filter::error::FilterResult;
use crate::Byte;

/// The DCT (discrete cosine transform) filter
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct Dct {}

impl Dct {
    pub(super) fn new(_decode_parms: Option<&Dictionary>) -> FilterResult<Self> {
        todo!("Implement Dct::new")
    }
}

impl<'buffer> Filter<'buffer> for Dct {
    fn filter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<Vec<Byte>> {
        todo!("Implement Dct::filter")
    }

    fn defilter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<Vec<Byte>> {
        todo!("Implement Dct::defilter")
    }
}
