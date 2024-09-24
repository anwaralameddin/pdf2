pub(crate) mod character_set;
pub(crate) mod error;
pub(crate) mod num;

use self::error::ParseErr;
use self::error::ParseResult;
use crate::Byte;

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

pub(crate) trait Parser<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<'buffer, (&[Byte], Self)>
    where
        Self: Sized;

    /// Try to parse the buffer and return an option:
    /// - Some(Ok(_)): if the buffer was parsed successfully
    /// - Some(Err(ParseErr::Failure(_))): if the parser failed with no possible
    /// recovery
    /// - None: if the parser returned another error, which could be recovered
    /// by another parser
    fn parse_suppress_recoverable<O>(buffer: &'buffer [Byte]) -> Option<ParseResult<(&[Byte], O)>>
    where
        Self: Sized,
        O: From<Self>,
    {
        let result = Self::parse(buffer);
        match result {
            Ok((buffer, object)) => Some(Ok((buffer, object.into()))),
            Err(ParseErr::Failure(err)) => Some(Err(ParseErr::Failure(err))),
            _ => None,
        }
    }
}

mod tests {
    #[macro_export]
    macro_rules! parse_assert_eq {
        ($buffer:expr, $expected_parsed:expr, $expected_remains:expr) => {
            assert_eq!(
                Parser::parse($buffer).unwrap(),
                ($expected_remains, $expected_parsed)
            );
        };
        // The two patterns differ only in the trailing comma
        ($buffer:expr, $expected_parsed:expr, $expected_remains:expr,) => {
            assert_eq!(
                Parser::parse($buffer).unwrap(),
                ($expected_remains, $expected_parsed)
            );
        };
    }

    #[macro_export]
    macro_rules! escape_assert_err {
        ($object:expr, $expected_error:expr) => {
            assert_eq!($object.escape(), Err($expected_error));
        };
    }
}
