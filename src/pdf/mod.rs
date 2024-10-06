pub(crate) mod error;

use ::std::collections::HashMap;
use ::std::fs::File;
use ::std::io::BufReader;
use ::std::path::Path;

use self::error::ObjectRecoverable;
use self::error::PdfErr;
use self::error::PdfErrorCode;
use self::error::PdfRecoverable;
use self::error::PdfResult;
use crate::object::indirect::object::IndirectObject;
use crate::parse::error::ParseErr;
use crate::parse::Parser;
use crate::parse::ResolvingParser;
use crate::parse::Span;
use crate::xref::pretable::PreTable;
use crate::xref::ObjectSpecifier;
use crate::xref::ToTable;
use crate::Byte;
use crate::GenerationNumber;
use crate::IncrementNumber;
use crate::ObjectNumber;

// TODO Add support for spans within object streams
pub(crate) type InUseObjects<'buffer> =
    HashMap<(ObjectNumber, GenerationNumber), (IndirectObject<'buffer>, IncrementNumber)>;
type OverridenObjects<'buffer> = Vec<(IndirectObject<'buffer>, IncrementNumber)>;

#[derive(Debug)]
pub struct PdfBuilder<'path> {
    path: &'path Path,
    buffer: Vec<Byte>,
    buffer_len: usize,
}

