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

use crate::object::direct::dictionary::Dictionary;
use crate::parse::character_set::eol;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::Span;
use crate::parse::KW_ENDSTREAM;
use crate::parse::KW_STREAM;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

pub(crate) const KEY_LENGTH: &str = "Length";
pub(crate) const KEY_F: &str = "F";
pub(crate) const KEY_FILTER: &str = "Filter";
pub(crate) const KEY_DECODEPARMS: &str = "DecodeParms";
pub(crate) const KEY_FFILTER: &str = "FFilter";
pub(crate) const KEY_FDECODEPARMS: &str = "FDecodeParms";
pub(crate) const KEY_DL: &str = "DL";

/// REFERENCE: [7.3.8 Stream objects, p31]
#[derive(PartialEq, Clone)]
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

impl Debug for Stream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}\n{}", self.dictionary, KW_STREAM)?;
        for &byte in self.data.iter() {
            write!(f, "{}", char::from(byte))?;
        }
        write!(f, "\n{}", KW_ENDSTREAM)
    }
}

impl<'buffer> Parser<'buffer> for Stream<'buffer> {
    /// REFERENCE: [7.3.8 Stream objects, p31-32]
    fn parse_span(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
        let size = buffer.len();

        let (remains, dictionary) = Dictionary::parse(buffer)?;

        let (remains, _) = tuple((
            opt(white_space_or_comment),
            tag(KW_STREAM),
            preceded(opt(char('\r')), char('\n')),
        ))(remains)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Stream),
                ParseErrorCode::NotFound(e.code),
            )
        ))?;
        // Here, we know that the buffer starts with a stream, and the following
        // errors should be propagated as StreamFailure

        let length = dictionary.required_usize(KEY_LENGTH).map_err(|err| {
            ParseFailure::new(
                buffer, // TODO (TEMP) Replace with dictionary.span() when implemented
                stringify!(Stream),
                ParseErrorCode::Object(err.to_string()),
            )
        })?;
        let buffer = remains;

        // FIXME Add support for indirect reference
        // DirectValue::Reference(_reference) => {
        //     todo!("Indirect reference for stream length")
        // }

        let file = dictionary.opt_get(KEY_F);
        if let Some(file) = file {
            todo!("Implement Stream with data stored in a file: {:?}", file);
        }

        let (buffer, data) = take::<_, _, NomError<_>>(length)(buffer).map_err(parse_failure!(
            e,
            ParseFailure::new(
                e.input,
                stringify!(Stream),
                ParseErrorCode::StreamData(e.code),
            )
        ))?;

        let (buffer, _) = delimited(opt(eol), tag(KW_ENDSTREAM), opt(white_space_or_comment))(
            buffer,
        )
        .map_err(parse_failure!(
            e,
            ParseFailure::new(
                e.input,
                stringify!(Stream),
                ParseErrorCode::MissingClosing(e.code),
            )
        ))?;

        let span = Span::new(offset, size - buffer.len());
        let stream = Self {
            dictionary,
            data,
            span,
        };
        Ok((buffer, stream))
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

        pub(crate) fn defilter(&'buffer self) -> FilterResult<'buffer, Vec<Byte>> {
            self.filter_chain()?.defilter(self.data)
        }

        // TODO Amend in line with the `PdFString::encode` method
        pub(crate) fn filter_buffer(
            &'buffer self,
            buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
        ) -> FilterResult<'buffer, Vec<Byte>> {
            FilteringChain::new(&self.dictionary)?.filter(buffer)
        }

        pub(crate) fn defilter_buffer(
            &'buffer self,
            buffer: impl Into<Vec<Byte>> + AsRef<[Byte]> + 'buffer,
        ) -> FilterResult<'buffer, Vec<Byte>> {
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
                .map_err(|err| EncodingErr::new(self.data, EncodingErrorCode::Filter(err.code)))?;
            encoding.decode(
                |data| {
                    filter_chain
                        .defilter(data)
                        .map_err(|err| EncodingErr::new(data, EncodingErrorCode::Filter(err.code)))
                },
                self.data,
            )
        }
    }
}

mod convert {

    use super::*;
    use crate::object::indirect::IndirectValue;
    use crate::parse::error::ParseFailure;

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

