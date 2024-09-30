use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;
use ::std::ops::Deref;
use nom::combinator::recognize;

use super::DirectValue;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse_recoverable;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.6 Array objects, p29]
#[derive(Debug, Clone)]
pub struct Array<'buffer> {
    array: Vec<DirectValue<'buffer>>,
    span: Span,
}

impl Display for Array<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[")?;
        for (i, obj) in self.array.iter().enumerate() {
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
        self.array == other.array && self.span == other.span
    }
}

impl<'buffer> ObjectParser<'buffer> for Array<'buffer> {
    fn parse_object(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<(&[Byte], Self)> {
        let size = buffer.len();
        let start = offset;

        let mut values = vec![];
        let mut value: DirectValue;
        let (mut buffer, recognised) = recognize(terminated(
            char('['),
            opt(white_space_or_comment),
        ))(buffer)
        .map_err(parse_recoverable!(
            e,
            ParseRecoverable::new(e.input, stringify!(Array), ParseErrorCode::NotFound(e.code))
        ))?;
        let mut offset = offset + recognised.len();
        // Here, we know that the buffer starts with an array, and the following
        // errors should be propagated as ArrayFailure
        loop {
            // Check for the end of the array (closing square bracket)
            if let Ok((remains, _)) = char::<_, NomError<_>>(']')(buffer) {
                buffer = remains;
                break;
            }
            // Parse the value
            (buffer, value) = DirectValue::parse_object(buffer, offset).map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(Array),
                    ParseErrorCode::RecMissingClosing(Box::new(err.code())),
                )
            })?;
            offset = value.span().end();

            values.push(value);
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remains, recognised)) = recognize(opt(white_space_or_comment))(buffer) {
                buffer = remains;
                offset += recognised.len();
            }
        }

        let span = Span::new(start, size - buffer.len());
        let array = Self {
            array: values,
            span,
        };
        Ok((buffer, array))
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod convert {
    use super::*;

    impl<'buffer> Array<'buffer> {
        pub fn new(values: Vec<DirectValue<'buffer>>, span: Span) -> Self {
            Self {
                array: values,
                span,
            }
        }
    }

    impl<'buffer> Deref for Array<'buffer> {
        type Target = Vec<DirectValue<'buffer>>;

        fn deref(&self) -> &Self::Target {
            &self.array
        }
    }

    impl<'buffer> IntoIterator for Array<'buffer> {
        type Item = DirectValue<'buffer>;
        type IntoIter = <Vec<DirectValue<'buffer>> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.array.into_iter()
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
    use crate::object::direct::numeric::Real;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::direct::string::Literal;
    use crate::parse::Span;
    use crate::parse_span_assert_eq;

    #[test]
    fn array_valid() {
        // A synthetic test
        let buffer = b"[1 1.0 true null(A literal string)/Name]";
        let expected_parsed = Array::new(
            vec![
                Integer::new(1, Span::new(1, 1)).into(),
                Real::new(1.0, Span::new(3, 3)).into(),
                Boolean::new(true, Span::new(7, 4)).into(),
                Null::new(Span::new(12, 4)).into(),
                Literal::from(("A literal string", Span::new(16, 18))).into(),
                Name::from(("Name", Span::new(34, 5))).into(),
            ],
            Span::new(0, 40),
        );
        parse_span_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // A synthetic test
        // Array: Empty
        let buffer = b"[]";
        let expected_parsed = Array::new(vec![], Span::new(0, 2));
        parse_span_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // A synthetic test
        // Array: 2D matrix
        let buffer = b"[[1 2 3][4 5 6][7 8 9]]";
        let expected_parsed = Array::new(
            vec![
                Array::new(
                    vec![
                        Integer::new(1, Span::new(2, 1)).into(),
                        Integer::new(2, Span::new(4, 1)).into(),
                        Integer::new(3, Span::new(6, 1)).into(),
                    ],
                    Span::new(1, 7),
                )
                .into(),
                Array::new(
                    vec![
                        Integer::new(4, Span::new(9, 1)).into(),
                        Integer::new(5, Span::new(11, 1)).into(),
                        Integer::new(6, Span::new(13, 1)).into(),
                    ],
                    Span::new(8, 7),
                )
                .into(),
                Array::new(
                    vec![
                        Integer::new(7, Span::new(16, 1)).into(),
                        Integer::new(8, Span::new(18, 1)).into(),
                        Integer::new(9, Span::new(20, 1)).into(),
                    ],
                    Span::new(15, 7),
                )
                .into(),
            ],
            Span::new(0, 23),
        );
        parse_span_assert_eq!(buffer, expected_parsed, "".as_bytes());

        // PDF produced by pdfTeX-1.40.21
        // Array: No space between elements
        let buffer = b"[<CD74097EBFE5D8A25FE8A229299730FA><CD74097EBFE5D8A25FE8A229299730FA>]";
        let expected_parsed = Array::new(
            vec![
                Hexadecimal::from(("CD74097EBFE5D8A25FE8A229299730FA", Span::new(1, 34))).into(),
                Hexadecimal::from(("CD74097EBFE5D8A25FE8A229299730FA", Span::new(35, 34))).into(),
            ],
            Span::new(0, 70),
        );
        parse_span_assert_eq!(buffer, expected_parsed, "".as_bytes());
    }

    #[test]
    fn array_invalid() {
        // Synthetic tests

        // Array: Not found
        let parse_result = Array::parse_object(b"1 1.0 true null(A literal string)/Name", 0);
        let expected_error = ParseRecoverable::new(
            b"1 1.0 true null(A literal string)/Name",
            stringify!(Array),
            ParseErrorCode::NotFound(ErrorKind::Char),
        );
        assert_err_eq!(parse_result, expected_error);

        // Array: Missing closing square bracket
        let parse_result = Array::parse_object(b"[1 1.0 true null(A literal string)/Name", 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Array),
            ParseErrorCode::RecMissingClosing(Box::new(ParseErrorCode::NotFoundUnion)),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
