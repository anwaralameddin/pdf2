use super::Filter;
use crate::process::filter::error::FilterResult;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct RL;

impl<'buffer> Filter<'buffer> for RL {
    fn filter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement RL::filter")
    }

    fn defilter(
        &self,
        _buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
    ) -> FilterResult<'buffer, Vec<Byte>> {
        todo!("Implement RL::defilter")
    }
}
