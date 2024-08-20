use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::ops::Deref;

use self::error::ArrayFailure;
use self::error::ArrayRecoverable;
use crate::fmt::debug_bytes;
use crate::object::direct::DirectValue;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_error;
use crate::Byte;

/// REFERENCE: [7.3.6 Array objects, p29]
#[derive(Debug, Default, Clone)]
pub struct Array(Vec<DirectValue>);

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[")?;
        for (i, obj) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", obj)?;
        }
        write!(f, "]")
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if a != b {
                return false;
            }
        }
        true
    }
}

impl Parser for Array {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let mut array = vec![];
        let mut value: DirectValue;
        let (mut buffer, _) =
            terminated(char('['), opt(white_space_or_comment))(buffer).map_err(parse_error!(
                e,
                ArrayRecoverable::NotFound {
                    code: e.code,
                    input: debug_bytes(buffer)
                }
            ))?;
        // Here, we know that the buffer starts with an array, and the following
        // errors should be propagated as ArrayFailure
        loop {
            // Check for the end of the array (closing square bracket)
            if let Ok((remaining, _)) = char::<_, NomError<_>>(']')(buffer) {
                buffer = remaining;
                break;
            }
            // Parse the value
            (buffer, value) =
                DirectValue::parse_semi_quiet::<DirectValue>(buffer).unwrap_or_else(|| {
                    Err(ParseErr::Failure(
                        ArrayFailure::MissingClosing(debug_bytes(buffer)).into(),
                    ))
                })?;

            array.push(value);
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remaining, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remaining;
            }
        }

        let array = Self(array);
        Ok((buffer, array))
    }
}

mod convert {
    use super::*;

    impl From<Vec<DirectValue>> for Array {
        fn from(value: Vec<DirectValue>) -> Self {
            Self(value)
        }
    }

    impl FromIterator<DirectValue> for Array {
        fn from_iter<T: IntoIterator<Item = DirectValue>>(iter: T) -> Array {
            Self(Vec::from_iter(iter))
        }
    }

    impl Deref for Array {
        type Target = Vec<DirectValue>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl IntoIterator for Array {
        type Item = DirectValue;
        type IntoIter = <Vec<DirectValue> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}

pub(crate) mod error {

    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum ArrayRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum ArrayFailure {
        #[error("Missing Closing: Expected a name or closing angle brackets. Input: {0}")]
        MissingClosing(String),
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::name::Name;
    use crate::object::direct::null::Null;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::parse::error::ParseFailure;
    use crate::parse_assert_eq;

    #[test]
    fn array_valid() {
        // A synthetic test
        let buffer = b"[1 1.0 true null(A literal string)/Name]";
        let expected_parsed = Array::from_iter([
            1.into(),
            1.0.into(),
            true.into(),
            Null.into(),
            Literal::from("A literal string").into(),
            Name::from("Name").into(),
        ]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // A synthetic test
        // Array: Empty
        let buffer = b"[]";
        let expected_parsed = Array::from_iter([]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // A synthetic test
        // Array: 2D matrix
        let buffer = b"[[1 2 3] [4 5 6] [7 8 9]]";
        let expected_parsed = Array::from_iter([
            Array::from_iter([1.into(), 2.into(), 3.into()]).into(),
            Array::from_iter([4.into(), 5.into(), 6.into()]).into(),
            Array::from_iter([7.into(), 8.into(), 9.into()]).into(),
        ]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        // Array: No space between elements
        let buffer = b"[<CD74097EBFE5D8A25FE8A229299730FA><CD74097EBFE5D8A25FE8A229299730FA>]";
        let expected_parsed = Array::from_iter([
            Hexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
            Hexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
        ]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());
    }

    #[test]
    fn array_invalid() {
        // Synthetic tests

        // Array: Not found
        let parse_result = Array::parse(b"1 1.0 true null(A literal string)/Name");
        let expected_error = ParseErr::Error(
            ArrayRecoverable::NotFound {
                code: ErrorKind::Char,
                input: "1 1.0 true null(A literal string)/Name".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Array: Missing closing square bracket
        let parse_result = Array::parse(b"[1 1.0 true null(A literal string)/Name");
        let expected_error = ParseErr::Failure(ParseFailure::Array(ArrayFailure::MissingClosing(
            "".to_string(),
        )));
        assert_err_eq!(parse_result, expected_error);
    }
}
