pub(crate) mod section;
pub(crate) mod stream;
pub(crate) mod trailer;

use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::section::Section;
use self::stream::XRefStream;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p55]
#[derive(Debug, PartialEq)]
pub(crate) enum Increment<'buffer> {
    Section(Section<'buffer>),
    Stream(XRefStream<'buffer>),
}

impl Display for Increment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Increment::Section(section) => write!(f, "{}", section),
            Increment::Stream(stream) => write!(f, "{}", stream),
        }
    }
}

impl<'buffer> ObjectParser<'buffer> for Increment<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: crate::Offset) -> ParseResult<'buffer, Self> {
        Section::parse_suppress_recoverable::<Self>(buffer, offset)
            .or_else(|| XRefStream::parse_suppress_recoverable::<Self>(buffer, offset))
            .unwrap_or_else(|| {
                // Except for Subsection, Section and XRefStream, NotFound
                // errors for xref objects should be propagated as failures.
                Err(
                    ParseFailure::new(buffer, stringify!(Increment), ParseErrorCode::NotFoundUnion)
                        .into(),
                )
            })
    }

    fn span(&self) -> Span {
        match self {
            Self::Section(section) => section.span(),
            Self::Stream(stream) => stream.span(),
        }
    }
}

mod table {
    use super::*;
    use crate::xref::error::XRefResult;
    use crate::xref::Table;
    use crate::xref::ToTable;

    impl ToTable for Increment<'_> {
        fn to_table(&self) -> XRefResult<Table> {
            match self {
                Self::Section(section) => section.to_table(),
                Self::Stream(stream) => stream.to_table(),
            }
        }
    }
}

mod convert {

    use super::trailer::KEY_PREV;
    use super::*;
    use crate::impl_from_ref;
    use crate::object::error::ObjectResult;
    use crate::Offset;

    impl_from_ref!('buffer, Section<'buffer>, Section, Increment<'buffer>);
    impl_from_ref!('buffer, XRefStream<'buffer>, Stream, Increment<'buffer>);

    impl<'buffer> Increment<'buffer> {
        pub(crate) fn prev(&self) -> ObjectResult<Option<Offset>> {
            let dictionary = match self {
                Self::Section(section) => &section.trailer,
                Self::Stream(stream) => &stream.stream.dictionary,
            };
            dictionary.opt_usize(KEY_PREV)
        }

        // Avoid calling several times. Store in the processed table.
        // pub(crate) fn trailer(self) -> XRefResult<Trailer<'buffer>> {
        //     let dictionary = match self {
        //         Self::Section(section) => section.trailer,
        //         Self::Stream(stream) => stream.trailer,
        //     };
        //     Trailer::try_from(dictionary)
        // }
    }
}

// TODO Add tests
