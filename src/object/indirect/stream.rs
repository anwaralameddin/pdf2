use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take;
use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::delimited;
use ::nom::sequence::preceded;
use ::nom::sequence::tuple;
use ::nom::Err as NomErr;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use nom::combinator::recognize;

use crate::object::direct::dictionary::Dictionary;
use crate::parse::character_set::eol;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::pdf::InUseObjects;
use crate::parse::ObjectParser;
use crate::parse::ResolvingParser;
use crate::parse::Span;
use crate::parse::KW_ENDSTREAM;
use crate::parse::KW_STREAM;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

pub(crate) const KEY_LENGTH: &[Byte] = b"Length";
pub(crate) const KEY_F: &[Byte] = b"F";
pub(crate) const KEY_FILTER: &[Byte] = b"Filter";
pub(crate) const KEY_DECODEPARMS: &[Byte] = b"DecodeParms";
pub(crate) const KEY_FFILTER: &[Byte] = b"FFilter";
pub(crate) const KEY_FDECODEPARMS: &[Byte] = b"FDecodeParms";
pub(crate) const KEY_DL: &[Byte] = b"DL";

/// REFERENCE: [7.3.8 Stream objects, p31]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Stream<'buffer> {
    pub(crate) dictionary: Dictionary<'buffer>,
    pub(crate) data: &'buffer [Byte],
    pub(crate) span: Span,
}

impl Display for Stream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n", self.dictionary, KW_STREAM)?;
        for &byte in self.data.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        write!(f, "\n{}", KW_ENDSTREAM)
    }
}

impl<'buffer> ResolvingParser<'buffer> for Stream<'buffer> {
    /// REFERENCE: [7.3.8 Stream objects, p31-32]
    fn parse(
        buffer: &'buffer [Byte],
        offset: Offset,
        parsed_objects: &InUseObjects<'buffer>,
    ) -> ParseResult<'buffer, Self> {
        let start = offset;

        let dictionary = Dictionary::parse(buffer, offset)?;
        let offset = dictionary.span().end();
        let remains = &buffer[offset..];

        let (remains, recognised) = recognize(tuple((
            opt(white_space_or_comment),
            tag(KW_STREAM),
            preceded(opt(char('\r')), char('\n')),
        )))(remains)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Stream),
                ParseErrorCode::NotFound(e.code),
            )
        ))?;
        let mut offset = offset + recognised.len();

        // Here, we know that the buffer starts with a stream, and the following
        // errors should be propagated as StreamFailure

        let length = dictionary
            .required_resolve_usize(KEY_LENGTH, parsed_objects)
            .map_err(|err| {
                ParseFailure::new(
                    &buffer[err.dictionary_span],
                    stringify!(Stream),
                    ParseErrorCode::Object(err),
                )
            })?;

        let file = dictionary.get(KEY_F);
        if let Some(file) = file {
            todo!("Implement Stream with data stored in a file: {:?}", file);
        }

        let (remains, data) =
            take::<_, _, NomError<_>>(length)(remains).map_err(parse_failure!(
                e,
                ParseFailure::new(
                    e.input,
                    stringify!(Stream),
                    ParseErrorCode::StreamData(e.code),
                )
            ))?;
        offset += length;

        let (_, recognised) = recognize(delimited(
            opt(eol),
            tag(KW_ENDSTREAM),
            opt(white_space_or_comment),
        ))(remains)
        .map_err(parse_failure!(
            e,
            ParseFailure::new(
                e.input,
                stringify!(Stream),
                ParseErrorCode::MissingClosing(e.code),
            )
        ))?;
        offset += recognised.len();

        let span = Span::new(start, offset);
        let stream = Self {
            dictionary,
            data,
            span,
        };
        Ok(stream)
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod filter {
    use super::*;
    use crate::process::filter::error::FilterResult;
    use crate::process::filter::Filter;
    use crate::process::filter::FilteringChain;
    use crate::Byte;

    impl<'buffer> Stream<'buffer> {
        pub(crate) fn filter_chain(&self) -> FilterResult<FilteringChain> {
            // TODO Store the filter Chain in the Stream struct
            FilteringChain::new(&self.dictionary)
        }

        pub(crate) fn defilter(&'buffer self) -> FilterResult<Vec<Byte>> {
            self.filter_chain()?.defilter(self.data)
        }

        // TODO Amend in line with the `PdFString::encode` method
        pub(crate) fn filter_buffer(
            &'buffer self,
            buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
        ) -> FilterResult<Vec<Byte>> {
            FilteringChain::new(&self.dictionary)?.filter(buffer)
        }

        pub(crate) fn defilter_buffer(
            &'buffer self,
            buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
        ) -> FilterResult<Vec<Byte>> {
            FilteringChain::new(&self.dictionary)?.defilter(buffer)
        }
    }
}

