use ::flate2::read::ZlibDecoder;
use ::flate2::read::ZlibEncoder;
use ::flate2::Compression;
use ::std::io::Read;

use self::error::FlateError;
use super::predictor::Predictor;
use super::Filter;
use crate::object::direct::dictionary::Dictionary;
use crate::process::error::ProcessResult;
use crate::Byte;

/// REFERENCE: [7.4.4 LZWDecode and FlateDecode filters, p38]
/// zlib/deflate compression filter.
#[derive(Debug, Default)]
pub(super) struct Fl {
    predictor: Predictor,
}

impl Filter for Fl {
    fn filter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        let bytes = self.predictor.filter(bytes)?;
        let mut filtered = vec![];

        let mut filter: ZlibEncoder<&[Byte]> =
            ZlibEncoder::new(bytes.as_ref(), Compression::default());
        filter
            .read_to_end(&mut filtered)
            .map_err(|err| FlateError::Filter(err.to_string()))?;

        Ok(filtered)
    }

    // TODO Replace flate2 with a library that allows restricting the output size
    fn defilter(&self, bytes: impl Into<Vec<Byte>> + AsRef<[Byte]>) -> ProcessResult<Vec<Byte>> {
        let mut defiltered = vec![];

        let mut defilter = ZlibDecoder::new(bytes.as_ref());
        defilter
            .read_to_end(&mut defiltered)
            .map_err(|err| FlateError::Defilter(err.to_string()))?;

        let defiltered = self.predictor.defilter(defiltered)?;
        Ok(defiltered)
    }
}

mod convert {
    use super::*;

    impl Fl {
        pub(in crate::process::filter) fn new(
            decode_parms: Option<&Dictionary>,
        ) -> ProcessResult<Self> {
            if let Some(decode_parms) = decode_parms {
                let predictor = Predictor::new(decode_parms)?;
                Ok(Self { predictor })
            } else {
                Ok(Self::default())
            }
        }
    }
}

pub(in crate::process) mod error {
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum FlateError {
        #[error("Filtering: {0}")]
        Filter(String),
        #[error("Defiltering: {0}")]
        Defilter(String),
    }
}

#[cfg(test)]
mod tests {

    // use super::Fl;
    // use crate::assert_err_eq;
    // use crate::object::indirect::stream::Stream;
    // use crate::parse::Parser;
    // use crate::process::filter::flate::error::FlateError;
    use crate::process::filter::tests::lax_stream_defilter_filter;

    #[test]
    fn flate_valid() {
        // NOTE: ZlibEncoder does not necessarily produce the same output as the
        // original stream. This is because the encoder may use a different
        // compression method that the original PDF producer. In fact, different
        // streams below use different compression methods, which can be seen
        // from the header bytes. E.g. the headers 0x68DE and 0x4889 appear in
        // the test data. However, ZlibEncoder with the default compression
        // level always produces the header 0x789C. Hence, the need for a lax
        // comparison.

        // PDF produced by pdfTeX-1.40.21
        let buffer =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xobject.bin");
        let expected = b"/Sh sh\n";
        lax_stream_defilter_filter(buffer, expected).unwrap();

        // PDF produced by pdfTeX-1.40.21
        let buffer =
            include_bytes!("../../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_stream.bin");
        let expected = include!("../../../tests/code/3AB9790B3CB9A73CF4BF095B2CE17671_data.rs");
        lax_stream_defilter_filter(buffer, expected).unwrap();

        // PDF produced by Microsoft Word for Office 365
        let buffer =
            include_bytes!("../../../tests/data/B72168B54640B245A7CCF42DCDC8C026_stream.bin");
        let expected = include!("../../../tests/code/B72168B54640B245A7CCF42DCDC8C026_data.rs");
        lax_stream_defilter_filter(buffer, expected).unwrap();

        // TODO Add tests
    }

    // TODO Add tests
    // #[test]
    // fn flate_invalid() {
    // let buffer = include_bytes!("../../../tests/process/filter/flate/ID.bin");
    // let (_, stream) = Stream::parse(buffer).unwrap();
    // let defiltered_result = Fl::default().defilter(&*stream.data);
    // let expected_error = FlateError::Defilter("corrupt deflate stream".to_string());
    // assert_err_eq!(defiltered_result, expected_error);
    // }
}
