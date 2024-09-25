use super::Filter;
use crate::process::error::ProcessResult;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct RL;

impl Filter for RL {
    fn filter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement RL::filter")
    }

    fn defilter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement RL::defilter")
    }
}