mod encode {
    use ::std::ffi::OsString;

    use super::*;
    use crate::process::encoding::error::EncodingErr;
    use crate::process::encoding::error::EncodingErrorCode;
    use crate::process::encoding::error::EncodingResult;
    use crate::process::encoding::Decoder;
    use crate::process::encoding::Encoding;
    use crate::process::filter::Filter;

    impl<'buffer> Stream<'buffer> {
        pub(crate) fn decode(&self, encoding: Encoding) -> EncodingResult<OsString> {
            let filter_chain = self
                .filter_chain()
                .map_err(|err| EncodingErr::new(self.data, EncodingErrorCode::Filter(err)))?;
            encoding.decode(
                |data| {
                    filter_chain
                        .defilter(data)
                        .map_err(|err| EncodingErr::new(data, EncodingErrorCode::Filter(err)))
                },
                self.data,
            )
        }
    }
}

mod convert {

    use super::*;

    impl<'buffer> Stream<'buffer> {
        pub(crate) fn new(
            dictionary: impl Into<Dictionary<'buffer>>,
            data: impl Into<&'buffer [Byte]>,
            span: Span,
        ) -> Self {
            Self {
                dictionary: dictionary.into(),
                data: data.into(),
                span,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::array::Array;
    use crate::object::direct::name::Name;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::error::ObjectErr;
    use crate::object::error::ObjectErrorCode;
    use crate::object::indirect::reference::Reference;
    use crate::parse::error::ParseFailure;
    use crate::res_parse_assert_eq;
    use crate::Byte;

    #[test]
    fn stream_valid() {
        // A synthetic test
        let buffer = b"<</Length 0>>\nstream\n\nendstream\nendobj";
        let stream = Stream::new(
            Dictionary::new(
                [(
                    KEY_LENGTH.to_vec(),
                    Integer::new(0, Span::new(10, 11)).into(),
                )],
                Span::new(0, 13),
            ),
            "".as_bytes(),
            Span::new(0, 32),
        );
        res_parse_assert_eq!(Stream, buffer, stream);

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.bin");
        let stream: Stream =
            include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.rs");
        res_parse_assert_eq!(Stream, buffer, stream);

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_stream.bin");
        let stream: Stream =
            include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_stream.rs");
        res_parse_assert_eq!(Stream, buffer, stream);

        // PDF produced by Microsoft Word for Office 365
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/B72168B54640B245A7CCF42DCDC8C026_stream.bin");
        let stream: Stream =
            include!("../../../tests/code/B72168B54640B245A7CCF42DCDC8C026_stream.rs");
        res_parse_assert_eq!(Stream, buffer, stream);

        // TODO Add a stream with a length that is an indirect reference
    }

    #[test]
    fn stream_invalid() {
        // Synthetic tests
        // Stream: Length not found in stream dictionary
        let parse_result = Stream::parse(b"<<>>\nstream\nendstream", 0, &InUseObjects::default());
        let expected_error = ParseFailure::new(
            b"<<>>",
            stringify!(Stream),
            ParseErrorCode::Object(ObjectErr::new(
                KEY_LENGTH,
                Span::new(0, 4),
                ObjectErrorCode::MissingRequiredEntry,
            )),
        );
        assert_err_eq!(parse_result, expected_error);

        // Stream: Length has the wrong type. Only NonNegative values and References are
        // allowed for Length Stream: Length of invalid value: -1
        let parse_result = Stream::parse(
            b"<</Length -1>>\nstream\nendstream",
            0,
            &InUseObjects::default(),
        );
        let expected_error = ParseFailure::new(
            b"<</Length -1>>",
            stringify!(Stream),
            ParseErrorCode::Object(ObjectErr::new(
                KEY_LENGTH,
                Span::new(0, 14),
                ObjectErrorCode::Type {
                    expected_type: stringify!(usize),
                    value_span: Span::new(10, 12),
                },
            )),
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO StreamFailure::LengthInvalidValue should be returned on machines
        // where usize::MAX is less than u64::MAX, e.g. 32-bit systems

        // Stream: Data is too short
        let parse_result = Stream::parse(
            b"<</Length 10>>\nstream\n0123456\nendstream",
            0,
            &InUseObjects::default(),
        );
        let expected_error = ParseFailure::new(
            b"dstream",
            stringify!(Stream),
            ParseErrorCode::MissingClosing(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Stream: Data is too long
        let parse_result = Stream::parse(
            b"<</Length 5>>\nstream\n0123456789\nendstream",
            0,
            &InUseObjects::default(),
        );
        let expected_error = ParseFailure::new(
            b"56789\nendstream",
            stringify!(Stream),
            ParseErrorCode::MissingClosing(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
