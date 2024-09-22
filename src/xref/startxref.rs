use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take_until;
use ::nom::character::complete::char;
use ::nom::character::complete::digit1;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::delimited;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use error::StartXRefCode;

use crate::new_parse_failure;
use crate::parse::character_set::eol;
use crate::parse::error::NewParseErr;
use crate::parse::error::NewParseFailure;
use crate::parse::error::NewParseResult;
use crate::parse::error::ParseErrorCode;
use crate::parse::num::ascii_to_u64;
use crate::parse::NewParser;
use crate::parse::EOF;
use crate::parse::KW_STARTXREF;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.5.4 Cross-reference table] and [7.5.5 File trailer]
/// The length of the last three lines of the file is constrained to:
/// ```rs
/// KW_STARTXREF.len() + EOL::MAX_LEN + OFFSET_MAX_LEN + EOL::MAX_LEN + EOF.len() +
/// EOL::MAX_LEN = 9 + 2 + 10 + 2 + 5 + 2 = 30,
/// ```
/// assuming the same offset length constraint applies to the offset in the
/// startxref line.
const STARTXREF_MAX_SIZE: usize = 30;

/// The minimum length of the last three lines of a PDF file:
/// ```rs
/// KW_STARTXREF.len() + EOL::MIN_LEN + OFFSET_MIN_LEN + EOL::MIN_LEN =
/// 9 + 1 + 1 + 1 + 5 = 17
/// ```
const STARTXREF_MIN_SIZE: usize = 17;

/// REFERENCE: [7.5.5 File trailer, p58]
#[derive(Debug, PartialEq)]
pub(crate) struct StartXRef(Offset);

impl Display for StartXRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n{}", KW_STARTXREF, self.0, EOF)
    }
}

impl NewParser<'_> for StartXRef {
    fn parse(buffer: &[Byte]) -> NewParseResult<(&[Byte], Self)> {
        let offset = if let Some(offset) = buffer.len().checked_sub(STARTXREF_MAX_SIZE) {
            offset
        } else {
            return Err(NewParseFailure {
                buffer,
                code: StartXRefCode::TooSmallBuffer.into(),
            }
            .into());
        };
        // alt((char('\r'), char('\n'))) is used here instead of eol to allow
        // for a file ending with "%%EOF\r" instead of "%%EOF\r\n". Also,
        // ´complete` rather than `streaming` variants of `tag` and `char` are
        // used to ensure that the parser does return an Incomplete error when
        // the file ends with the EOF marker without trailing EOL characters.
        let (buffer, _) = take_until::<_, _, NomError<_>>(KW_STARTXREF)(&buffer[offset..])
            .unwrap_or((&buffer[offset..], &[]));
        let (_, start_xref_offset) = delimited(
            terminated(tag(KW_STARTXREF), eol),
            digit1,
            delimited(eol, tag(EOF), opt(alt((char('\r'), char('\n'))))),
        )(buffer)
        .map_err(new_parse_failure!(
            e,
            NewParseFailure {
                buffer,
                code: ParseErrorCode::NotFound(stringify!(StartXRef), Some(e.code)),
            }
        ))?;

        let start_xref_offset: Offset =
            ascii_to_u64(start_xref_offset).ok_or_else(|| NewParseFailure {
                buffer: start_xref_offset,
                code: ParseErrorCode::OffSet(stringify!(StartXRef)),
            })?;

        Ok((&[], Self(start_xref_offset)))
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<Offset> for StartXRef {
        fn from(value: Offset) -> Self {
            Self(value)
        }
    }

    impl Deref for StartXRef {
        type Target = Offset;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub(crate) mod error {

    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum StartXRefCode {
        #[error("TooSmallBuffer")]
        TooSmallBuffer,
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;

    #[test]
    fn start_xref_valid() {
        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/CD74097EBFE5D8A25FE8A229299730FA_xref_stream.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, start_xref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(start_xref_offset, StartXRef(238838));

        // PDF produced by MikTeX pdfTeX-1.40.11
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/907C09F6EB56BEAF5235FAC6F37F5B84_trailer.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, startxref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(startxref_offset, StartXRef(265666));

        // PDF produced by pdfTeX-1.40.21
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/3AB9790B3CB9A73CF4BF095B2CE17671_xref_stream.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, startxref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(startxref_offset, StartXRef(309373));

        // PDF produced by pdfTeX-1.40.22
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/1F0F80D27D156F7EF35B1DF40B1BD3E8_xref_stream.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let (_, startxref_offset) = StartXRef::parse(&buffer[offset..]).unwrap();
        assert_eq!(startxref_offset, StartXRef(365385));
    }

    #[test]
    fn startxref_invalid() {
        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_byte_offset.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = NewParseFailure {
            buffer: b"startxref\r\n%%EOF\r\n",
            code: ParseErrorCode::NotFound(stringify!(StartXRef), Some(ErrorKind::Digit)),
        };
        assert_err_eq!(parse_result, expected_error);

        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_eof.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = NewParseFailure {
            buffer: b"startxref\r\n999999\r\n%%PDF-1.4\r\n",
            code: ParseErrorCode::NotFound(stringify!(StartXRef), Some(ErrorKind::Tag)),
        };
        assert_err_eq!(parse_result, expected_error);

        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_eol.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = NewParseFailure {
            buffer: b"dobj\r\nstartxre\r\nf999999%%EOF\r\n",
            code: ParseErrorCode::NotFound(stringify!(StartXRef), Some(ErrorKind::Tag)),
        };
        assert_err_eq!(parse_result, expected_error);

        // Synthetic test
        let buffer: &[Byte] =
            include_bytes!("../../tests/data/SYNTHETIC_startxref_invalid_missing_startxref.bin");
        let offset = buffer.len() - STARTXREF_MAX_SIZE;
        let parse_result = StartXRef::parse(&buffer[offset..]);
        let expected_error = NewParseFailure {
            buffer: b"tream\r\nendobj\r\n999999\r\n%%EOF\r\n",
            code: ParseErrorCode::NotFound(stringify!(StartXRef), Some(ErrorKind::Tag)),
        };
        assert_err_eq!(parse_result, expected_error);

        // TODO Add tests
    }
}
