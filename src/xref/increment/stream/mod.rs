pub(crate) mod entry;

use ::nom::error::Error as NomError;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::trailer::Trailer;
use crate::object::indirect::id::Id;
use crate::object::indirect::object::IndirectObject;
use crate::object::indirect::stream::OwnedStream;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::KW_ENDOBJ;
use crate::parse::KW_OBJ;
use crate::Byte;

/// REFERENCE: [7.5.8 Cross-reference streams, p65-66]
#[derive(Debug, PartialEq)]
pub(crate) struct XRefStream {
    pub(crate) id: Id,
    pub(crate) stream: OwnedStream,
    pub(crate) trailer: Trailer,
}

impl Display for XRefStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}\n{}\n{}", self.id, KW_OBJ, self.stream, KW_ENDOBJ)
    }
}

impl Parser<'_> for XRefStream {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        // There is no need for extra error handling here as
        // IndirectObject::parse already distinguishes between Failure and other
        // errors
        let (remains, IndirectObject { id, value }) = Parser::parse(buffer)?;

        let stream = OwnedStream::try_from(value)?;

        let trailer = Trailer::try_from(&stream.dictionary).map_err(|err| ParseFailure {
            buffer, // TODO (TEMP) Remove with stream.dictionary.as_bytes() when implemented
            object: stringify!(XRefStream),
            code: ParseErrorCode::InvalidTrailerDictionary(err),
        })?;
        let buffer = remains;

        let xref_stream = XRefStream {
            id,
            stream,
            trailer,
        };
        Ok((buffer, xref_stream))
    }
}

mod process {
    use ::nom::bytes::complete::take;
    use ::nom::multi::many0;
    use ::nom::sequence::tuple;
    use ::nom::Err as NomErr;
    use ::std::collections::HashSet;

    use super::entry::error::EntryError;
    use super::entry::Entry;
    use super::error::XRefStreamError;
    use super::*;
    use crate::object::direct::dictionary::error::MissingEntryError;
    use crate::process::error::NewProcessErr;
    use crate::process::error::NewProcessResult;
    use crate::process_err;
    use crate::xref::increment::error::IncrementError;
    use crate::xref::increment::trailer::KEY_TYPE;
    use crate::xref::increment::trailer::KEY_W;
    use crate::xref::increment::trailer::VAL_XREF;
    use crate::xref::Table;
    use crate::xref::ToTable;
    use crate::ObjectNumberOrZero;

    impl ToTable for XRefStream {
        fn to_table(&self) -> NewProcessResult<Table> {
            // TODO Change the errors below into warnings as long as they don't
            // prevent building the table

            // Size
            let size = self.trailer.size();
            // Index
            let mut index = self.trailer.index();
            let default_index = [(0, size)];
            if index.is_empty() {
                index = &default_index;
            }

            let entries = self.get_entries()?;
            let mut entries_iter = entries.iter();

            let mut object_numbers: HashSet<ObjectNumberOrZero> = Default::default();

            index.iter().try_fold(
                Table::default(),
                |mut table, (first_object_number, count)| {
                    for entry_index in 0..*count {
                        // TODO We probably need a different warning when [0, size] is used
                        let entry =
                            entries_iter
                                .next()
                                .ok_or(XRefStreamError::EntriesTooShort {
                                    first_object_number: *first_object_number,
                                    count: *count,
                                    index: entry_index,
                                })?;
                        let object_number = first_object_number + entry_index;

                        if !object_numbers.insert(object_number) {
                            return Err(IncrementError::DuplicateObjectNumber(object_number).into());
                        }

                        match entry {
                            Entry::Free(next_free, generation_number) => {
                                table.insert_free(object_number, *generation_number, *next_free);
                            }
                            Entry::InUse(offset, generation_number) => {
                                table.insert_in_use(object_number, *generation_number, *offset)?;
                            }
                            Entry::Compressed(stream_id, index_number) => {
                                table.insert_compressed(
                                    object_number,
                                    *stream_id,
                                    *index_number,
                                )?;
                            }
                            Entry::NullReference(value1, value2, value3) => {
                                eprintln!("NullReference: {}, {}, {}", value1, value2, value3);
                            }
                        }
                    }
                    Ok(table)
                },
            )
        }
    }

