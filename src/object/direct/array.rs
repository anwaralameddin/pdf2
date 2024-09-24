use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::ops::Deref;

use crate::object::direct::OwnedDirectValue;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_recoverable;
use crate::Byte;

/// REFERENCE: [7.3.6 Array objects, p29]
#[derive(Debug, Default, Clone)]
pub struct OwnedArray(Vec<OwnedDirectValue>);

impl Display for OwnedArray {
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

impl PartialEq for OwnedArray {
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

impl Parser<'_> for OwnedArray {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let mut array = vec![];
        let mut value: OwnedDirectValue;
        let (mut buffer, _) = terminated(char('['), opt(white_space_or_comment))(buffer).map_err(
            parse_recoverable!(
                e,
                ParseRecoverable {
                    buffer: e.input,
                    object: stringify!(Array),
                    code: ParseErrorCode::NotFound(e.code)
                }
            ),
        )?;
        // Here, we know that the buffer starts with an array, and the following
        // errors should be propagated as ArrayFailure
        loop {
            // Check for the end of the array (closing square bracket)
            if let Ok((remains, _)) = char::<_, NomError<_>>(']')(buffer) {
                buffer = remains;
                break;
            }
            // Parse the value
            (buffer, value) = OwnedDirectValue::parse(buffer).map_err(|err| ParseFailure {
                buffer: err.buffer(),
                object: stringify!(Array),
                code: ParseErrorCode::RecMissingClosing(Box::new(err.code())),
            })?;

            array.push(value);
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remains, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remains;
            }
        }

        let array = Self(array);
        Ok((buffer, array))
    }
}

mod convert {
    use super::*;

    impl From<Vec<OwnedDirectValue>> for OwnedArray {
        fn from(value: Vec<OwnedDirectValue>) -> Self {
            Self(value)
        }
    }

    impl FromIterator<OwnedDirectValue> for OwnedArray {
        fn from_iter<T: IntoIterator<Item = OwnedDirectValue>>(iter: T) -> OwnedArray {
            Self(Vec::from_iter(iter))
        }
    }

    impl Deref for OwnedArray {
        type Target = Vec<OwnedDirectValue>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl IntoIterator for OwnedArray {
        type Item = OwnedDirectValue;
        type IntoIter = <Vec<OwnedDirectValue> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::name::OwnedName;
    use crate::object::direct::null::Null;
    use crate::object::direct::string::OwnedHexadecimal;
    use crate::object::direct::string::OwnedLiteral;
    use crate::parse_assert_eq;

    #[test]
    fn array_valid() {
        // A synthetic test
        let buffer = b"[1 1.0 true null(A literal string)/Name]";
        let expected_parsed = OwnedArray::from_iter([
            1.into(),
            1.0.into(),
            true.into(),
            Null.into(),
            OwnedLiteral::from("A literal string").into(),
            OwnedName::from("Name").into(),
        ]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // A synthetic test
        // Array: Empty
        let buffer = b"[]";
        let expected_parsed = OwnedArray::from_iter([]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // A synthetic test
        // Array: 2D matrix
        let buffer = b"[[1 2 3] [4 5 6] [7 8 9]]";
        let expected_parsed = OwnedArray::from_iter([
            OwnedArray::from_iter([1.into(), 2.into(), 3.into()]).into(),
            OwnedArray::from_iter([4.into(), 5.into(), 6.into()]).into(),
            OwnedArray::from_iter([7.into(), 8.into(), 9.into()]).into(),
        ]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        // Array: No space between elements
        let buffer = b"[<CD74097EBFE5D8A25FE8A229299730FA><CD74097EBFE5D8A25FE8A229299730FA>]";
        let expected_parsed = OwnedArray::from_iter([
            OwnedHexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
            OwnedHexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
        ]);
        parse_assert_eq!(buffer, expected_parsed, "".as_bytes());
    }

    #[test]
    fn array_invalid() {
        // Synthetic tests

        // Array: Not found
        let parse_result = OwnedArray::parse(b"1 1.0 true null(A literal string)/Name");
        let expected_error = ParseRecoverable {
            buffer: b"1 1.0 true null(A literal string)/Name",
            object: stringify!(Array),
            code: ParseErrorCode::NotFound(ErrorKind::Char),
        };
        assert_err_eq!(parse_result, expected_error);

        // Array: Missing closing square bracket
        let parse_result = OwnedArray::parse(b"[1 1.0 true null(A literal string)/Name");
        let expected_error = ParseFailure {
            buffer: b"",
            object: stringify!(Array),
            code: ParseErrorCode::RecMissingClosing(Box::new(ParseErrorCode::NotFoundUnion)),
        };
        assert_err_eq!(parse_result, expected_error);
    }
}