/// REFERENCE: [7.5.1 General, p53]
#[derive(Debug)]
pub struct Pdf<'path> {
    path: &'path Path,
    buffer: &'path [Byte],
    buffer_len: usize,
    // • The trailer
    // trailer: Trailer<'path>,
    /// • The cross-reference table
    pretable: PreTable<'path>,
    // • The version of the PDF specification
    // TODO version: Version,
    /// REFERENCE:
    /// - [7.5.1 General, p53]
    /// - [7.5.3 File body, p55]
    /// • The body of a PDF file
    in_use_objects: InUseObjects<'path>,
    overridden_objects: OverridenObjects<'path>,
    // TODO Add support for:
    // - Free objects
    // - Compressed objects
    // - comments: Vec<Comment>,
    // - spans: BTreeSet<Span>,
    errors: PdfRecoverable<'path>,
}

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

    fn parse_objects(
        &'path self,
        in_use: Vec<(ObjectSpecifier, Option<ParseErr>)>,
        errors: &mut Vec<ObjectRecoverable<'path>>,
    ) -> PdfResult<(InUseObjects, OverridenObjects)> {
        // At this point, we should not immediately fail. Instead, we
        // collect all errors and report them at the end.

        let mut in_use_objects = InUseObjects::default();
        let mut overridden_objects = OverridenObjects::default();
        let mut to_parse = in_use;
        // Different data structures are considered here, namely:
        // - Vec (faster iteration)
        // - BTreeSet (faster in-order buffer parsing)
        // - VecDeque (faster insertion and removal)
        // No measureable performance difference was observed between the three.
        // However, Vec::default() was more performant than
        // Vec::with_capacity(to_parse.len()) on the test set.
        let mut erroneous = Vec::default();

        // TODO Do we need to avoid resolve in the first iteration?
        loop {
            'inner: for ((object_number, generation_number, increment_number, offset), _) in
                to_parse.iter()
            {
                // TODO Check the standard on how to handle offset 0 for an
                // in-use object
                if *offset >= self.buffer_len || *offset == 0 {
                    errors.push(ObjectRecoverable::OutOfBounds(
                        *object_number,
                        *generation_number,
                        *offset,
                        self.buffer_len,
                    ));
                    continue 'inner;
                }
                // Given the above check, there is no need to check for
                // out-of-bounds offsets in the `IndirectObject::parse` method
                // or when parsing indirect objects' IDs and values. Still, this
                // does not cover cross-reference sections and streams. Hence, a
                // similar check is used in `Increment::parse`.

                let object = match IndirectObject::parse(&self.buffer, *offset, &in_use_objects) {
                    Ok(object) => object,
                    Err(err) => {
                        erroneous.push((
                            (
                                *object_number,
                                *generation_number,
                                *increment_number,
                                *offset,
                            ),
                            Some(err),
                        ));
                        continue 'inner;
                    }
                };
                // At this point, we have a valid indirect object, and there is
                // no need to skip the object on errors
                let parsed_id = object.id;
                if parsed_id.object_number != *object_number
                    || parsed_id.generation_number != *generation_number
                {
                    errors.push(ObjectRecoverable::MismatchedId(
                        *object_number,
                        *generation_number,
                        parsed_id,
                    ));
                }

                if let Some((overridden_object, overridden_increment_number)) = in_use_objects
                    .insert(
                        (*object_number, *generation_number),
                        (object, *increment_number),
                    )
                {
                    // TODO Equality should not happen, recheck the relevant
                    // portion of the code and replace the equality branch with
                    // unreachable
                    if *increment_number >= overridden_increment_number {
                        overridden_objects.push((overridden_object, overridden_increment_number));
                    } else if let Some((new_object, new_increment_number)) = in_use_objects.insert(
                        (*object_number, *generation_number),
                        (overridden_object, overridden_increment_number),
                    ) {
                        overridden_objects.push((new_object, new_increment_number));
                    }
                }
            }

            if erroneous.is_empty() || to_parse.len() == erroneous.len() {
                for ((object_number, generation_number, increment_number, offset), err) in
                    erroneous.into_iter()
                {
                    if let Some(err) = err {
                        errors.push(ObjectRecoverable::Parse(
                            object_number,
                            generation_number,
                            increment_number,
                            offset,
                            &self.buffer,
                            err,
                        ));
                    }
                }

                break;
            }
            to_parse = erroneous;
            erroneous = Vec::default();
        }

        Ok((in_use_objects, overridden_objects))
    }

    pub fn build(&'path self) -> PdfResult<'path, Pdf<'path>> {
        // REFERENCE: [7.5.1 General, p54]
        // Apart from linearised PDFs, files should be read from the end
        // using the trailer and cross-reference table.
        let pretable = PreTable::parse(&self.buffer)
            .map_err(|err| PdfErr::new(self.path, PdfErrorCode::Parse(&self.buffer, err)))?;
        let table = pretable
            .to_table()
            .map_err(|err| PdfErr::new(self.path, PdfErrorCode::XRef(&self.buffer, err)))?;
        // let trailer = self.get_trailer(pretable)?;
        let mut errors = Vec::default();
        let (in_use_objects, overridden_objects) = self.parse_objects(table.in_use, &mut errors)?;
        let errors = PdfRecoverable::new(self.path, errors);

        Ok(Pdf {
            path: self.path,
            buffer: &self.buffer,
            buffer_len: self.buffer_len,
            // trailer,
            pretable,
            in_use_objects,
            overridden_objects,
            errors,
        })
    }
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
            "PDF: {} # In-use Objects: {} # Overriden Objects: {} # Errors: {}",
            self.path.display(),
            self.in_use_objects.len(),
            self.overridden_objects.len(),
            self.errors.len()
        )
    }

    pub fn join_spans(&self) -> (Vec<Span>, Vec<Span>) {
        // TODO Rethink the algorithm. In particular, compare the performance of
        // Vec::sort_unstable, Vec::sort, and BTreeSet
        let mut spans = Vec::with_capacity(self.in_use_objects.len() + self.pretable.spans().len());
        spans.extend(
            self.in_use_objects
                .values()
                .map(|(object, _)| object.span()),
        );
        spans.extend(
            self.overridden_objects
                .iter()
                .map(|(object, _)| object.span()),
        );
        spans.extend(self.pretable.spans());
        spans.sort_unstable();
        let mut parsed = Vec::default();
        let mut not_parsed = Vec::default();
        let mut prev_start = 0usize;
        let mut prev_end = 0usize;
        for span in spans {
            if span.start() <= prev_end {
                prev_end = span.end().max(prev_end);
                continue;
            }
            if prev_start != prev_end {
                parsed.push(Span::new(prev_start, prev_end));
            }
            not_parsed.push(Span::new(prev_end, span.start()));
            prev_start = span.start();
            prev_end = span.end();
        }
        if prev_start != prev_end {
            parsed.push(Span::new(prev_start, prev_end));
        }
        if prev_end != self.buffer_len {
            not_parsed.push(Span::new(prev_end, self.buffer_len));
        }

        (parsed, not_parsed)
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
            let buffer_len = buffer.len();
            Ok(Self {
                path,
                buffer,
                buffer_len,
            })
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
        let mut erroneous = 0;
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
                        println!("Path: {}", path.display());
                        let builder = PdfBuilder::new(&path).unwrap();
                        let pdf = builder.build().unwrap();
                        match pdf.status() {
                            Ok(_) => {
                                println!(
                                    "{}: # Objects {:?}",
                                    path.display(),
                                    pdf.in_use_objects.len()
                                );

                                let (parsed, not_parsed) = pdf.join_spans();
                                println!(
                                    "INFO: Parsed spans: {}: {:?}",
                                    pdf.path.display(),
                                    parsed
                                );
                                println!(
                                    "INFO: Not parsed spans: {}: {:?}",
                                    pdf.path.display(),
                                    not_parsed
                                );
                            }
                            Err(err) => {
                                eprintln!("ERROR: {}", err);
                                erroneous += 1;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
        if erroneous != 0 {
            panic!("Errors: Failed to parse objects in {erroneous} files");
        }
    }
}
