use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::combinator::map;
use ::nom::error::Error as NomError;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::BooleanRecoverable;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::KW_FALSE;
use crate::parse::KW_TRUE;
use crate::parse_error;
use crate::Byte;

/// REFERENCE:  [7.3.2 Boolean objects, p24]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Boolean(bool);

impl Display for Boolean {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Parser for Boolean {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, value) = alt((
            map(tag::<_, _, NomError<_>>(KW_TRUE), |_| Self(true)),
            map(tag(KW_FALSE), |_| Self(false)),
        ))(buffer)
        .map_err(parse_error!(
            e,
            BooleanRecoverable::NotFound {
                code: e.code,
                input: debug_bytes(e.input),
            }
        ))?;

        Ok((buffer, value))
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum BooleanRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<bool> for Boolean {
        fn from(value: bool) -> Self {
            Self(value)
        }
    }

    impl Deref for Boolean {
        type Target = bool;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse_assert_eq;

    #[test]
    fn boolean_valid() {
        parse_assert_eq!(b"true", Boolean(true), "".as_bytes());
        parse_assert_eq!(b"true<", Boolean(true), "<".as_bytes());
        parse_assert_eq!(b"true    <<", Boolean(true), "    <<".as_bytes());

        parse_assert_eq!(b"false<", Boolean(false), "<".as_bytes());
        parse_assert_eq!(b"false .", Boolean(false), " .".as_bytes());

        parse_assert_eq!(b"true false", Boolean(true), " false".as_bytes());
        parse_assert_eq!(b"false true", Boolean(false), " true".as_bytes());
        parse_assert_eq!(b"truefalse", Boolean(true), "false".as_bytes());
        parse_assert_eq!(b"falsetrue", Boolean(false), "true".as_bytes());
    }

    #[test]
    fn boolean_invalid() {
        // Boolean: Not found
        let parse_result = Boolean::parse(b"tr");
        let expected_error = ParseErr::Error(
            BooleanRecoverable::NotFound {
                code: ErrorKind::Tag,
                input: "tr".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
