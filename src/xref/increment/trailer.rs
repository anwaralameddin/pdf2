use ::std::collections::HashMap;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::object::direct::name::OwnedName;
use crate::object::direct::string::OwnedString;
use crate::object::direct::OwnedDirectValue;
use crate::object::indirect::reference::Reference;
use crate::IndexNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

// Common dictionary keys
const KEY_SIZE: &str = "Size";
const KEY_PREV: &str = "Prev";
// Section dictionary keys
const KEY_ENCRYPT: &str = "Encrypt";
const KEY_ID: &str = "ID";
const KEY_INFO: &str = "Info";
const KEY_ROOT: &str = "Root";
// Stream dictionary keys
const KEY_INDEX: &str = "Index";
pub(super) const KEY_TYPE: &str = "Type";
pub(super) const KEY_W: &str = "W";
pub(super) const VAL_XREF: &str = "XRef";
// Hybrid-reference file trailer dictionary keys
const KEY_XREF_STM: &str = "XRefStm";
// + Other stream dictionary keys

/// REFERENCE:
/// - [7.5.8.2 Cross-reference stream dictionary, p66],
/// - ["Table 5 — Entries common to all stream dictionaries"],
/// - [7.5.5 File trailer, p58-59],
/// - ["Table 15 — Entries in the file trailer dictionary"],
/// - [Table 17 — Additional entries specific to a cross-reference stream
/// dictionary, p66-67]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Trailer {
    size: IndexNumber,
    prev: Option<Offset>,
    // HACK: Although it is required in the standard, it's not always present
    // in the examples, especially in the cross-reference section introduced by
    // incremental updates
    root: Option<Reference>, // FIXME Reference to Dictionary
    // TODO(QUESTION): Can it be a direct object?
    encrypt: Option<Reference>, // FIXME Reference to Dictionary
    // TODO(QUESTION): Can it be a direct object?
    info: Option<Reference>, // FIXME Reference to Dictionary
    id: Option<[OwnedString; 2]>,
    xref_stm: Option<Offset>,
    r#type: Option<OwnedName>,
    index: Vec<(ObjectNumberOrZero, IndexNumber)>,
    w: Option<[usize; 3]>,
    // TODO(TEMP) Reconsider after finising NewProcessErr
    // others: HashMap<&'a Name, &'a DirectValue>,
    others: HashMap<OwnedName, OwnedDirectValue>,
}

impl Display for Trailer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "<<")?;
        writeln!(f, "{} {}", OwnedName::from(KEY_SIZE), self.size)?;
        if let Some(prev) = self.prev {
            writeln!(f, "{} {}", OwnedName::from(KEY_PREV), prev)?;
        }
        if let Some(root) = self.root {
            writeln!(f, "{} {}", OwnedName::from(KEY_ROOT), root)?;
        }
        if let Some(encrypt) = self.encrypt.as_ref() {
            writeln!(f, "{} {}", OwnedName::from(KEY_ENCRYPT), encrypt)?;
        }
        if let Some(info) = self.info {
            writeln!(f, "{} {}", OwnedName::from(KEY_INFO), info)?;
        }
        if let Some([id1, id2]) = self.id.as_ref() {
            writeln!(f, "{} [{}{}]", OwnedName::from(KEY_ID), id1, id2)?;
        }
        if let Some(xref_stm) = self.xref_stm {
            writeln!(f, "{} {}", OwnedName::from(KEY_XREF_STM), xref_stm)?;
        }
        if let Some(r#type) = self.r#type.as_ref() {
            writeln!(f, "{} {}", OwnedName::from(KEY_TYPE), r#type)?;
        }
        if !self.index.is_empty() {
            write!(f, "{} [ ", OwnedName::from(KEY_INDEX))?;
            for (first_object_number, entry_count) in self.index.iter() {
                write!(f, "{} {} ", first_object_number, entry_count)?;
            }
            writeln!(f, "]")?;
        }
        if let Some(w) = self.w {
            write!(f, "{} [ ", OwnedName::from(KEY_W))?;
            for value in w.iter() {
                write!(f, "{} ", value)?;
            }
            writeln!(f, "]")?;
        }
        for (key, value) in self.others.iter() {
            writeln!(f, "{} {}", key, value)?;
        }
        writeln!(f, ">>")
    }
}

mod convert {
    use ::std::collections::HashMap;

