use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::io::ErrorKind;
use ::std::num::TryFromIntError;
use ::thiserror::Error;

use super::*;
use crate::object::indirect::id::Id;
use crate::parse::error::ParseErr;
use crate::process::error::ProcessErr;
use crate::Offset;

pub type PdfResult<'path, T> = Result<T, PdfErr<'path>>;

// Include the path in all PdfErr variants to simplify debugging
#[derive(Debug, Error, PartialEq, Clone)]
pub enum PdfErr<'path> {
    #[error("Parse. File: {0}. Error: {1}")]
    Parse(&'path Path, ParseErr),
    #[error("Process. File: {0}. Error: {1}")]
    Process(&'path Path, ProcessErr),
    #[error("Empty cross-reference table. File: {0}")]
    EmptyPreTable(&'path Path),
    // ::std::io::Error as IoError; does not implement PartialEq or Clone,
    // and it's mor convenient to use ::std::io::ErrorKind here instead
    #[error("Open. File: {0}. Error kind: {1}")]
    OpenFile(&'path Path, ErrorKind),
    #[error("Seek. File: {0}. Error kind: {1}")]
    Seek(&'path Path, ErrorKind),
    #[error("Read. File: {0}. Error kind: {1}")]
    ReadFile(&'path Path, ErrorKind),
}

#[derive(Debug, Error, PartialEq, Clone)]
pub struct PdfRecoverable<'path> {
    path: &'path Path,
    errors: Vec<ObjectRecoverable>,
}

impl<'path> Display for PdfRecoverable<'path> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        writeln!(
            f,
            "Combined errors. File: {}. Number of errors: {}",
            self.path.display(),
            self.errors.len()
        )?;
        for err in &self.errors {
            writeln!(f, "File: {}. Error {}", self.path.display(), err)?;
        }
        Ok(())
    }
}

// This error variant is always included in the PdfRecoverable, and
// there is no need to include the path here.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ObjectRecoverable {
    #[error("Parse. Id: {0}. Offset {1}. Error: {2}")]
    Parse(Id, u64, ParseErr),
    #[error("Convert offset to usize. Id: {0}. Offset: {1}. Error: {2}")]
    OffsetAsUsize(Id, Offset, TryFromIntError),
    #[error("Mismatched id: {0} != {1}")]
    MismatchedId(Id, Id),
}

mod convert {

    use ::std::ops::Deref;

    use super::*;

    impl<'path> PdfRecoverable<'path> {
        pub fn new(path: &'path Path, errors: Vec<ObjectRecoverable>) -> Self {
            Self { path, errors }
        }
    }

    impl<'path> Deref for PdfRecoverable<'path> {
        type Target = Vec<ObjectRecoverable>;

        fn deref(&self) -> &Self::Target {
            &self.errors
        }
    }
}
