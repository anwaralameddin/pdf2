use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use crate::parse::character_set::comment;
use crate::parse::character_set::eol;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::Span;
use crate::parse::MARKER_PDF;
use crate::parse_recoverable;
use crate::Byte;

const HEADER_MIN_SIZE: usize = 8;

/// REFERENCE: [7.5.2 File header, p54-55]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Header<'buffer> {
    version: Version,
    data: &'buffer [Byte],
    span: Span,
}

impl Display for Header<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "{}", self.version)?;
        for &byte in self.data {
            write!(f, "{}", char::from(byte))?;
        }
        writeln!(f)
    }
}

impl<'buffer> Parser<'buffer> for Header<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<'buffer, Self> {
        if buffer.len() < HEADER_MIN_SIZE {
            return Err(ParseFailure::new(
                buffer,
                stringify!(Header),
                ParseErrorCode::TooSmallBuffer,
            )
            .into());
        }
        let version = Version::parse(buffer)?;
        let mut offset = 8;
        let remains = &buffer[offset..];

        let (remains, recognised) = eol(remains).map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Header),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        offset += recognised.len();

        let (remains, data) = comment(remains).unwrap_or((remains, &[]));
        offset += data.len() + 1;
        // TODO Verify that data.len() >= 4 if not empty

        let (_, recognised) = eol(remains).map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Header),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        offset += recognised.len();

        let span = Span::new(0, offset);
        Ok(Header {
            version,
            data,
            span,
        })
    }

    fn spans(&self) -> Vec<Span> {
        vec![self.span]
    }
}

/// REFERENCE: [7.5.2 File header, p54-55]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Version {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
    V1_4,
    V1_5,
    V1_6,
    V1_7,
    V2_0,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let version = match self {
            Version::V1_0 => "1.0",
            Version::V1_1 => "1.1",
            Version::V1_2 => "1.2",
            Version::V1_3 => "1.3",
            Version::V1_4 => "1.4",
            Version::V1_5 => "1.5",
            Version::V1_6 => "1.6",
            Version::V1_7 => "1.7",
            Version::V2_0 => "2.0",
        };

        write!(f, "{}{}", MARKER_PDF, version)
    }
}

impl Parser<'_> for Version {
    fn parse(buffer: &[Byte]) -> ParseResult<Self> {
        let (_, version) = comment(buffer).map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(
                e.input,
                stringify!(Version),
                ParseErrorCode::NotFound(e.code)
            )
        ))?;
        let version = match version {
            b"PDF-1.0" => Version::V1_0,
            b"PDF-1.1" => Version::V1_1,
            b"PDF-1.2" => Version::V1_2,
            b"PDF-1.3" => Version::V1_3,
            b"PDF-1.4" => Version::V1_4,
            b"PDF-1.5" => Version::V1_5,
            b"PDF-1.6" => Version::V1_6,
            b"PDF-1.7" => Version::V1_7,
            b"PDF-2.0" => Version::V2_0,
            _ => {
                return Err(ParseFailure::new(
                    version,
                    stringify!(Version),
                    ParseErrorCode::UnsupportedVersion(version),
                )
                .into())
            }
        };

        Ok(version)
    }

    fn spans(&self) -> Vec<Span> {
        vec![Span::new(0, 8)]
    }
}

mod convert {
    use super::*;

    impl<'buffer> Header<'buffer> {
        pub(crate) fn version(&self) -> Version {
            self.version
        }

        pub(crate) fn data(&self) -> &[Byte] {
            self.data
        }
    }
}
