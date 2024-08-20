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

use self::error::StreamFailure;
use self::error::StreamRecoverable;
use crate::fmt::debug_bytes;
use crate::object::direct::dictionary::Dictionary;
use crate::parse::character_set::eol;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::KW_ENDSTREAM;
use crate::parse::KW_STREAM;
use crate::parse_error;
use crate::parse_failure;
use crate::Byte;
use crate::Bytes;

pub(crate) const KEY_LENGTH: &str = "Length";
pub(crate) const KEY_F: &str = "F";
pub(crate) const KEY_FILTER: &str = "Filter";
pub(crate) const KEY_DECODEPARMS: &str = "DecodeParms";
pub(crate) const KEY_FFILTER: &str = "FFilter";
pub(crate) const KEY_FDECODEPARMS: &str = "FDecodeParms";
pub(crate) const KEY_DL: &str = "DL";

/// REFERENCE: [7.3.8 Stream objects, p31]
#[derive(PartialEq, Default, Clone)]
pub(crate) struct Stream {
    pub(crate) dictionary: Dictionary,
    pub(crate) data: Bytes,
}

impl Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n", self.dictionary, KW_STREAM)?;
        for &byte in self.data.iter() {
            write!(f, "{}", byte as char)?;
        }
        write!(f, "\n{}", KW_ENDSTREAM)
    }
}

impl Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}",
            self.dictionary,
            KW_STREAM,
            debug_bytes(&self.data),
            KW_ENDSTREAM
        )
    }
}

impl Parser for Stream {
    /// REFERENCE: [7.3.8 Stream objects, p31-32]
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, dictionary) = Dictionary::parse_semi_quiet::<Dictionary>(buffer)
            .unwrap_or_else(|| {
                Err(ParseErr::Error(
                    StreamRecoverable::DictionaryNotFound(debug_bytes(buffer)).into(),
                ))
            })?;

        let (buffer, _) = tuple((
            opt(white_space_or_comment),
            tag(KW_STREAM),
            preceded(opt(char('\r')), char('\n')),
        ))(buffer)
        .map_err(parse_error!(
            e,
            StreamRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(e.input),
            }
        ))?;
        // Here, we know that the buffer starts with a stream, and the following
        // errors should be propagated as StreamFailure

        let length = dictionary
            .get_u64(KEY_LENGTH)
            .map_err(|err| ParseErr::Failure(StreamFailure::LengthDatatype(err.value).into()))?
            .ok_or_else(|| {
                ParseErr::Failure(StreamFailure::LengthNotFound(dictionary.to_string()).into())
            })?;

        let length = usize::try_from(length)
            .map_err(|_| ParseErr::Failure(StreamFailure::LengthInvalidValue(length).into()))?;

        // FIXME Add support for indirect reference
        // DirectValue::Reference(_reference) => {
        //     todo!("Indirect reference for stream length")
        // }

        let file = dictionary.get(KEY_F);
        if let Some(file) = file {
            todo!("Implement Stream with data stored in a file: {:?}", file);
        }

        let (buffer, data) = take::<_, _, NomError<_>>(length)(buffer).map_err(parse_failure!(
            e,
            StreamFailure::StreamData {
                kind: e.code,
                input: debug_bytes(e.input),
            }
        ))?;

        let (buffer, _) = delimited(opt(eol), tag(KW_ENDSTREAM), opt(white_space_or_comment))(
            buffer,
        )
        .map_err(parse_failure!(
            e,
            StreamFailure::MissingClosing {
                kind: e.code,
                input: debug_bytes(e.input)
            }
        ))?;

        let stream = Self {
            dictionary,
            data: data.into(),
        };
        Ok((buffer, stream))
    }
}

mod process {
    use ::std::ffi::OsString;

    use super::*;
    use crate::process::encoding::Decoder;
    use crate::process::encoding::Encoding;
    use crate::process::error::ProcessResult;
    use crate::process::filter::Filter;
    use crate::process::filter::FilteringChain;
    use crate::Byte;

    impl Stream {
        pub(crate) fn defilter(&self) -> ProcessResult<Vec<Byte>> {
            // TODO Store the filter Chain in the Stream struct
            FilteringChain::new(&self.dictionary)?.defilter(&*self.data)
        }

        // TODO Amend in line with the `PdFString::encode` method
        pub(crate) fn filter_buffer(
            &self,
            buffer: impl Into<Vec<Byte>> + AsRef<[Byte]>,
        ) -> ProcessResult<Vec<Byte>> {
            FilteringChain::new(&self.dictionary)?.filter(buffer)
        }

