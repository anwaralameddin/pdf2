pub(crate) mod section;
pub(crate) mod stream;
pub(crate) mod trailer;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::section::Section;
use self::stream::XRefStream;
use crate::parse::error::NewParseErr;
use crate::parse::error::NewParseRecoverable;
use crate::parse::error::NewParseResult;
use crate::parse::error::ParseErrorCode;
use crate::parse::NewParser;
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

impl NewParser<'_> for Increment {
    fn parse(buffer: &[Byte]) -> NewParseResult<(&[Byte], Self)> {
        Section::parse_semi_quiet::<Self>(buffer)
            .or_else(|| XRefStream::parse_semi_quiet::<Self>(buffer))
            .unwrap_or_else(|| {
                Err(NewParseRecoverable {
                    buffer,
                    code: ParseErrorCode::NotFound(stringify!(Increment), None),
                }
                .into())
            })
    }
}

mod process {
    use super::*;
    use crate::process::error::NewProcessResult;
    use crate::xref::Table;
    use crate::xref::ToTable;

    impl ToTable for Increment {
        fn to_table(&self) -> NewProcessResult<Table> {
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
    use ::std::num::TryFromIntError;
    use ::thiserror::Error;

    use crate::process::error::NewProcessErr;

    // NewProcessErr do not implement Copy
    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum IncrementCode {
        #[error("{0}. Trailer dictionary. Error: {1}")]
        TrailerDictionary(&'static str, NewProcessErr),
    }

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum IncrementError {
        #[error("Generation number. Error: {1}. Input: {0}")]
        EntryGenerationNumber(u64, TryFromIntError),
        #[error("Duplicate object number: {0}")]
        DuplicateObjectNumber(u64),
    }
}

// TODO Add tests
