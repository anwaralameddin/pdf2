use ::std::collections::HashMap;
use ::std::fs::File;
use ::std::io::BufReader;
use ::std::path::PathBuf;

use self::error::PdfError;
use self::error::PdfResult;
use crate::object::indirect::id::Id;
use crate::object::indirect::IndirectValue;
use crate::xref::increment::trailer::Trailer;
use crate::xref::Table;
use crate::Byte;

type ObjectsInUse = HashMap<Id, (IndirectValue, Span)>;

// TODO Add support for spans within object streams
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    start: usize,
    len: usize,
}

#[derive(Debug)]
pub struct PdfBuilder {
    path: PathBuf,
    buffer: Vec<Byte>,
}

/// REFERENCE: [7.5.1 General, p53]
#[derive(Debug)]
pub struct Pdf {
    path: PathBuf,
    buffer: Vec<Byte>,
    /// • The trailer
    trailer: Trailer,
    /// • The cross-reference table
    table: Table,
    // • The version of the PDF specification
    // TODO version: Version,
    /// REFERENCE:
    /// - [7.5.1 General, p53]
    /// - [7.5.3 File body, p55]
    /// • The body of a PDF file
    objects_in_use: ObjectsInUse,
    // TODO Add support for:
    // - Free objects
    // - Compressed objects
    // - comments: Vec<Comment>,
    // - spans: BTreeSet<Span>,
    errors: Vec<PdfError>,
}

impl Pdf {
    pub fn status(&self) -> PdfResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(PdfError::Multiple(self.errors.clone()))
        }
    }

    // TODO A temporary debugging method
    pub fn summary(&self) -> String {
        format!(
            "PDF: {:?} # Objects: {:?} # Errors: {:?}",
            self.path,
            self.objects_in_use.len(),
            self.errors.len()
        )
    }
}

mod process {

    use super::error::PdfError;
    use super::error::PdfResult;
    use super::*;
    use crate::object::indirect::object::IndirectObject;
    use crate::parse::Parser;
    use crate::xref::pretable::PreTable;
    use crate::xref::ToTable;

    impl PdfBuilder {
        fn get_trailer(&self, pretable: &PreTable) -> PdfResult<Trailer> {
            if let Some(increment) = pretable.back() {
                Ok(increment.trailer().clone())
            } else {
                Err(PdfError::EmptyPreTable(self.path.clone()))
            }
        }

        fn parse_objects_in_use(
            &mut self,
            table: &Table,
        ) -> PdfResult<(ObjectsInUse, Vec<PdfError>)> {
            // At this point, we should not immediately fail. Instead, we
            // collect all errors and report them at the end.
            let mut errors = Vec::default();
            let mut objects = HashMap::default();
            for (offset, id) in table.in_use.iter() {
                let start = match usize::try_from(*offset) {
                    Ok(offset) => offset,
                    Err(err) => {
                        errors.push(PdfError::OffsetAsUsize(*id, *offset, err));
                        continue;
                    }
                };
                let (remaining, object) = match IndirectObject::parse(&self.buffer[start..]) {
                    Ok((remaining, object)) => (remaining, object),
                    Err(err) => {
                        errors.push(PdfError::Object(*id, *offset, err));
                        continue;
                    }
                };
                // At this point, we have a valid indirect object, and there is
                // no need to skip the object on errors
                let IndirectObject {
                    id: parsed_id,
                    value,
                } = object;
                if parsed_id != *id {
                    errors.push(PdfError::MismatchedId(*id, parsed_id));
                    // continue;
                }
                let span = Span {
                    start,
                    len: self.buffer[start..].len() - remaining.len(),
                };

                objects.insert(*id, (value, span));
            }

            Ok((objects, errors))
        }

        pub fn build(mut self) -> PdfResult<Pdf> {
            // REFERENCE: [7.5.1 General, p54]
            // Apart from linearised PDFs, files should be read from the end
            // using the trailer and cross-reference table.
            let (_, pretable) = PreTable::parse(&self.buffer)?;
            let trailer = self.get_trailer(&pretable)?;
            let table = pretable.to_table()?;
            let (objects_in_use, errors) = self.parse_objects_in_use(&table)?;

            Ok(Pdf {
                path: self.path,
                buffer: self.buffer,
                trailer,
                table,
                objects_in_use,
                errors,
            })
        }
    }
}

