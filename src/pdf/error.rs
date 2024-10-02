use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::io::ErrorKind;
use ::thiserror::Error;

use super::*;
use crate::error::DisplayUsingBuffer;
use crate::object::indirect::id::Id;
use crate::parse::error::ParseErr;
use crate::xref::error::XRefErr;
use crate::GenerationNumber;
use crate::ObjectNumber;
use crate::Offset;

pub(crate) type PdfResult<'path, T> = Result<T, PdfErr<'path>>;

#[derive(Debug, Error, PartialEq, Clone)]
#[error("PDF. Error: {code}. File: {path}")]
pub struct PdfErr<'path> {
    pub(crate) path: &'path Path,
    pub(crate) code: PdfErrorCode<'path>,
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum PdfErrorCode<'path> {
    #[error("Parse. Error: {}", .1.display_using_buffer(.0))]
    Parse(&'path [Byte], ParseErr<'path>),
    #[error("XRef. Error: {}", .1.display_using_buffer(.0))]
    XRef(&'path [Byte], XRefErr),
    #[error("Empty cross-reference table")]
    EmptyPreTable,
    // ::std::io::Error as IoError; does not implement PartialEq or Clone,
    // and it's mor convenient to use ::std::io::ErrorKind here instead
    #[error("Open. Error kind: {0}")]
    OpenFile(ErrorKind),
    #[error("Seek. Error kind: {0}")]
    Seek(ErrorKind),
    #[error("Read. Error kind: {0}")]
    ReadFile(ErrorKind),
}

#[derive(Debug, Error, PartialEq, Clone)]
pub struct PdfRecoverable<'path> {
    path: &'path Path,
    errors: Vec<ObjectRecoverable<'path>>,
}

impl Display for PdfRecoverable<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        writeln!(
            f,
            "Combined errors. File: {}. Number of errors: {}",
            self.path.display(),
            self.errors.len()
        )?;
        for err in &self.errors {
            writeln!(f, "File: {}. Error: {}", self.path.display(), err)?;
        }
        Ok(())
    }
}

// This error variant is always included in the PdfRecoverable, and
// there is no need to include the path here.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ObjectRecoverable<'path> {
    #[error("Parse. Id: {0} {1}. Offset {2}. Error: {}", .4.display_using_buffer(.3))]
    Parse(
        ObjectNumber,
        GenerationNumber,
        Offset,
        &'path [Byte],
        ParseErr<'path>,
    ),
    #[error("Mismatched id: {0} {1} != {2}")]
    MismatchedId(ObjectNumber, GenerationNumber, Id),
    #[error("Object out of bounds. Id: {0} {1}. Offset: {2}. Buffer length: {3}")]
    OutOfBounds(ObjectNumber, GenerationNumber, Offset, usize),
}

mod convert {

    use ::std::ops::Deref;

    use super::*;

    impl<'path> PdfErr<'path> {
        pub fn new(path: &'path Path, code: PdfErrorCode<'path>) -> Self {
            Self { path, code }
        }
    }

    impl<'path> PdfRecoverable<'path> {
        pub fn new(path: &'path Path, errors: Vec<ObjectRecoverable<'path>>) -> Self {
            Self { path, errors }
        }
    }

    impl<'path> Deref for PdfRecoverable<'path> {
        type Target = Vec<ObjectRecoverable<'path>>;

        fn deref(&self) -> &Self::Target {
            &self.errors
        }
    }
}
