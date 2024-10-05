pub(crate) mod entry;

use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::trailer::Trailer;
use crate::object::indirect::id::Id;
use crate::object::indirect::object::IndirectObject;
use crate::object::indirect::stream::Stream;
use crate::object::indirect::IndirectValue;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::pdf::InUseObjects;
use crate::parse::ObjectParser;
use crate::parse::ResolvingParser;
use crate::parse::Span;
use crate::parse::KW_ENDOBJ;
use crate::parse::KW_OBJ;
use crate::process::filter::FilteringChain;
use crate::xref::startxref::StartXRef;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.5.8 Cross-reference streams, p65-66]
#[derive(Debug, PartialEq)]
pub(crate) struct XRefStream<'buffer> {
    pub(crate) id: Id,
    pub(crate) stream: Stream<'buffer>,
    // [F.3.4 First-page cross-reference table and trailer (Part 3), p885]
    // For linearised PDF files, the dummy cross-reference table offset is
    // optional
    // TODO Validate the value of startxref
    pub(crate) startxref: Option<StartXRef>,
    pub(crate) span: Span,
}

impl Display for XRefStream<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}\n{}\n{}", self.id, KW_OBJ, self.stream, KW_ENDOBJ)
    }
}

impl<'buffer> ObjectParser<'buffer> for XRefStream<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<'buffer, Self> {
        let start = offset;

        // [Table 17 — Additional entries specific to a cross-reference stream
        // dictionary, p66-67]
        // The entire in the trailer dictionary needed to parse the
        // cross-reference stream should never be references, and we can safely
        // use ParseObjects::default() here.
        let parsed_objects = InUseObjects::default();
        // There is no need for extra error handling here as
        // IndirectObject::parse already distinguishes between Failure and other
        // errors
        let IndirectObject { id, value, span } =
            ResolvingParser::parse(buffer, offset, &parsed_objects)?;
        let mut offset = span.end();
        let remains = &buffer[offset..];

        let stream = if let IndirectValue::Stream(stream) = value {
            stream
        } else {
            return Err(ParseFailure::new(
                &buffer[value.span()],
                stringify!(XRefStream),
                ParseErrorCode::RecMissingSubobject(
                    stringify!(Stream),
                    Box::new(ParseErrorCode::WrongObjectType),
                ),
            )
            .into());
        };

        // Skip white space and comments
        // TODO Double check if comments are allowed here
        if let Ok((_, recognised)) = recognize(opt(white_space_or_comment))(remains) {
            offset += recognised.len();
        }

        let startxref: Option<StartXRef> = StartXRef::parse_suppress_recoverable(buffer, offset)
            .transpose()
            .map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(XRefStream),
                    ParseErrorCode::RecMissingSubobject(
                        stringify!(StartXRef),
                        Box::new(err.code()),
                    ),
                )
            })?;

        let offset = startxref
            .as_ref()
            .map(|value| value.span().end())
            .unwrap_or(offset);

        let span = Span::new(start, offset);
        Ok(XRefStream::new(id, stream, startxref, span))
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod table {
    use ::nom::bytes::complete::take;
    use ::nom::multi::many0;
    use ::nom::sequence::tuple;
    use ::nom::Err as NomErr;
    use ::std::collections::HashSet;
    use ::std::ops::Deref;

    use super::entry::Entry;
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
                    if r#type.deref() != &VAL_XREF {
                        Err(ObjectErr::new(
                            KEY_TYPE,
                            self.stream.dictionary.span(),
                            ObjectErrorCode::Value {
                                expected: VAL_XREF,
                                value_span: r#type.span(),
                            },
                        ))
                    } else {
                        Ok(r#type)
                    }
                })?;
            let trailer = Trailer::try_from(&self.stream.dictionary)?;
            // W
            let w = trailer.w.ok_or_else(|| {
                ObjectErr::new(KEY_W, trailer.span(), ObjectErrorCode::MissingRequiredEntry)
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
                        let entry = entries_iter.next().ok_or(XRefErr::EntriesTooShort {
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
                            Entry::Compressed(stream_object_number, index_number) => {
                                table.insert_compressed(
                                    object_number,
                                    *stream_object_number,
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
        fn get_entries(stream: &Stream, w: [usize; 3]) -> XRefResult<Vec<Entry>> {
            let [count1, count2, count3] = w;

            let decoded_data = FilteringChain::new(&stream.dictionary)?.defilter(stream.data)?;
            let buffer = decoded_data.as_slice();

            let mut parser = many0(tuple((
                take::<_, _, NomError<_>>(count1),
                take(count2),
                take(count3),
            )));
            // - The only error that can be returned by `take` is `Err::Error`
            // - `tuple` only propagates the error from its inner parsers and
            // does not generate errors of its own
            // - Except for infinite loop check, `many0` consumes its inner
            // parser `Err::Error` and only propagates other error types
            // Therefore, `parser` above should not error out
            let (buffer, entries) =
                parser(buffer).map_err(xref_err!(e, { XRefErr::EntriesDecodedParse(e.code) }))?;
            if !buffer.is_empty() {
                return Err(XRefErr::EntriesDecodedLength(
                    [count1, count2, count3],
                    decoded_data.len(),
                ));
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
        pub(crate) fn new(
            id: Id,
            stream: Stream<'buffer>,
            startxref: Option<StartXRef>,
            span: Span,
        ) -> Self {
            Self {
                id,
                stream,
                startxref,
                span,
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::object::direct::array::Array;
    use crate::object::direct::dictionary::Dictionary;
    use crate::object::direct::name::Name;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::indirect::reference::Reference;
    use crate::parse_assert_eq;

    #[test]
    fn xref_stream_valid() {
        // PDF produced by pdfTeX-1.40.22
        let buffer = include_bytes!(
            "../../../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/1F0F80D27D156F7EF35B1DF40B1BD3E8_dictionary.rs");
        let xref_stream = XRefStream {
            id: unsafe { Id::new_unchecked(749, 0, 0, 6) },
            stream: Stream::new(dictionary, &buffer[215..1975], Span::new(10, 1986)),
            startxref: Some(StartXRef::new(365385, Span::new(1993, 2016))),
            span: Span::new(0, 2016),
        };
        parse_assert_eq!(XRefStream, buffer, xref_stream);

        // PDF produced by pdfTeX-1.40.21
        let buffer = include_bytes!(
            "../../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_dictionary.rs");
        let xref_stream = XRefStream::new(
            unsafe { Id::new_unchecked(439, 0, 0, 6) },
            Stream::new(dictionary, &buffer[215..1304], Span::new(10, 1315)),
            Some(StartXRef::new(309373, Span::new(1322, 1345))),
            Span::new(0, 1345),
        );
        parse_assert_eq!(XRefStream, buffer, xref_stream);

        // PDF produced by pdfTeX-1.40.21
        let buffer = include_bytes!(
            "../../../../tests/data/CD74097EBFE5D8A25FE8A229299730FA_xref_stream.bin"
        );
        let dictionary =
            include!("../../../../tests/code/CD74097EBFE5D8A25FE8A229299730FA_dictionary.rs");
        let xref_stream = XRefStream::new(
            unsafe { Id::new_unchecked(190, 0, 0, 6) },
            Stream::new(dictionary, &buffer[215..717], Span::new(10, 728)),
            Some(StartXRef::new(238838, Span::new(735, 758))),
            Span::new(0, 758),
        );
        parse_assert_eq!(XRefStream, buffer, xref_stream);
    }

    // TODO Add tests
    // #[test]
    // fn xref_stream_invalid() {
    //
    // }
}