    use super::*;
    use crate::object::direct::dictionary::error::DataTypeError;
    use crate::object::direct::dictionary::error::MissingEntryError;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::string::OwnedString;
    use crate::object::direct::OwnedDirectValue;
    use crate::object::indirect::reference::Reference;
    use crate::object::indirect::stream::KEY_DECODEPARMS;
    use crate::object::indirect::stream::KEY_DL;
    use crate::object::indirect::stream::KEY_F;
    use crate::object::indirect::stream::KEY_FDECODEPARMS;
    use crate::object::indirect::stream::KEY_FFILTER;
    use crate::object::indirect::stream::KEY_FILTER;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::process::error::NewProcessErr;
    use crate::ObjectNumberOrZero;
    use crate::Offset;

    // TODO(TEMP) Convert Dictionary to Dictionary<'lifetime>
    impl TryFrom<&OwnedDictionary> for Trailer {
        type Error = NewProcessErr;

        fn try_from(value: &OwnedDictionary) -> Result<Self, Self::Error> {
            let size = value.get_u64(KEY_SIZE)?.ok_or(MissingEntryError {
                key: KEY_SIZE,
                data_type: stringify!(u64),
            })?;

            let prev = value.get_usize(KEY_PREV)?;

            let root = value.get_reference(KEY_ROOT)?.cloned(); // TODO (TEMP) Remove cloned

            let encrypt = value.get_reference(KEY_ENCRYPT)?.cloned(); // TODO (TEMP) Remove cloned

            let info = value.get_reference(KEY_INFO)?.cloned(); // TODO (TEMP) Remove cloned

            let id = value
                .get_array(KEY_ID)?
                .map(|array| match array.as_slice() {
                    [OwnedDirectValue::String(id_1), OwnedDirectValue::String(id_2)] => {
                        // TODO Check the string lengths and report anomalies
                        // TODO(TEMP) Remove clone
                        Ok([id_1.clone(), id_2.clone()])
                    }
                    _ => Err(DataTypeError {
                        entry: KEY_ID,
                        expected_type: stringify!([String_; 2]),
                        value: array.to_string(), // TODO (TEMP) Remove to_string()
                        object: value.to_string(), // TODO (TEMP) Remove to_string()
                    }),
                })
                .transpose()?;
            let xref_stm = value.get_usize(KEY_XREF_STM)?;

            let r#type = value.get_name(KEY_TYPE)?.cloned(); // TODO (TEMP) Remove cloned

            let w = value
                .get_array(KEY_W)?
                .map(|array| match array.as_slice() {
                    [value1, value2, value3] => {
                        let [field1, field2, field3] = [value1, value2, value3].map(|field| {
                            field.as_usize().ok_or(DataTypeError {
                                entry: KEY_W,
                                expected_type: stringify!(usize),
                                value: field.to_string(), // TODO (TEMP) Remove to_string()
                                object: value.to_string(), // TODO (TEMP) Remove to_string()
                            })
                        });
                        Ok([field1?, field2?, field3?])
                    }
                    _ => Err(DataTypeError {
                        entry: KEY_W,
                        expected_type: stringify!(an array of three integers),
                        value: array.to_string(), // TODO (TEMP) Remove to_string()
                        object: value.to_string(), // TODO (TEMP) Remove to_string()
                    }),
                })
                .transpose()?;

            let index = value
                .get_array(KEY_INDEX)?
                .map(|array| {
                    let chunks = array.chunks_exact(2);
                    if !chunks.remainder().is_empty() {
                        return Err(DataTypeError {
                            entry: KEY_INDEX,
                            expected_type: stringify!(an array of pairs of integers),
                            value: array.to_string(), // TODO (TEMP) Remove to_string()
                            object: value.to_string(), // TODO (TEMP) Remove to_string()
                        });
                    }
                    let mut index = Vec::with_capacity(array.len() / 2);
                    for chunk in chunks {
                        if let [first_object_number, entry_count] = chunk {
                            let first_object_number =
                                first_object_number.as_u64().ok_or(DataTypeError {
                                    entry: KEY_INDEX,
                                    expected_type: stringify!(ObjectNumberOrZero),
                                    value: array.to_string(), // TODO (TEMP) Remove to_string()
                                    object: value.to_string(), // TODO (TEMP) Remove to_string()
                                })?;
                            let entry_count = entry_count.as_u64().ok_or(DataTypeError {
                                entry: KEY_INDEX,
                                expected_type: stringify!(IndexNumber),
                                value: array.to_string(), // TODO (TEMP) Remove to_string()
                                object: value.to_string(), // TODO (TEMP) Remove to_string()
                            })?;
                            index.push((first_object_number, entry_count));
                        } else {
                            unreachable!(
                                "Chunks provided by chunks_exact(2) should always have 2 elements"
                            );
                        }
                    }
                    Ok(index)
                })
                .transpose()?
                .unwrap_or_default();

            let others: HashMap<_, _> = value
                .clone() // TODO(TEMP)) Remove clone
                .into_iter() // TODO(TEMP) Use iter() instead
                .filter(|(key, _)| {
                    key.ne(KEY_SIZE)
                        && key.ne(KEY_PREV)
                        && key.ne(KEY_ROOT)
                        && key.ne(KEY_ENCRYPT)
                        && key.ne(KEY_INFO)
                        && key.ne(KEY_ID)
                        && key.ne(KEY_XREF_STM)
                        && key.ne(KEY_TYPE)
                        && key.ne(KEY_INDEX)
                        && key.ne(KEY_W)
                })
                .collect();

            // Report non-expected additional/missing entries in the trailer dictionar
            if root.is_none() {
                eprintln!("Trailer is missing the required entry: Root");
            }
            for (key, value) in others.iter() {
                // REFERENCE: [Table 5 — Entries common to all stream
                // dictionaries, p32-33]
                if key.ne(KEY_LENGTH)
                    && key.ne(KEY_FILTER)
                    && key.ne(KEY_DECODEPARMS)
                    && key.ne(KEY_F)
                    && key.ne(KEY_FFILTER)
                    && key.ne(KEY_FDECODEPARMS)
                    && key.ne(KEY_DL)
                {
                    eprintln!("Trailer contains additional entry: {} {}", key, value);
                }
            }

            Ok(Trailer {
                size,
                prev,
                root,
                encrypt,
                info,
                id,
                xref_stm,
                r#type,
                index,
                w,
                others,
            })
        }
    }

