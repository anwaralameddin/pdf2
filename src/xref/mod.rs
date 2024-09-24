pub(crate) mod increment;
pub(crate) mod pretable;
pub(crate) mod startxref;

use ::std::collections::BTreeSet;
use ::std::collections::HashMap;

use self::error::XRefError;
use crate::object::indirect::id::Id;
use crate::process::error::NewProcessResult;
use crate::GenerationNumber;
use crate::IndexNumber;
use crate::ObjectNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

pub(crate) trait ToTable {
    // TODO(TEMP) Consider consuming the object
    fn to_table(&self) -> NewProcessResult<Table>;
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Table {
    // TODO(QUESTION) Can the same object number and generation number be used
    // more than once? If so, add the section number to avoid collisions
    pub(crate) in_use: BTreeSet<(Offset, Id)>,
    // TODO
    // - Any need to subtarct one from the generation number to get the actual
    // freed object?
    // - (QUESTION) Can an object be free if it was never used?
    // - Validate that they partially form a linked list
    pub(crate) free: HashMap<Id, ObjectNumberOrZero>,
    pub(crate) compressed: HashMap<Id, (Id, IndexNumber)>,
}

impl Table {
    pub(super) fn insert_free(
        &mut self,
        object_number: ObjectNumberOrZero,
        generation_number: GenerationNumber,
        next_free: ObjectNumberOrZero,
    ) -> Option<ObjectNumberOrZero> {
        // Ignore the object number 0
        let object_number = ObjectNumber::new(object_number)?;
        let id = Id::new(object_number, generation_number);
        self.free.insert(id, next_free)
    }

    pub(super) fn insert_in_use(
        &mut self,
        object_number: ObjectNumberOrZero,
        generation_number: GenerationNumber,
        offset: Offset,
    ) -> Result<(), XRefError> {
        // TODO use 'static if a lifetime is introduced
        let object_number = ObjectNumber::new(object_number).ok_or(XRefError::XRefInUseObject {
            object_number,
            generation_number,
            offset,
        })?;
        let id = Id::new(object_number, generation_number);
        // TODO Check if the offset or id is already in use
        self.in_use.insert((offset, id));
        Ok(())
    }

    pub(super) fn insert_compressed(
        &mut self,
        object_number: ObjectNumberOrZero,
        stream_id: Id,
        index: IndexNumber,
    ) -> Result<Option<(Id, IndexNumber)>, XRefError> {
        // TODO use 'static if a lifetime is introduced
        let object_number =
            ObjectNumber::new(object_number).ok_or(XRefError::XRefCompressedObject {
                object_number,
                stream_id,
                index,
            })?;
        let id = Id::new(object_number, GenerationNumber::default());
        Ok(self.compressed.insert(id, (stream_id, index)))
    }

    pub(super) fn extend(&mut self, other: Table) {
        // TODO Report overriden values
        self.in_use.extend(other.in_use);
        // FIXME Be careful when extending free objects. The below does not take
        // into account objects that are reused
        self.free.extend(other.free);
        self.compressed.extend(other.compressed);
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    use crate::object::indirect::id::Id;
    use crate::GenerationNumber;
    use crate::IndexNumber;
    use crate::ObjectNumberOrZero;
    use crate::Offset;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum XRefError {
        #[error(
            "In-use Object. Number: {}, Generation: {}, Offset: {}",
            object_number,
            generation_number,
            offset
        )]
        XRefInUseObject {
            object_number: ObjectNumberOrZero,
            generation_number: GenerationNumber,
            offset: Offset,
        },
        #[error(
            "Compressed Object. Number: {}, Stream: {}, index: {}",
            object_number,
            stream_id,
            index
        )]
        XRefCompressedObject {
            object_number: ObjectNumberOrZero,
            stream_id: Id,
            index: IndexNumber,
        },
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
                        let file = File::open(&path).unwrap();
                        let mut reader = BufReader::new(file);
                        let mut buffer = vec![];
                        reader.read_to_end(&mut buffer).unwrap();
                        let pretable = PreTable::parse(&buffer);
                        match pretable {
                            Ok((_, pretable)) => {
                                pretable.to_table().unwrap();
                                println!("{}: # Increments {:?}", path.display(), pretable.len());
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
                        println!("Path: {}", path.display());
                        let file = File::open(&path).unwrap();
                        let mut reader = BufReader::new(file);
                        let mut buffer = vec![];
                        reader.read_to_end(&mut buffer).unwrap();
                        let pretable = PreTable::parse(&buffer);
                        match pretable {
                            Ok((_, pretable)) if pretable.to_table().is_ok() => {
                                eprintln!("{}: # Increments {:?}", path.display(), pretable.len());
                                err_msgs.push(path);
                            }
                            Ok(_) => {
                                println!(
                                    "{}: Successfully parsed the cross-reference table but failed \
                                     to process it",
                                    path.display()
                                );
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
