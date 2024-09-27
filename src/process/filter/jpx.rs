use super::Filter;
use crate::process::filter::error::FilterResult;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct Jpx;

impl<'buffer> Filter<'buffer> for Jpx {
    fn filter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement Jpx::filter")
    }

    fn defilter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement Jpx::defilter")
    }
}
