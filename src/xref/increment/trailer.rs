use ::std::collections::HashMap;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::fmt::debug_bytes;
use crate::object::direct::dictionary::Dictionary;
use crate::object::direct::name::Name;
use crate::object::direct::string::String_;
use crate::object::direct::DirectValue;
use crate::object::indirect::reference::Reference;
use crate::parse::Span;
use crate::Byte;
use crate::IndexNumber;
use crate::ObjectNumberOrZero;
use crate::Offset;

// Common dictionary keys
const KEY_SIZE: &[Byte] = b"Size";
pub(crate) const KEY_PREV: &[Byte] = b"Prev";
// Section dictionary keys
const KEY_ENCRYPT: &[Byte] = b"Encrypt";
const KEY_ID: &[Byte] = b"ID";
const KEY_INFO: &[Byte] = b"Info";
const KEY_ROOT: &[Byte] = b"Root";
// [Byte]eam dictionary keys
const KEY_INDEX: &[Byte] = b"Index";
pub(super) const KEY_TYPE: &[Byte] = b"Type";
pub(super) const KEY_W: &[Byte] = b"W";
pub(super) const VAL_XREF: &[Byte] = b"XRef";
// Hybrid-reference file trailer dictionary keys
const KEY_XREF_STM: &[Byte] = b"XRefStm";
// + Other stream dictionary keys

/// REFERENCE:
/// - [7.5.8.2 Cross-reference stream dictionary, p66],
/// - ["Table 5 — Entries common to all stream dictionaries"],
/// - [7.5.5 File trailer, p58-59],
/// - ["Table 15 — Entries in the file trailer dictionary"],
/// - [Table 17 — Additional entries specific to a cross-reference stream
/// dictionary, p66-67]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Trailer<'buffer> {
    pub(crate) size: IndexNumber,
    pub(crate) prev: Option<Offset>,
    // HACK: Although it is required in the standard, it's not always present
    // in the examples, especially in the cross-reference section introduced by
    // incremental updates
    pub(crate) root: Option<Reference>, // FIXME Reference to Dictionary
    // TODO(QUESTION): Can it be a direct object?
    pub(crate) encrypt: Option<Reference>, // FIXME Reference to Dictionary
    // TODO(QUESTION): Can it be a direct object?
    pub(crate) info: Option<Reference>, // FIXME Reference to Dictionary
    pub(crate) id: Option<[String_<'buffer>; 2]>,
    pub(crate) xref_stm: Option<Offset>,
    pub(crate) r#type: Option<&'buffer Name<'buffer>>,
    pub(crate) index: Vec<(ObjectNumberOrZero, IndexNumber)>,
    pub(crate) w: Option<[usize; 3]>,
    pub(crate) others: HashMap<&'buffer Vec<Byte>, &'buffer DirectValue<'buffer>>,
    pub(crate) span: Span,
    pub(crate) dictionary: &'buffer Dictionary<'buffer>, // TODO (TEMP)
}

impl Display for Trailer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "<<")?;
        writeln!(f, "/Size {}", self.size)?;
        if let Some(prev) = self.prev {
            writeln!(f, "/Prev {}", prev)?;
        }
        if let Some(root) = self.root {
            writeln!(f, "/Root {}", root)?;
        }
        if let Some(encrypt) = self.encrypt.as_ref() {
            writeln!(f, "/Encrypt {}", encrypt)?;
        }
        if let Some(info) = self.info {
            writeln!(f, "/Info {}", info)?;
        }
        if let Some([id1, id2]) = self.id.as_ref() {
            writeln!(f, "/ID [{}{}]", id1, id2)?;
        }
        if let Some(xref_stm) = self.xref_stm {
            writeln!(f, "/XRefStm {}", xref_stm)?;
        }
        if let Some(r#type) = self.r#type.as_ref() {
            writeln!(f, "/Type {}", r#type)?;
        }
        if !self.index.is_empty() {
            write!(f, "/Index [ ")?;
            for (first_object_number, entry_count) in self.index.iter() {
                write!(f, "{} {} ", first_object_number, entry_count)?;
            }
            writeln!(f, "]")?;
        }
        if let Some(w) = self.w {
            write!(f, "/w [ ")?;
            for value in w.iter() {
                write!(f, "{} ", value)?;
            }
            writeln!(f, "]")?;
        }
        for (key, value) in self.others.iter() {
            writeln!(f, "{} {}", debug_bytes(key), value)?;
        }
        writeln!(f, ">>")
    }
}

