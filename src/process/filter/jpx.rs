use super::Filter;
use crate::process::error::ProcessResult;
use crate::Byte;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct Jpx;

impl Filter for Jpx {
    fn filter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Jpx::filter")
    }

    fn defilter(&self, _bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        todo!("Implement Jpx::defilter")
    }
}
