pub(crate) mod character_set;
pub(crate) mod error;
pub(crate) mod num;

use self::error::ParseErr;
use self::error::ParseResult;
use crate::Byte;
use crate::Offset;

pub(crate) const EOF: &str = "%%EOF";
pub(crate) const KW_ENDOBJ: &str = "endobj";
pub(crate) const KW_ENDSTREAM: &str = "endstream";
pub(crate) const KW_FALSE: &str = "false";
pub(crate) const KW_ID: &str = "ID";
pub(crate) const KW_INFO: &str = "info";
pub(crate) const KW_NULL: &str = "null";
pub(crate) const KW_OBJ: &str = "obj";
pub(crate) const KW_ROOT: &str = "root";
pub(crate) const KW_SIZE: &str = "size";
pub(crate) const KW_START: &str = "start";
pub(crate) const KW_R: &str = "R";
pub(crate) const KW_STARTXREF: &str = "startxref";
pub(crate) const KW_STREAM: &str = "stream";
pub(crate) const KW_TRAILER: &str = "trailer";
pub(crate) const KW_TRUE: &str = "true";
pub(crate) const KW_XREF: &str = "xref";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Span {
    start: usize,
    end: usize,
}
pub(crate) trait Parser<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<'buffer, Self>
    where
        Self: Sized;

    fn spans(&self) -> Vec<Span>;
}

pub(crate) trait ObjectParser<'buffer> {
    fn parse_object(
        buffer: &'buffer [Byte],
        offset: Offset,
    ) -> ParseResult<'buffer, (&[Byte], Self)>
    where
        Self: Sized;

    fn span(&self) -> Span;

    fn parse_suppress_recoverable_span<O>(
        buffer: &'buffer [Byte],
        offset: Offset,
    ) -> Option<ParseResult<(&[Byte], O)>>
    where
        Self: Sized,
        O: From<Self>,
    {
        let result = Self::parse_object(buffer, offset);
        match result {
            Ok((buffer, object)) => Some(Ok((buffer, object.into()))),
            Err(ParseErr::Failure(err)) => Some(Err(ParseErr::Failure(err))),
            _ => None,
        }
    }
}

mod convert {
    use super::*;

    impl Span {
        pub fn new(start: usize, len: usize) -> Self {
            Self {
                start,
                end: start + len,
            }
        }

        pub fn start(&self) -> usize {
            self.start
        }

        pub fn end(&self) -> usize {
            self.end
        }
    }
}

mod tests {
    #[macro_export]
    macro_rules! parse_span_assert_eq {
        ($buffer:expr, $expected_parsed:expr, $expected_remains:expr) => {
            assert_eq!(
                ObjectParser::parse_object($buffer, 0).unwrap(),
                ($expected_remains, $expected_parsed)
            );
        };
        // The two patterns differ only in the trailing comma
        ($buffer:expr, $expected_parsed:expr, $expected_remains:expr,) => {
            assert_eq!(
                ObjectParser::parse_object($buffer, 0).unwrap(),
                ($expected_remains, $expected_parsed)
            );
        };
    }
}
