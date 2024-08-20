pub(crate) mod section;
pub(crate) mod stream;
pub(crate) mod trailer;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::IncrementRecoverable;
use self::section::Section;
use self::stream::XRefStream;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p55]
#[derive(Debug, PartialEq)]
pub(crate) enum Increment {
    Section(Section),
    Stream(XRefStream),
}

impl Display for Increment {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Increment::Section(section) => write!(f, "{}", section),
            Increment::Stream(stream) => write!(f, "{}", stream),
        }
    }
}

impl Parser for Increment {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Section::parse_semi_quiet::<Self>(buffer)
            .or_else(|| XRefStream::parse_semi_quiet::<Self>(buffer))
            .unwrap_or_else(|| {
                Err(ParseErr::Error(
                    IncrementRecoverable::NotFound(debug_bytes(buffer)).into(),
                ))
            })
    }
}

mod process {
    use super::*;
    use crate::process::error::ProcessResult;
    use crate::xref::Table;
    use crate::xref::ToTable;

    impl ToTable for Increment {
        fn to_table(&self) -> ProcessResult<Table> {
            match self {
                Self::Section(section) => section.to_table(),
                Self::Stream(stream) => stream.to_table(),
            }
        }
    }
}

mod convert {

    use super::trailer::Trailer;
    use super::*;

    impl From<Section> for Increment {
        fn from(value: Section) -> Self {
            Increment::Section(value)
        }
    }

    impl From<XRefStream> for Increment {
        fn from(value: XRefStream) -> Self {
            Increment::Stream(value)
        }
    }

    impl Increment {
        pub(crate) fn trailer(&self) -> &Trailer {
            match self {
                Self::Section(section) => &section.trailer,
                Self::Stream(stream) => &stream.trailer,
            }
        }
    }
}

pub(crate) mod error {

    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IncrementRecoverable {
        #[error("Not found: {0}")]
        NotFound(String),
    }
}

// TODO Add tests