    impl<'buffer> TryFrom<IndirectValue<'buffer>> for Stream<'buffer> {
        type Error = ParseFailure<'static>;

        fn try_from(value: IndirectValue<'buffer>) -> Result<Self, Self::Error> {
            if let IndirectValue::Stream(stream) = value {
                Ok(stream)
            } else {
                Err(ParseFailure::new(
                    &[], // TODO (TEMP) Replace with value.span() when implemented
                    stringify!(Stream),
                    ParseErrorCode::WrongObjectType,
                ))
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
    use crate::object::direct::DirectValue;
    use crate::object::error::ObjectErr;
    use crate::object::error::ObjectErrorCode;
    use crate::object::indirect::reference::Reference;
    use crate::parse::error::ParseFailure;
    use crate::parse_span_assert_eq;
    use crate::Byte;

    #[test]
    fn stream_valid() {
        // A synthetic test
        let buffer = b"<</Length 0>>\nstream\n\nendstream\nendobj";
        let stream = Stream::new(
            Dictionary::from_iter([(KEY_LENGTH.into(), Integer::new(0, Span::new(10, 1)).into())]),
            "".as_bytes(),
            Span::new(0, buffer.len()),
        );
        parse_span_assert_eq!(buffer, stream, "endobj".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.bin");
        let stream: Stream =
            include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.rs");
        parse_span_assert_eq!(buffer, stream, "1 0 R\n".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_stream.bin");
        let stream: Stream =
            include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_stream.rs");
        parse_span_assert_eq!(buffer, stream, "1 0 R\n".as_bytes());

        // PDF produced by Microsoft Word for Office 365
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/B72168B54640B245A7CCF42DCDC8C026_stream.bin");
        let stream: Stream =
            include!("../../../tests/code/B72168B54640B245A7CCF42DCDC8C026_stream.rs");
        parse_span_assert_eq!(buffer, stream, "endobj\r\n".as_bytes());

        // TODO Add a stream with a length that is an indirect reference
    }

    #[test]
    fn stream_invalid() {
        // Synthetic tests
        // Stream: Length not found in stream dictionary
        let parse_result = Stream::parse_span(b"<<>>\nstream\nendstream", 0);
        let expected_error = ParseFailure::new(
            b"<<>>\nstream\nendstream", // b"<<>>"
            stringify!(Stream),
            ParseErrorCode::Object(
                ObjectErr::new(
                    KEY_LENGTH,
                    &Dictionary::default(),
                    ObjectErrorCode::MissingRequiredEntry,
                )
                .to_string(),
            ),
        );
        assert_err_eq!(parse_result, expected_error);

        // Stream: Length has the wrong type. Only NonNegative values and References are
        // allowed for Length Stream: Length of invalid value: -1
        let parse_result = Stream::parse_span(b"<</Length -1>>\nstream\nendstream", 0);
        let value: DirectValue = Integer::new(-1, Span::new(10, 2)).into();
        let expected_error = ParseFailure::new(
            b"<</Length -1>>\nstream\nendstream", // b"-1",
            stringify!(Stream),
            ParseErrorCode::Object(
                ObjectErr::new(
                    KEY_LENGTH,
                    &Dictionary::from_iter([(KEY_LENGTH.into(), value.clone())]),
                    ObjectErrorCode::Type {
                        expected_type: stringify!(usize),
                        value: &value,
                    },
                )
                .to_string(),
            ),
        );
        assert_err_eq!(parse_result, expected_error);

        // TODO StreamFailure::LengthInvalidValue should be returned on machines
        // where usize::MAX is less than u64::MAX, e.g. 32-bit systems

        // Stream: Data is too short
        let parse_result = Stream::parse_span(b"<</Length 10>>\nstream\n0123456\nendstream", 0);
        let expected_error = ParseFailure::new(
            b"dstream",
            stringify!(Stream),
            ParseErrorCode::MissingClosing(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);

        // Stream: Data is too long
        let parse_result = Stream::parse_span(b"<</Length 5>>\nstream\n0123456789\nendstream", 0);
        let expected_error = ParseFailure::new(
            b"56789\nendstream",
            stringify!(Stream),
            ParseErrorCode::MissingClosing(ErrorKind::Tag),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