    impl Trailer {
        pub(crate) fn new(size: IndexNumber) -> Self {
            Self {
                size,
                prev: Default::default(),
                root: Default::default(),
                encrypt: Default::default(),
                info: Default::default(),
                id: Default::default(),
                xref_stm: Default::default(),
                r#type: Default::default(),
                index: Default::default(),
                w: Default::default(),
                others: Default::default(),
            }
        }

        pub(crate) fn set_size(mut self, size: IndexNumber) -> Self {
            self.size = size;
            self
        }

        pub(crate) fn set_prev(mut self, prev: Offset) -> Self {
            self.prev.replace(prev);
            self
        }

        pub(crate) fn set_root(mut self, root: Reference) -> Self {
            self.root.replace(root);
            self
        }

        pub(crate) fn set_encrypt(mut self, encrypt: Reference) -> Self {
            self.encrypt.replace(encrypt);
            self
        }

        pub(crate) fn set_info(mut self, info: Reference) -> Self {
            self.info.replace(info);
            self
        }

        pub(crate) fn set_id(mut self, id: [OwnedString; 2]) -> Self {
            self.id.replace(id);
            self
        }

        pub(crate) fn set_xref_stm(mut self, xref_stm: Offset) -> Self {
            self.xref_stm.replace(xref_stm);
            self
        }

        pub(crate) fn set_type(mut self, r#type: OwnedName) -> Self {
            self.r#type.replace(r#type);
            self
        }

        pub(crate) fn set_index(mut self, index: Vec<(ObjectNumberOrZero, IndexNumber)>) -> Self {
            self.index = index;
            self
        }

        pub(crate) fn set_w(mut self, w: [usize; 3]) -> Self {
            self.w.replace(w);
            self
        }

        pub(crate) fn set_others(mut self, others: HashMap<OwnedName, OwnedDirectValue>) -> Self {
            self.others = others;
            self
        }

        pub(crate) fn size(&self) -> IndexNumber {
            self.size
        }

        pub(crate) fn prev(&self) -> Option<Offset> {
            self.prev
        }

        pub(crate) fn root(&self) -> Option<&Reference> {
            self.root.as_ref()
        }

        pub(crate) fn encrypt(&self) -> Option<&Reference> {
            self.encrypt.as_ref()
        }

        pub(crate) fn info(&self) -> Option<&Reference> {
            self.info.as_ref()
        }

        pub(crate) fn id(&self) -> Option<&[OwnedString; 2]> {
            self.id.as_ref()
        }

        pub(crate) fn xref_stm(&self) -> Option<Offset> {
            self.xref_stm
        }

        pub(crate) fn r#type(&self) -> Option<&OwnedName> {
            self.r#type.as_ref()
        }

        pub(crate) fn index(&self) -> &[(ObjectNumberOrZero, IndexNumber)] {
            &self.index
        }

