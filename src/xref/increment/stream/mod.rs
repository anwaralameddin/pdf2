pub(crate) mod entry;

use ::nom::error::Error as NomError;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::trailer::Trailer;
use crate::object::indirect::id::Id;
use crate::object::indirect::object::IndirectObject;
use crate::object::indirect::stream::Stream;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::Span;
use crate::parse::KW_ENDOBJ;
use crate::parse::KW_OBJ;
use crate::process::filter::FilteringChain;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.5.8 Cross-reference streams, p65-66]
#[derive(Debug, PartialEq)]
pub(crate) struct XRefStream<'buffer> {
    pub(crate) id: Id,
    pub(crate) stream: Stream<'buffer>,
}

impl Display for XRefStream<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}\n{}\n{}", self.id, KW_OBJ, self.stream, KW_ENDOBJ)
    }
}

impl<'buffer> Parser<'buffer> for XRefStream<'buffer> {
    fn parse_span(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
        // There is no need for extra error handling here as
        // IndirectObject::parse already distinguishes between Failure and other
        // errors
        let (remains, IndirectObject { id, value }) = Parser::parse_span(buffer, offset)?;

        let stream = Stream::try_from(value)?;

        let xref_stream = XRefStream::new(id, stream);

        let buffer = remains;

        Ok((buffer, xref_stream))
    }

    fn span(&self) -> Span {
        let start = self.id.span().start();
        let end = self.stream.span().end();
        Span::new(start, end)
    }
}

mod table {
    use ::nom::bytes::complete::take;
    use ::nom::multi::many0;
    use ::nom::sequence::tuple;
    use ::nom::Err as NomErr;
    use ::std::collections::HashSet;

    use super::entry::Entry;
    use super::error::XRefStreamErrorCode;
    use super::*;
    use crate::object::error::ObjectErr;
    use crate::object::error::ObjectErrorCode;
    use crate::process::filter::Filter;
    use crate::xref::error::XRefErr;
    use crate::xref::error::XRefResult;
    use crate::xref::increment::trailer::KEY_TYPE;
    use crate::xref::increment::trailer::KEY_W;
    use crate::xref::increment::trailer::VAL_XREF;
    use crate::xref::Table;
    use crate::xref::ToTable;
    use crate::xref_err;
    use crate::ObjectNumberOrZero;

    impl ToTable for XRefStream<'_> {
        fn to_table(&self) -> XRefResult<Table> {
            // TODO Change the errors below into warnings as long as they don't
            // prevent building the table

            self.stream
                .dictionary
                .required_name(KEY_TYPE)
                .and_then(|r#type| {
                    if r#type.ne(&VAL_XREF) {
                        Err(ObjectErr::new(
                            KEY_TYPE,
                            &self.stream.dictionary,
                            ObjectErrorCode::Name {
                                expected: VAL_XREF,
                                value: r#type,
                            },
                        ))
                    } else {
                        Ok(r#type)
                    }
                })?;
            let trailer = Trailer::try_from(&self.stream.dictionary)?;
            // W
            let w = trailer.w.ok_or_else(|| {
                ObjectErr::new(
                    KEY_W,
                    trailer.dictionary,
                    ObjectErrorCode::MissingRequiredEntry,
                )
            })?;
            // Size
            let size = trailer.size;
            // Index
            let mut index = trailer.index.as_slice();
            let default_index = [(0, size)];
            if index.is_empty() {
                index = &default_index;
            }

            let entries = Self::get_entries(&self.stream, w)?;
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
                                .ok_or(XRefStreamErrorCode::EntriesTooShort {
                                    first_object_number: *first_object_number,
                                    count: *count,
                                    index: entry_index,
                                })?;
                        let object_number = first_object_number + entry_index;

                        if !object_numbers.insert(object_number) {
                            return Err(XRefErr::DuplicateObjectNumber(object_number));
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

    impl XRefStream<'_> {
        // TODO (TEMP)
        fn get_entries<'buffer>(
            stream: &'buffer Stream<'buffer>,
            w: [usize; 3],
        ) -> XRefResult<'buffer, Vec<Entry>> {
            let [count1, count2, count3] = w;

            let decoded_data = FilteringChain::new(&stream.dictionary)?.defilter(stream.data)?;
            let buffer = decoded_data.as_slice();

            let mut parser = many0(tuple((
                take::<_, _, NomError<_>>(count1),
                take(count2),
                take(count3),
            )));

            let (buffer, entries) = parser(buffer).map_err(xref_err!(e, {
                XRefStreamErrorCode::ParseDecoded(e.input.to_vec(), e.code)
            }))?;
            if !buffer.is_empty() {
                return Err(XRefStreamErrorCode::DecodedLength(
                    [count1, count2, count3],
                    decoded_data.len(),
                )
                .into());
            }
            let entries = entries
                .into_iter()
                .map(|(field1, field2, field3)| Entry::try_from((field1, field2, field3)))
                .collect::<XRefResult<Vec<Entry>>>()?;
            Ok(entries)
        }
    }
}

mod convert {
    use super::*;

    impl<'buffer> XRefStream<'buffer> {
        pub(crate) fn new(id: Id, stream: Stream<'buffer>) -> Self {
            Self { id, stream }
        }
    }
}

pub(in crate::xref) mod error {

    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    use crate::fmt::debug_bytes;
    use crate::Byte;
    use crate::IndexNumber;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum XRefStreamErrorCode {
        // TODO (TEMP) Replace Vec<Byte> below with Span
        #[error("Parsing Decoded data. Error kind: {}. Buffer: {}", .1.description(), debug_bytes(.0))]
        ParseDecoded(Vec<Byte>, ErrorKind),
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

    use super::*;
    use crate::object::direct::array::Array;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::indirect::reference::Reference;
    use crate::parse_span_assert_eq;

    #[test]
    fn xref_stream_valid() {
        // PDF produced by pdfTeX-1.40.22
        let buffer = include_bytes!(
            "../../../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/1F0F80D27D156F7EF35B1DF40B1BD3E8_dictionary.rs");
        let xref_stream = XRefStream {
            id: unsafe { Id::new_unchecked(749, 0) },
            stream: Stream::new(dictionary, &buffer[215..1975]),
        };
        parse_span_assert_eq!(buffer, xref_stream, &buffer[1993..]);

        // PDF produced by pdfTeX-1.40.21
        let buffer = include_bytes!(
            "../../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_dictionary.rs");
        let xref_stream = XRefStream::new(
            unsafe { Id::new_unchecked(439, 0) },
            Stream::new(dictionary, &buffer[215..1304]),
        );
        parse_span_assert_eq!(buffer, xref_stream, &buffer[1322..]);

        // PDF produced by pdfTeX-1.40.21
        let buffer = include_bytes!(
            "../../../../tests/data/CD74097EBFE5D8A25FE8A229299730FA_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/CD74097EBFE5D8A25FE8A229299730FA_dictionary.rs");
        let xref_stream = XRefStream::new(
            unsafe { Id::new_unchecked(190, 0) },
            Stream::new(dictionary, &buffer[215..717]),
        );
        parse_span_assert_eq!(buffer, xref_stream, &buffer[735..]);
    }

    // TODO Add tests
    // #[test]
    // fn xref_stream_invalid() {
    //
    // }
}