        pub(crate) fn defilter_buffer(
            &self,
            buffer: impl Into<Vec<Byte>> + AsRef<[Byte]>,
        ) -> ProcessResult<Vec<Byte>> {
            FilteringChain::new(&self.dictionary)?.defilter(buffer)
        }

        pub(crate) fn decode(&self, encoding: Encoding) -> ProcessResult<OsString> {
            self.defilter()
                .and_then(|decoded| encoding.decode(&decoded))
        }
    }
}

mod convert {

    use super::*;
    use crate::object::indirect::IndirectValue;

    impl Stream {
        pub(crate) fn new(dictionary: impl Into<Dictionary>, data: impl Into<Bytes>) -> Self {
            Self {
                dictionary: dictionary.into(),
                data: data.into(),
            }
        }
    }

    impl TryFrom<IndirectValue> for Stream {
        type Error = ParseFailure;

        fn try_from(value: IndirectValue) -> Result<Self, Self::Error> {
            if let IndirectValue::Stream(stream) = value {
                Ok(stream)
            } else {
                Err(StreamFailure::WrongDataType(stringify!(Stream), value.to_string()).into())
            }
        }
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum StreamRecoverable {
        #[error("Dictionary not found: {0}")]
        DictionaryNotFound(String),
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum StreamFailure {
        #[error("Length entry not found. Dictionary: {0}")]
        LengthNotFound(String),
        #[error("Length entry has the wrong data type. Input: {0}")]
        LengthDatatype(String),
        #[error("Length entry has invalid value. Input: {0}")]
        LengthInvalidValue(u64),
        #[error("Failed to parse stream data: {kind:?}. Input: {input}")]
        StreamData { kind: ErrorKind, input: String },
        #[error("Missing Closing: {kind:?}. Input: {input}")]
        MissingClosing { kind: ErrorKind, input: String },
        #[error("Wrong data type.  Expected a {0} value, found {1}")]
        WrongDataType(&'static str, String),
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::array::Array;
    use crate::object::direct::name::Name;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::indirect::reference::Reference;
    use crate::parse_assert_eq;
    use crate::Byte;

    #[test]
    fn stream_valid() {
        // A synthetic test
        let buffer = b"<</Length 0>>\nstream\n\nendstream\nendobj";
        let stream = Stream::new(Dictionary::from_iter([(KEY_LENGTH.into(), 0.into())]), []);
        parse_assert_eq!(buffer, stream, "endobj".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.bin");
        let stream: Stream =
            include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.rs");
        parse_assert_eq!(buffer, stream, "1 0 R\n".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_stream.bin");
        let stream: Stream =
            include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_stream.rs");
        parse_assert_eq!(buffer, stream, "1 0 R\n".as_bytes());

        // PDF produced by Microsoft Word for Office 365
        let buffer: &[Byte] =
            include_bytes!("../../../tests/data/B72168B54640B245A7CCF42DCDC8C026_stream.bin");
        let stream: Stream =
            include!("../../../tests/code/B72168B54640B245A7CCF42DCDC8C026_stream.rs");
        parse_assert_eq!(buffer, stream, "endobj\r\n".as_bytes());

        // TODO Add a stream with a length that is an indirect reference
    }

    #[test]
    fn stream_invalid() {
        // Synthetic tests
        // Stream: Length not found in stream dictionary
        let parse_result = Stream::parse(b"<<>>\nstream\nendstream");
        let expected_error = ParseErr::Failure(StreamFailure::LengthNotFound("<<>>".into()).into());
        assert_err_eq!(parse_result, expected_error);

        // Stream: Length has the wrong type. Only NonNegative values and References are
        // allowed for Length Stream: Length of invalid value: -1
        let parse_result = Stream::parse(b"<</Length -1>>\nstream\nendstream");
        let expected_error =
            ParseErr::Failure(StreamFailure::LengthDatatype("-1".to_string()).into());
        assert_err_eq!(parse_result, expected_error);

        // TODO StreamFailure::LengthInvalidValue should be returned on machines
        // where usize::MAX is less than u64::MAX, e.g. 32-bit systems

        // Stream: Data is too short
        let parse_result = Stream::parse(b"<</Length 10>>\nstream\n0123456\nendstream");
        let expected_error = ParseErr::Failure(
            StreamFailure::MissingClosing {
                kind: ErrorKind::Tag,
                input: "dstream".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Stream: Data is too long
        let parse_result = Stream::parse(b"<</Length 5>>\nstream\n0123456789\nendstream");
        let expected_error = ParseErr::Failure(
            StreamFailure::MissingClosing {
                kind: ErrorKind::Tag,
                input: "56789\nendstream".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