    impl XRefStream {
        fn get_entries(&self) -> NewProcessResult<Vec<Entry>> {
            // Type
            self.trailer
                .r#type()
                .ok_or(NewProcessErr::from(MissingEntryError {
                    key: KEY_TYPE,
                    data_type: stringify!(Name),
                }))
                .and_then(|type_| {
                    if type_.ne(VAL_XREF) {
                        Err(XRefStreamError::WrongValue {
                            key: KEY_TYPE,
                            expected: VAL_XREF,
                            value: type_.clone(), // TODO (TEMP) Remove clone
                        }
                        .into())
                    } else {
                        Ok(type_)
                    }
                })?;
            // W
            let [count1, count2, count3] = self.trailer.w().ok_or(MissingEntryError {
                key: KEY_W,
                data_type: stringify!(array of three integers),
            })?;

            let decoded_data = self.stream.defilter()?;
            let buffer = decoded_data.as_slice();

            let mut parser = many0(tuple((
                take::<_, _, NomError<_>>(*count1),
                take(*count2),
                take(*count3),
            )));

            let (buffer, entries) = parser(buffer).map_err(process_err!(e, {
                XRefStreamError::ParseDecoded(e.input.to_vec(), e.code) // TODO (TEMP) Remove .to_vec()
            }))?;
            if !buffer.is_empty() {
                return Err(XRefStreamError::DecodedLength(
                    [*count1, *count2, *count3],
                    decoded_data.len(),
                )
                .into());
            }
            let entries = entries
                .into_iter()
                .map(|(field1, field2, field3)| Entry::try_from((field1, field2, field3)))
                .collect::<Result<Vec<_>, EntryError>>()?;
            Ok(entries)
        }
    }
}

mod convert {
    use super::*;
    use crate::process::error::NewProcessResult;

    impl XRefStream {
        pub(crate) fn new(id: Id, stream: &OwnedStream) -> NewProcessResult<Self> {
            let trailer = Trailer::try_from(&stream.dictionary)?;
            Ok(Self {
                id,
                stream: stream.clone(), // TODO(TEMP) Remove clone
                trailer,
            })
        }
    }
}

pub(crate) mod error {

    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    use crate::fmt::debug_bytes;
    use crate::object::direct::name::OwnedName;
    use crate::Byte;
    use crate::IndexNumber;

    // Name and Vec do not implement Copy
    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum XRefStreamError {
        #[error("Wrong value. Key {key}. Expected a {expected} value, found {value}")]
        WrongValue {
            key: &'static str,
            expected: &'static str,
            value: OwnedName, // TODO(TEMP) &'buffer [Byte]
        },
        #[error("Parsing Decoded data. Error kind: {}. Buffer: {}", .1.description(), debug_bytes(.0))]
        ParseDecoded(Vec<Byte>, ErrorKind), // TODO(TEMP) Remove Vec<Byte> with &'buffer [Byte]
        #[error("Decoded data length {1}: Not a multiple of the sum of W values: {0:?}")]
        DecodedLength([usize; 3], usize),
        #[error(
            "Entries too short. First object number: {}. Entry count: {}. Missing the {}th entry",
            first_object_number,
            count,
            index
        )]
        EntriesTooShort {
            first_object_number: u64,
            count: IndexNumber,
            index: IndexNumber,
        },
    }
}

#[cfg(test)]
mod tests {
    use ::std::collections::HashMap;

    use super::*;
    use crate::object::direct::array::OwnedArray;
    use crate::object::direct::dictionary::OwnedDictionary;
    use crate::object::direct::name::OwnedName;
    use crate::object::direct::string::OwnedHexadecimal;
    use crate::object::indirect::reference::Reference;
    use crate::object::indirect::stream::KEY_FILTER;
    use crate::object::indirect::stream::KEY_LENGTH;
    use crate::parse_assert_eq;
    use crate::xref::increment::trailer::VAL_XREF;

    #[test]
    fn xref_stream_valid() {
        // PDF produced by pdfTeX-1.40.22
        let buffer = include_bytes!(
            "../../../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/1F0F80D27D156F7EF35B1DF40B1BD3E8_dictionary.rs");
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
                (OwnedName::from(KEY_FILTER), OwnedName::from("FlateDecode").into()),
            ]));
        let xref_stream = XRefStream {
            id: unsafe { Id::new_unchecked(749, 0) },
            stream: OwnedStream::new(dictionary, &buffer[215..1975]),
            trailer,
        };
        parse_assert_eq!(buffer, xref_stream, &buffer[1993..]);

        // PDF produced by pdfTeX-1.40.21
        let buffer = include_bytes!(
            "../../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_dictionary.rs");
        let xref_stream = XRefStream::new(
            unsafe { Id::new_unchecked(439, 0) },
            &OwnedStream::new(dictionary, &buffer[215..1304]),
        )
        .unwrap();
        parse_assert_eq!(buffer, xref_stream, &buffer[1322..]);

        // PDF produced by pdfTeX-1.40.21
        let buffer = include_bytes!(
            "../../../../tests/data/CD74097EBFE5D8A25FE8A229299730FA_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/CD74097EBFE5D8A25FE8A229299730FA_dictionary.rs");
        let xref_stream = XRefStream::new(
            unsafe { Id::new_unchecked(190, 0) },
            &OwnedStream::new(dictionary, &buffer[215..717]),
        )
        .unwrap();
        parse_assert_eq!(buffer, xref_stream, &buffer[735..]);
    }

    // TODO Add tests
    // #[test]
    // fn xref_stream_invalid() {
    //
    // }
}