mod convert {
    use ::std::collections::HashMap;

    use super::*;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::DirectValue;
    use crate::object::error::ObjectErr;
    use crate::object::error::ObjectErrorCode;
    use crate::object::error::ObjectResult;
    use crate::object::indirect::reference::Reference;
    use crate::object::indirect::stream::KEY_DECODEPARMS;
    use crate::object::indirect::stream::KEY_DL;
    use crate::object::indirect::stream::KEY_F;
    use crate::object::indirect::stream::KEY_FDECODEPARMS;
    use crate::object::indirect::stream::KEY_FFILTER;
    use crate::object::indirect::stream::KEY_FILTER;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::parse::ObjectParser;
    use crate::xref::error::XRefErr;
    use crate::ObjectNumberOrZero;
    use crate::Offset;

    impl<'buffer> TryFrom<&'buffer Dictionary<'buffer>> for Trailer<'buffer> {
        type Error = XRefErr<'buffer>;

        fn try_from(dictionary: &'buffer Dictionary<'buffer>) -> Result<Self, Self::Error> {
            let size = dictionary.required_u64(KEY_SIZE)?;

            let prev = dictionary.opt_usize(KEY_PREV)?;

            let root = dictionary.opt_reference(KEY_ROOT)?.copied();

            let encrypt = dictionary.opt_reference(KEY_ENCRYPT)?.copied();

            let info = dictionary.opt_reference(KEY_INFO)?.copied();

            let id = dictionary
                .opt_array(KEY_ID)?
                .map(|array| {
                    match array.as_slice() {
                        [DirectValue::String(id0), DirectValue::String(id1)] => {
                            // TODO Check the string lengths and report anomalies
                            Ok([*id0, *id1])
                        }
                        _ => Err(ObjectErr::new(
                            KEY_ID,
                            dictionary,
                            ObjectErrorCode::Array {
                                value: array,
                                expected: stringify!([String_; 2]),
                            },
                        )),
                    }
                })
                .transpose()?;

            let xref_stm = dictionary.opt_usize(KEY_XREF_STM)?;

            let r#type = dictionary.opt_name(KEY_TYPE)?;

            let w = dictionary
                .opt_array(KEY_W)?
                .map(|array| match array.as_slice() {
                    [value1, value2, value3] => {
                        let [field1, field2, field3] = [value1, value2, value3].map(|field| {
                            field.as_usize().ok_or_else(|| {
                                ObjectErr::new(
                                    KEY_W,
                                    dictionary,
                                    ObjectErrorCode::Type {
                                        value: field,
                                        expected_type: stringify!(usize),
                                    },
                                )
                            })
                        });
                        Ok([field1?, field2?, field3?])
                    }
                    _ => Err(ObjectErr::new(
                        KEY_W,
                        dictionary,
                        ObjectErrorCode::Array {
                            value: array,
                            expected: stringify!(an array of three integers),
                        },
                    )),
                })
                .transpose()?;

            let index = dictionary
                .opt_array(KEY_INDEX)?
                .map(|array| {
                    let chunks = array.chunks_exact(2);
                    if !chunks.remainder().is_empty() {
                        return Err(ObjectErr::new(
                            KEY_INDEX,
                            dictionary,
                            ObjectErrorCode::Array {
                                value: array,
                                expected: stringify!(an array of pairs of integers),
                            },
                        ));
                    }
                    let mut index = Vec::with_capacity(array.len() / 2);
                    for chunk in chunks {
                        if let [first_object_number, entry_count] = chunk {
                            let first_object_number =
                                first_object_number.as_u64().ok_or_else(|| {
                                    ObjectErr::new(
                                        KEY_INDEX,
                                        dictionary,
                                        ObjectErrorCode::Type {
                                            value: first_object_number,
                                            expected_type: stringify!(ObjectNumberOrZero),
                                        },
                                    )
                                })?;
                            let entry_count = entry_count.as_u64().ok_or_else(|| {
                                ObjectErr::new(
                                    KEY_INDEX,
                                    dictionary,
                                    ObjectErrorCode::Type {
                                        value: entry_count,
                                        expected_type: stringify!(IndexNumber),
                                    },
                                )
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

            let others: HashMap<_, _> = dictionary
                .iter()
                .filter(|(key, _)| {
                    key.ne(&&KEY_SIZE)
                        && key.ne(&&KEY_PREV)
                        && key.ne(&&KEY_ROOT)
                        && key.ne(&&KEY_ENCRYPT)
                        && key.ne(&&KEY_INFO)
                        && key.ne(&&KEY_ID)
                        && key.ne(&&KEY_XREF_STM)
                        && key.ne(&&KEY_TYPE)
                        && key.ne(&&KEY_INDEX)
                        && key.ne(&&KEY_W)
                })
                .collect();

            // Report non-expected additional/missing entries in the trailer dictionar
            if root.is_none() {
                eprintln!("Trailer is missing the required entry: Root");
            }
            for (key, value) in others.iter() {
                // REFERENCE: [Table 5 — Entries common to all stream
                // dictionaries, p32-33]
                if key.ne(&&KEY_LENGTH)
                    && key.ne(&&KEY_FILTER)
                    && key.ne(&&KEY_DECODEPARMS)
                    && key.ne(&&KEY_F)
                    && key.ne(&&KEY_FFILTER)
                    && key.ne(&&KEY_FDECODEPARMS)
                    && key.ne(&&KEY_DL)
                {
                    eprintln!(
                        "Trailer contains additional entry: {} {}",
                        debug_bytes(key),
                        value
                    );
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
                span: dictionary.span(),
                dictionary,
            })
        }
    }

    impl<'buffer> Trailer<'buffer> {
        pub(crate) fn new(
            size: IndexNumber,
            span: Span,
            dictionary: &'buffer Dictionary<'buffer>,
        ) -> Self {
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
                span,
                dictionary,
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

        pub(crate) fn set_id(mut self, id: [String_<'buffer>; 2]) -> Self {
            self.id.replace(id);
            self
        }

        pub(crate) fn set_xref_stm(mut self, xref_stm: Offset) -> Self {
            self.xref_stm.replace(xref_stm);
            self
        }

        pub(crate) fn set_type(mut self, r#type: &'buffer Name<'buffer>) -> Self {
            self.r#type.replace(r#type);
            self
        }

        pub(crate) fn set_index(
            mut self,
            index: impl IntoIterator<Item = (ObjectNumberOrZero, IndexNumber)>,
        ) -> Self {
            self.index = index.into_iter().collect();
            self
        }

        pub(crate) fn set_w(mut self, w: [usize; 3]) -> Self {
            self.w.replace(w);
            self
        }

        pub(crate) fn set_others(
            mut self,
            others: impl IntoIterator<Item = (&'buffer Vec<Byte>, &'buffer DirectValue<'buffer>)>,
        ) -> Self {
            self.others = others.into_iter().collect();
            self
        }

        pub(crate) fn required_type(&self) -> ObjectResult<&Name> {
            self.r#type.ok_or_else(|| {
                ObjectErr::new(
                    KEY_TYPE,
                    self.dictionary,
                    ObjectErrorCode::MissingRequiredEntry,
                )
            })
        }

        pub(crate) fn required_w(&self) -> ObjectResult<[usize; 3]> {
            self.w.ok_or_else(|| {
                ObjectErr::new(
                    KEY_W,
                    self.dictionary,
                    ObjectErrorCode::MissingRequiredEntry,
                )
            })
        }

        pub(crate) fn span(&self) -> Span {
            self.span
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Real;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::object::error::ObjectErr;
    use crate::object::error::ObjectErrorCode;
    use crate::object::indirect::object::IndirectObject;
    use crate::object::indirect::stream::KEY_FILTER;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::object::indirect::IndirectValue;
    use crate::parse::ObjectParser;

    #[test]
    fn section_trailer_valid() {
        // Synthetic test
        let buffer = include_bytes!("../../../tests/data/SYNTHETIC_trailer.bin");
        let dictionary = Dictionary::parse(buffer, 0).unwrap();
        let trailer = include!("../../../tests/code/SYNTHETIC_trailer.rs");
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());

        // PDF produced by pdfTeX-1.40.16
        let buffer =
            include_bytes!("../../../tests/data/483F2EC937A8888A3F98DD1FF73B1F6B_trailer.bin");
        let dictionary = Dictionary::parse(buffer, 0).unwrap();
        let trailer = include!("../../../tests/code/483F2EC937A8888A3F98DD1FF73B1F6B_trailer.rs");
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());

        // PDF produced by pdfTeX-1.40.16
        let buffer =
            include_bytes!("../../../tests/data/8401FBC530C8AE9B8EC1425170A70921_trailer.bin");
        let key_rigid = b"rgid".to_vec();
        let vale_rigid: DirectValue = Literal::from((
            "PB:318039020_AS:510882528206848@1498815294792",
            Span::new(120, 47),
        ))
        .into();
        let key_habibi = b"habibi-version".to_vec();
        let val_habibi: DirectValue = Literal::from(("8.12.0", Span::new(184, 8))).into();
        let key_comunity = b"comunity-version".to_vec();
        let val_comunity: DirectValue = Literal::from(("v189.11.0", Span::new(211, 11))).into();
        let key_worker = b"worker-version".to_vec();
        let val_worker: DirectValue = Literal::from(("8.12.0", Span::new(239, 8))).into();
        let key_dd = b"dd".to_vec();
        let val_dd: DirectValue = Literal::from(("1498815349362", Span::new(252, 15))).into();

        let dictionary = Dictionary::parse(buffer, 0).unwrap();
        let trailer = include!("../../../tests/code/8401FBC530C8AE9B8EC1425170A70921_trailer.rs");
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());

        // PDF produced by pdfunite from PDFs produced by LaTeX
        let buffer =
            include_bytes!("../../../tests/data/8E3F7CBC1ADD2112724D45EBD1E2B0C6_trailer.bin");
        let dictionary = Dictionary::parse(buffer, 0).unwrap();
        let trailer = include!("../../../tests/code/8E3F7CBC1ADD2112724D45EBD1E2B0C6_trailer.rs");
        assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());
    }

    #[test]
    fn stream_trailer_valid() {
        // PDF produced by pdfTeX-1.40.22
        let buffer =
            include_bytes!("../../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin");
        let object = IndirectObject::parse(buffer, 0).unwrap();
        let val_ref = Name::new(VAL_XREF, Span::new(19, 5));
        let key_length = KEY_LENGTH.to_vec();

        let val_length: DirectValue = Integer::new(1760, Span::new(173, 4)).into();
        let key_filter = KEY_FILTER.to_vec();
        let val_filter: DirectValue = Name::from(("FlateDecode", Span::new(192, 12))).into();

        if let IndirectValue::Stream(stream) = object.value {
            let dictionary = stream.dictionary;
            let trailer = Trailer::new(750, Span::new(10, 197), &dictionary)
                .set_root(unsafe { Reference::new_unchecked(747, 0, 67, 7) })
                .set_w([1, 3, 1])
                .set_index([(0, 750)])
                .set_info(unsafe { Reference::new_unchecked(748, 0, 81, 7) })
                .set_id([
                    Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(94, 34)))
                        .into(),
                    Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(129, 34)))
                        .into(),
                ])
                .set_type(&val_ref)
                .set_others([(&key_length, &val_length), (&key_filter, &val_filter)]);
            assert_eq!(trailer, Trailer::try_from(&dictionary).unwrap());
        } else {
            panic!("Expected an indirect object with a stream value");
        }

        // TODO Add tests
    }

    #[test]
    fn trailer_invalid() {
        // Synthetic test

        // Wrong data type for Size
        // FIXME  Unstanle as the dictionary is not guaranteed to be in this format

        // Missing required key Size
        let buffer = b"<</Root 2 0 R /Info 1 0 R>>\nstartxref\n99999\n%%EOF";
        let dictionary = Dictionary::parse(buffer, 0).unwrap();
        let parse_result = Trailer::try_from(&dictionary);

        let expected_error =
            ObjectErr::new(KEY_SIZE, &dictionary, ObjectErrorCode::MissingRequiredEntry);
        assert_err_eq!(parse_result, expected_error);

        let buffer = b"<</Size 1.1/Root 2 0 R/Info 1 0 R>>\nstartxref\n99999\n%%EOF";
        let dictionary = Dictionary::parse(buffer, 0).unwrap();
        let parse_result = Trailer::try_from(&dictionary);
        let value: DirectValue = Real::new(1.1, Span::new(8, 3)).into();
        let expected_error = ObjectErr::new(
            KEY_SIZE,
            &dictionary,
            ObjectErrorCode::Type {
                value: &value,
                expected_type: stringify!(u64),
            },
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
