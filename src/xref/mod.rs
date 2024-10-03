pub(crate) mod error;
pub(crate) mod increment;
pub(crate) mod pretable;
pub(crate) mod startxref;

use ::std::collections::HashMap;

use self::error::XRefErr;
use crate::parse::error::ParseErr;
use crate::xref::error::XRefResult;
use crate::GenerationNumber;
use crate::IndexNumber;
use crate::ObjectNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

pub(crate) trait ToTable {
    fn to_table(&self) -> XRefResult<Table>;
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Table<'buffer> {
    // TODO(QUESTION) Can the same object number and generation number be used
    // more than once? If so, add the section number to avoid collisions
    pub(crate) in_use: Vec<(
        Offset,
        (ObjectNumber, GenerationNumber),
        Option<ParseErr<'buffer>>,
    )>,
    // TODO
    // - Any need to subtarct one from the generation number to get the actual
    // freed object?
    // - (QUESTION) Can an object be free if it was never used?
    // - Validate that they partially form a linked list
    pub(crate) free: HashMap<(ObjectNumber, GenerationNumber), ObjectNumberOrZero>,
    pub(crate) compressed: HashMap<ObjectNumber, (ObjectNumber, IndexNumber)>,
    // TODO Add trailer here rather than in the Pdf struct
}

impl<'buffer> Table<'buffer> {
    pub(super) fn insert_free(
        &mut self,
        object_number: ObjectNumberOrZero,
        generation_number: GenerationNumber,
        next_free: ObjectNumberOrZero,
    ) -> Option<ObjectNumberOrZero> {
        // Ignore the object number 0
        let object_number = ObjectNumber::new(object_number)?;
        self.free
            .insert((object_number, generation_number), next_free)
    }

    pub(super) fn insert_in_use(
        &mut self,
        object_number: ObjectNumberOrZero,
        generation_number: GenerationNumber,
        offset: Offset,
    ) -> XRefResult<()> {
        let object_number = ObjectNumber::new(object_number).ok_or(XRefErr::InUseObjectNumber {
            object_number,
            generation_number,
            offset,
        })?;
        // TODO Check if the offset or id is already in use
        self.in_use
            .push((offset, (object_number, generation_number), None));
        Ok(())
    }

    pub(super) fn insert_compressed(
        &mut self,
        object_number: ObjectNumberOrZero,
        stream_object_number: ObjectNumber,
        index: IndexNumber,
    ) -> XRefResult<Option<(ObjectNumber, IndexNumber)>> {
        let object_number =
            ObjectNumber::new(object_number).ok_or(XRefErr::CompressedObjectNumber {
                object_number,
                stream_object_number,
                index,
            })?;
        Ok(self
            .compressed
            .insert(object_number, (stream_object_number, index)))
    }

    pub(super) fn extend(&mut self, other: Table<'buffer>) {
        // TODO Report overriden values
        self.in_use.extend(other.in_use);
        // FIXME Be careful when extending free objects. The below does not take
        // into account objects that are reused
        self.free.extend(other.free);
        self.compressed.extend(other.compressed);
    }
}

#[cfg(test)]
mod tests {
    use ::std::collections::VecDeque;
    use ::std::fs::read_dir;
    use ::std::fs::File;
    use ::std::io::BufReader;
    use ::std::io::Read;
    use ::std::path::PathBuf;

    use super::pretable::PreTable;
    use crate::parse::Parser;
    use crate::xref::ToTable;

    #[test]
    fn xref_valid() {
        // TODO Ensure that the directory is not empty for
        let dir = PathBuf::from("tests/data/parse/xref/valid");
        let mut err_msgs = Vec::default();
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
                        let file = File::open(&path).unwrap();
                        let mut reader = BufReader::new(file);
                        let mut buffer = Vec::default();
                        reader.read_to_end(&mut buffer).unwrap();
                        let pretable = PreTable::parse(&buffer);
                        match pretable {
                            Ok(pretable) => {
                                let pretable_len = pretable.len();
                                pretable.to_table().unwrap();
                                println!("{}: # Increments {:?}", path.display(), pretable_len);
                            }
                            Err(err) => {
                                eprintln!("{}: Error: {}", path.display(), err);
                                err_msgs.push(path);
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

    #[test]
    fn xref_invalid() {
        // TODO Ensure that the directory is not empty for
        let dir = PathBuf::from("tests/data/parse/xref/invalid");
        let mut err_msgs = Vec::default();
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
                        let file = File::open(&path).unwrap();
                        let mut reader = BufReader::new(file);
                        let mut buffer = Vec::default();
                        reader.read_to_end(&mut buffer).unwrap();
                        let pretable = PreTable::parse(&buffer);
                        let pretable_len = pretable
                            .as_ref()
                            .map(|pretable| pretable.len())
                            .unwrap_or_default();
                        match pretable {
                            Ok(pretable) => {
                                if pretable.to_table().is_ok() {
                                    eprintln!(
                                        "{}: # Increments {:?}",
                                        path.display(),
                                        pretable_len
                                    );
                                    err_msgs.push(path);
                                } else {
                                    println!(
                                        "{}: Successfully parsed the cross-reference table but \
                                         failed to process it",
                                        path.display()
                                    );
                                }
                            }
                            Err(err) => {
                                println!("{}: Error: {}", path.display(), err);
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
        if !err_msgs.is_empty() {
            panic!(
                "Errors: Successfully parsed the cross-reference table in {} invalid files",
                err_msgs.len()
            );
        }
    }
}