mod convert {
    use ::std::io::Read;
    use ::std::io::Seek;
    use ::std::io::SeekFrom;

    use super::error::PdfResult;
    use super::*;

    impl PdfBuilder {
        pub fn new(path: impl Into<PathBuf>) -> PdfResult<Self> {
            let path = path.into();
            let file = File::open(&path).map_err(|err| {
                PdfError::Io(format!("Failed to open {}: {}", path.display(), err))
            })?;
            let mut reader = BufReader::new(file);
            reader.seek(SeekFrom::Start(0)).map_err(|err| {
                PdfError::Io(format!(
                    "Failed to seek the start of {}: {}",
                    path.display(),
                    err
                ))
            })?;
            let mut buffer = Vec::default();
            reader.read_to_end(&mut buffer).map_err(|err| {
                PdfError::Io(format!("Failed to read {}: {}", path.display(), err))
            })?;
            Ok(Self { path, buffer })
        }
    }
}

pub(crate) mod error {
    use ::std::num::TryFromIntError;
    use ::std::path::PathBuf;
    use ::thiserror::Error;

    use crate::object::indirect::id::Id;
    use crate::parse::error::ParseErr;
    use crate::process::error::ProcessErr;
    use crate::Offset;

    pub type PdfResult<T> = Result<T, PdfError>;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum PdfError {
        // TODO Split into different variants
        #[error("IoError: {0}")]
        Io(String),

        #[error("Parse {0}")]
        Parse(#[from] ParseErr),
        #[error("Process {0}")]
        Process(#[from] ProcessErr),

        #[error("Failed to convert offset {1} for id {0}: {2}")]
        OffsetAsUsize(Id, Offset, TryFromIntError),
        #[error("Empty cross-reference table in {0}")]
        EmptyPreTable(PathBuf),
        #[error("Reached end of file while parsing indirect object {0} at offset {1}")]
        EndOfFile(Id, Offset),
        #[error("Mismatched id: {0} != {1}")]
        MismatchedId(Id, Id),
        #[error("Parse: {0} at offset {1}: {2}")]
        Object(Id, u64, ParseErr),
        // TODO Implement Display for a wrapper around Vec<PdfError>
        #[error("Multiple errors: {0:?}")]
        Multiple(Vec<PdfError>),
    }
}

#[cfg(test)]
mod tests {

    use ::std::collections::VecDeque;
    use ::std::fs::read_dir;
    use ::std::path::PathBuf;

    use super::*;

    #[test]
    fn file_valid() {
        // TODO Ensure that the directory is not empty
        let dir = PathBuf::from("tests/data/parse/file/valid");
        let mut err_msgs = vec![];
        let mut dirs = VecDeque::from([dir]);
        while let Some(dir) = dirs.pop_back() {
            let entries = if let Ok(entries) = read_dir(&dir) {
                entries
            } else {
                eprintln!("Skip: Failed to read the directory {}", dir.display());
                continue;
            };
            for entry in entries {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push_front(path);
                    continue;
                }
                match path.extension() {
                    Some(extension)
                        if extension.to_ascii_lowercase() == "pdf" && path.is_file() =>
                    {
                        eprintln!("Path: {}", path.display());
                        let builder = PdfBuilder::new(&path).unwrap();
                        match builder.build() {
                            Ok(pdf) => {
                                println!(
                                    "{}: # Objects {:?}",
                                    path.display(),
                                    pdf.objects_in_use.len()
                                );
                            }
                            Err(err) => {
                                eprintln!("ERROR: {}: {}", path.display(), err);
                                err_msgs.push(format!("{}: {}", path.display(), err));
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
        if !err_msgs.is_empty() {
            panic!(
                "Errors: Failed to parse the cross-reference table in {} files",
                err_msgs.len()
            );
        }
    }
}
