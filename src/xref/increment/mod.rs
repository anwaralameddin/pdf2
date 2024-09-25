pub(crate) mod section;
pub(crate) mod stream;
pub(crate) mod trailer;

use self::section::Section;
use self::stream::XRefStream;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p55]
#[derive(Debug, PartialEq)]
pub(crate) enum Increment<'buffer> {
    Section(Section<'buffer>),
    Stream(XRefStream<'buffer>),
}

// FIXME Implement Display for Increment
// impl Display for Increment<'_> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
//         match self {
//             Increment::Section(section) => write!(f, "{}", section),
//             Increment::Stream(stream) => write!(f, "{}", stream),
//         }
//     }
// }

impl<'buffer> Parser<'buffer> for Increment<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        Section::parse_suppress_recoverable::<Self>(buffer)
            .or_else(|| XRefStream::parse_suppress_recoverable::<Self>(buffer))
            .unwrap_or_else(|| {
                // Except for Subsection, Section and XRefStream, NotFound
                // errors for xref objects should be propagated as failures.
                Err(ParseFailure {
                    buffer,
                    object: stringify!(Increment),
                    code: ParseErrorCode::NotFoundUnion,
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

    impl ToTable for Increment<'_> {
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
    use crate::impl_from_ref;
    use crate::Offset;

    impl_from_ref!('buffer, Section<'buffer>, Section, Increment<'buffer>);
    impl_from_ref!('buffer, XRefStream<'buffer>, Stream, Increment<'buffer>);

    impl<'buffer> Increment<'buffer> {
        pub(crate) fn prev(&self) -> Option<Offset> {
            match self {
                Self::Section(section) => section.trailer.prev(),
                Self::Stream(stream) => stream.trailer.prev(),
            }
        }

        pub(crate) fn trailer(self) -> Trailer<'buffer> {
            match self {
                Self::Section(section) => section.trailer,
                Self::Stream(stream) => stream.trailer,
            }
        }
    }
}

pub(crate) mod error {
    use ::std::num::TryFromIntError;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum IncrementError {
        #[error("Generation number. Error: {1}. Input: {0}")]
        EntryGenerationNumber(u64, TryFromIntError),
        #[error("Duplicate object number: {0}")]
        DuplicateObjectNumber(u64),
    }
}

// TODO Add tests
