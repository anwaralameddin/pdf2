use super::Filter;
use crate::object::direct::dictionary::Dictionary;
use crate::process::filter::error::FilterResult;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct Crypt {}

impl Crypt {
    pub(super) fn new<'buffer>(
        _decode_parms: Option<&'buffer Dictionary>,
    ) -> FilterResult<'buffer, Self> {
        todo!("Implement Crypt::new")
    }
}
impl<'buffer> Filter<'buffer> for Crypt {
    fn filter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement Crypt::filter")
    }

    fn defilter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement Crypt::defilter")
    }
}