        pub(crate) fn w(&self) -> Option<&[usize; 3]> {
            self.w.as_ref()
        }

        pub(crate) fn others(&self) -> &HashMap<OwnedName, OwnedDirectValue> {
            &self.others
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::dictionary::error::MissingEntryError;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::string::OwnedHexadecimal;
    use crate::object::direct::string::OwnedLiteral;
    use crate::object::indirect::object::IndirectObject;
    use crate::object::indirect::stream::KEY_FILTER;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::parse::Parser;

    #[test]
    fn section_trailer_valid() {
        // Synthetic test
        let buffer = include_bytes!("../../../tests/data/SYNTHETIC_trailer.bin");
        let trailer = include!("../../../tests/code/SYNTHETIC_trailer.rs");
        let (_, dictionary) = OwnedDictionary::parse(buffer).unwrap();
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());

        // PDF produced by pdfTeX-1.40.16
        let buffer =
            include_bytes!("../../../tests/data/483F2EC937A8888A3F98DD1FF73B1F6B_trailer.bin");
        let trailer = include!("../../../tests/code/483F2EC937A8888A3F98DD1FF73B1F6B_trailer.rs");
        let (_, dictionary) = OwnedDictionary::parse(buffer).unwrap();
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());

        // PDF produced by pdfTeX-1.40.16
        let buffer =
            include_bytes!("../../../tests/data/8401FBC530C8AE9B8EC1425170A70921_trailer.bin");
        let trailer = include!("../../../tests/code/8401FBC530C8AE9B8EC1425170A70921_trailer.rs");
        let (_, dictionary) = OwnedDictionary::parse(buffer).unwrap();
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());

        // PDF produced by pdfunite from PDFs produced by LaTeX
        let buffer =
            include_bytes!("../../../tests/data/8E3F7CBC1ADD2112724D45EBD1E2B0C6_trailer.bin");
        let trailer = include!("../../../tests/code/8E3F7CBC1ADD2112724D45EBD1E2B0C6_trailer.rs");
        let (_, dictionary) = OwnedDictionary::parse(buffer).unwrap();
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());
    }

    #[test]
    fn stream_trailer_valid() {
        // PDF produced by pdfTeX-1.40.22
        let buffer =
            include_bytes!("../../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin");
        let (_, object) = IndirectObject::parse(buffer).unwrap();
        let dictionary = &object.value.as_stream().unwrap().dictionary;
        let trailer = Trailer::new(750)
            .set_root(unsafe { Reference::new_unchecked(747, 0) })
            .set_w([1, 3, 1])
            .set_index(vec![(0, 750)])
            .set_info(unsafe { Reference::new_unchecked(748, 0) })
            .set_id([
                OwnedHexadecimal::from("1F0F80D27D156F7EF35B1DF40B1BD3E8").into(),
                OwnedHexadecimal::from("1F0F80D27D156F7EF35B1DF40B1BD3E8").into(),
            ])
            .set_type(OwnedName::from(VAL_XREF))
            .set_others(HashMap::from_iter([
                (OwnedName::from(KEY_LENGTH), 1760.into()),
                (
                    OwnedName::from(KEY_FILTER),
                    OwnedName::from("FlateDecode").into(),
                ),
            ]));
        assert_eq!(trailer, Trailer::try_from(dictionary).unwrap());

        // TODO Add tests
    }

    #[test]
    fn trailer_invalid() {
        // Synthetic test

        // Missing required key Size
        let buffer = b"<</Root 2 0 R /Info 1 0 R>>\nstartxref\n99999\n%%EOF";
        let (_, dictionary) = OwnedDictionary::parse(buffer).unwrap();
        let parse_result = Trailer::try_from(&dictionary);
        let expected_error = MissingEntryError {
            key: KEY_SIZE,
            data_type: stringify!(u64),
        };
        assert_err_eq!(parse_result, expected_error);

        // Wrong data type for Size
        // TODO Unstanle as the dictionary is not guaranteed to be in this format
        // let buffer = b"<</Size 1.1/Root 2 0 R/Info 1 0 R>>\nstartxref\n99999\n%%EOF";
        // let (_, dictionary) = Dictionary::parse(buffer).unwrap();
        // let parse_result = Trailer::try_from(&dictionary);
        // let expected_error = DataTypeErr {
        //     key: KEY_SIZE,
        //     expected_type: stringify!(u64),
        //     value: "1.1",
        //     dictionary: "<</Size 1.1/Root 2 0 R/Info 1 0 R>>",
        // };
        // assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
