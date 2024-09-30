pub(crate) mod error;

use ::std::collections::HashMap;
use ::std::fs::File;
use ::std::io::BufReader;
use ::std::path::Path;

use self::error::PdfErrorCode;
use self::error::PdfRecoverable;
use crate::object::indirect::id::Id;
use crate::object::indirect::IndirectValue;
use crate::parse::Span;
use crate::xref::Table;
use crate::Byte;

// TODO Add support for spans within object streams
// FIXME Use object.span instead
type ObjectsInUse<'path> = HashMap<Id, (IndirectValue<'path>, Span)>;

#[derive(Debug)]
pub struct PdfBuilder<'path> {
    path: &'path Path,
    buffer: Vec<Byte>,
}

/// REFERENCE: [7.5.1 General, p53]
#[derive(Debug)]
pub struct Pdf<'path> {
    path: &'path Path,
    buffer: &'path [Byte],
    // • The trailer
    // trailer: Trailer<'path>,
    /// • The cross-reference table
    table: Table,
    // • The version of the PDF specification
    // TODO version: Version,
    /// REFERENCE:
    /// - [7.5.1 General, p53]
    /// - [7.5.3 File body, p55]
    /// • The body of a PDF file
    objects_in_use: ObjectsInUse<'path>,
    // TODO Add support for:
    // - Free objects
    // - Compressed objects
    // - comments: Vec<Comment>,
    // - spans: BTreeSet<Span>,
    errors: PdfRecoverable<'path>,
}

impl Pdf<'_> {
    pub fn status(&self) -> Result<(), &PdfRecoverable> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(&self.errors)
        }
    }

    // TODO A temporary debugging method
    pub fn summary(&self) -> String {
        format!(
            "PDF: {} # Objects: {} # Errors: {}",
            self.path.display(),
            self.objects_in_use.len(),
            self.errors.len()
        )
    }
}

mod build {

    use super::error::ObjectRecoverable;
    use super::error::PdfErr;
    use super::error::PdfErrorCode;
    use super::error::PdfResult;
    use super::*;
    use crate::object::indirect::object::IndirectObject;
    use crate::parse::Parser;
    use crate::xref::pretable::PreTable;
    use crate::xref::ToTable;

    impl<'path> PdfBuilder<'path> {
        // fn get_trailer(&'path self, pretable: PreTable<'path>) -> PdfResult<Trailer> {
        //     let mut pretable = pretable;
        //     pretable
        //         .pop()
        //         .map(|increment| increment.trailer())
        //         .transpose()
        //         .map_err(|err| PdfErr::new(self.path, PdfErrorCode::XRef(err)))?
        //         .ok_or_else(||PdfErr::new(self.path, PdfErrorCode::EmptyPreTable))
        // }

        fn parse_objects_in_use(
            &'path self,
            table: &Table,
            errors: &mut Vec<ObjectRecoverable<'path>>,
        ) -> PdfResult<ObjectsInUse> {
            // At this point, we should not immediately fail. Instead, we
            // collect all errors and report them at the end.
            let mut objects = HashMap::default();
            for (offset, id) in table.in_use.iter() {
                let (remains, object) = match IndirectObject::parse(&self.buffer[*offset..]) {
                    Ok((remains, object)) => (remains, object),
                    Err(err) => {
                        errors.push(ObjectRecoverable::Parse(*id, *offset, err));
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
                    errors.push(ObjectRecoverable::MismatchedId(*id, parsed_id));
                    // continue;
                }
                let span = Span::new(
                    *offset,
                    *offset + (self.buffer[*offset..].len() - remains.len()),
                );

                objects.insert(*id, (value, span));
            }

            Ok(objects)
        }

        pub fn build(&'path self) -> PdfResult<'path, Pdf<'path>> {
            // REFERENCE: [7.5.1 General, p54]
            // Apart from linearised PDFs, files should be read from the end
            // using the trailer and cross-reference table.
            let (_, pretable) = PreTable::parse(&self.buffer)
                .map_err(|err| PdfErr::new(self.path, PdfErrorCode::Parse(err)))?;
            let table = pretable
                .to_table()
                .map_err(|err| PdfErr::new(self.path, PdfErrorCode::XRef(err.to_string())))?;
            // let trailer = self.get_trailer(pretable)?;
            let mut errors = Vec::default();
            let objects_in_use = self.parse_objects_in_use(&table, &mut errors)?;
            let errors = PdfRecoverable::new(self.path, errors);

            Ok(Pdf {
                path: self.path,
                buffer: &self.buffer,
                // trailer,
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

    use super::error::PdfErr;
    use super::error::PdfResult;
    use super::*;

    impl<'path> PdfBuilder<'path> {
        pub fn new(path: &'path Path) -> PdfResult<Self> {
            let file = File::open(path)
                .map_err(|err| PdfErr::new(path, PdfErrorCode::OpenFile(err.kind())))?;
            let mut reader = BufReader::new(file);
            reader
                .seek(SeekFrom::Start(0))
                .map_err(|err| PdfErr::new(path, PdfErrorCode::Seek(err.kind())))?;
            let mut buffer = Vec::default();
            reader
                .read_to_end(&mut buffer)
                .map_err(|err| PdfErr::new(path, PdfErrorCode::ReadFile(err.kind())))?;
            Ok(Self { path, buffer })
        }
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
