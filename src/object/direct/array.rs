use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::ops::Deref;

use super::DirectValue;
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
pub struct Array<'buffer>(Vec<DirectValue<'buffer>>);

impl Display for Array<'_> {
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

impl PartialEq for Array<'_> {
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

impl<'buffer> Parser<'buffer> for Array<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        let mut array = vec![];
        let mut value: DirectValue;
        let (mut buffer, _) = terminated(char('['), opt(white_space_or_comment))(buffer).map_err(
            parse_recoverable!(
                e,
                ParseRecoverable::new(e.input, stringify!(Array), ParseErrorCode::NotFound(e.code))
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
            (buffer, value) = DirectValue::parse(buffer).map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(Array),
                    ParseErrorCode::RecMissingClosing(Box::new(err.code())),
                )
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

    impl<'buffer> From<Vec<DirectValue<'buffer>>> for Array<'buffer> {
        fn from(value: Vec<DirectValue<'buffer>>) -> Self {
            Self(value)
        }
    }

    impl<'buffer> FromIterator<DirectValue<'buffer>> for Array<'buffer> {
        fn from_iter<T: IntoIterator<Item = DirectValue<'buffer>>>(iter: T) -> Array<'buffer> {
            Self(Vec::from_iter(iter))
        }
    }

    impl<'buffer> Deref for Array<'buffer> {
        type Target = Vec<DirectValue<'buffer>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'buffer> IntoIterator for Array<'buffer> {
        type Item = DirectValue<'buffer>;
        type IntoIter = <Vec<DirectValue<'buffer>> as IntoIterator>::IntoIter;

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
    use crate::object::direct::boolean::Boolean;
    use crate::object::direct::name::Name;
    use crate::object::direct::null::Null;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::parse::Span;
    use crate::parse_assert_eq;

    #[test]
    fn array_valid() {
        // A synthetic test
        let buffer = b"[1 1.0 true null(A literal string)/Name]";
        let expected_parsed = Array::from_iter([
            Integer::new(1, Span::new(1, 1)).into(),
            1.0.into(),
            Boolean::new(true, Span::new(6, 4)).into(),
            Null::new(Span::new(12, 4)).into(),
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
        let buffer = b"[[1 2 3][4 5 6][7 8 9]]";
        let expected_parsed = Array::from_iter([
            Array::from_iter([
                Integer::new(1, Span::new(2, 1)).into(),
                Integer::new(2, Span::new(4, 1)).into(),
                Integer::new(3, Span::new(6, 1)).into(),
            ])
            .into(),
            Array::from_iter([
                Integer::new(4, Span::new(9, 1)).into(),
                Integer::new(5, Span::new(11, 1)).into(),
                Integer::new(6, Span::new(13, 1)).into(),
            ])
            .into(),
            Array::from_iter([
                Integer::new(7, Span::new(16, 1)).into(),
                Integer::new(8, Span::new(18, 1)).into(),
                Integer::new(9, Span::new(20, 1)).into(),
            ])
            .into(),
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
        let expected_error = ParseRecoverable::new(
            b"1 1.0 true null(A literal string)/Name",
            stringify!(Array),
            ParseErrorCode::NotFound(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Array: Missing closing square bracket
        let parse_result = Array::parse(b"[1 1.0 true null(A literal string)/Name");
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Array),
            ParseErrorCode::RecMissingClosing(Box::new(ParseErrorCode::NotFoundUnion)),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
