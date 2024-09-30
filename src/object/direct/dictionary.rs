use ::nom::bytes::complete::tag;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::collections::HashMap;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::name::Name;
use super::DirectValue;
use crate::object::indirect::reference::Reference;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_recoverable;
use crate::Byte;

/// REFERENCE: [7.3.7 Dictionary objects, p30-31]
#[derive(Debug, Default, Clone)]
pub struct Dictionary<'buffer>(HashMap<Name<'buffer>, DirectValue<'buffer>>);

impl Display for Dictionary<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<<")?;
        for (i, (key, value)) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{} {}", key, value)?;
        }
        write!(f, ">>")
    }
}

impl PartialEq for Dictionary<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.escape() == other.escape()
    }
}

impl<'buffer> Parser<'buffer> for Dictionary<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        let mut dictionary = HashMap::default();
        let mut key: Name;
        let mut value: DirectValue;
        let (mut buffer, _) = terminated(tag(b"<<"), opt(white_space_or_comment))(buffer).map_err(
            parse_recoverable!(
                e,
                ParseRecoverable::new(
                    e.input,
                    stringify!(Dictionary),
                    ParseErrorCode::NotFound(e.code),
                )
            ),
        )?;
        // Here, we know that the buffer starts with a dictionary, and the
        // following errors should be propagated as DictionaryFailure
        loop {
            // Check for the end of the dictionary (closing angle brackets)
            if let Ok((remains, _)) = tag::<_, _, NomError<_>>(b">>")(buffer) {
                buffer = remains;
                break;
            }
            // Parse the key
            (buffer, key) = Name::parse(buffer).map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(Dictionary),
                    ParseErrorCode::RecMissingClosing(Box::new(err.code())),
                )
            })?;
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remains, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remains;
            }
            // Parse the value
            (buffer, value) = DirectValue::parse(buffer).map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(Dictionary),
                    ParseErrorCode::RecMissingValue(key.to_vec(), Box::new(err.code())),
                )
            })?;

            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remains, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remains;
            }
            // Record the key-value pair
            if let Some(old_value) = dictionary.insert(key, value) {
                // Dictionary keys should not be duplicated.
                // REFERENCE: [7.3.7 Dictionary objects, p30]
                //
                // TODO
                // - Print only if verbose mode is enabled.
                // - Replace with a `log::error!` call
                eprintln!(
                    "Dictionary: Overwriting value for key {}: {} -> {:?}",
                    key,
                    old_value,
                    dictionary.get(&key)
                );
            };
        }

        let dictionary = Self(dictionary);
        Ok((buffer, dictionary))
    }
}

mod escape {
    use super::*;

    impl Dictionary<'_> {
        /// REFERENCE: [7.3.7 Dictionary objects, p30] and [7.3.9 Null object, p33]
        pub(crate) fn escape(&self) -> HashMap<&Name, &DirectValue> {
            // FIXME Take into account values that are references to missing
            // objects, which is the same as having the value null. Also,
            // consider the effect of two references pointing to the same
            // object.
            self.0
                .iter()
                .filter(|(_, value)| !matches!(value, DirectValue::Null(_)))
                .collect()
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;
    use crate::object::direct::array::Array;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Numeric;
    use crate::object::error::ObjectErr;
    use crate::object::error::ObjectErrorCode;
    use crate::object::error::ObjectResult;

    impl<'buffer> From<HashMap<Name<'buffer>, DirectValue<'buffer>>> for Dictionary<'buffer> {
        fn from(value: HashMap<Name<'buffer>, DirectValue<'buffer>>) -> Self {
            Self(value)
        }
    }

    impl<'buffer> FromIterator<(Name<'buffer>, DirectValue<'buffer>)> for Dictionary<'buffer> {
        fn from_iter<T: IntoIterator<Item = (Name<'buffer>, DirectValue<'buffer>)>>(
            iter: T,
        ) -> Dictionary<'buffer> {
            Self(HashMap::from_iter(iter))
        }
    }

    impl<'buffer> Deref for Dictionary<'buffer> {
        type Target = HashMap<Name<'buffer>, DirectValue<'buffer>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'buffer> IntoIterator for Dictionary<'buffer> {
        type Item = (Name<'buffer>, DirectValue<'buffer>);
        type IntoIter = <HashMap<Name<'buffer>, DirectValue<'buffer>> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<'buffer> Dictionary<'buffer> {
        pub(crate) fn opt_get(&'buffer self, key: &'static str) -> Option<&DirectValue> {
            self.0.get(&Name::from(key))
        }

        pub(crate) fn required_get(&'buffer self, key: &'static str) -> ObjectResult<&DirectValue> {
            self.opt_get(key)
                .ok_or_else(|| ObjectErr::new(key, self, ObjectErrorCode::MissingRequiredEntry))
        }

        pub(crate) fn opt_array(&self, key: &'static str) -> ObjectResult<Option<&Array>> {
            self.opt_get(key)
                .map(|value| {
                    value.as_array().ok_or_else(|| {
                        ObjectErr::new(
                            key,
                            self,
                            ObjectErrorCode::Type {
                                value,
                                expected_type: stringify!(Array),
                            },
                        )
                    })
                })
                .transpose()
        }

        pub(crate) fn required_array(&self, key: &'static str) -> ObjectResult<&Array> {
            self.opt_array(key).and_then(|value| {
                value
                    .ok_or_else(|| ObjectErr::new(key, self, ObjectErrorCode::MissingRequiredEntry))
            })
        }

        pub(crate) fn opt_name(&self, key: &'static str) -> ObjectResult<Option<&Name>> {
            self.opt_get(key)
                .map(|value| {
                    value.as_name().ok_or_else(|| {
                        ObjectErr::new(
                            key,
                            self,
                            ObjectErrorCode::Type {
                                value,
                                expected_type: stringify!(Name),
                            },
                        )
                    })
                })
                .transpose()
        }

        pub(crate) fn required_name(&self, key: &'static str) -> ObjectResult<&Name> {
            self.opt_name(key).and_then(|value| {
                value
                    .ok_or_else(|| ObjectErr::new(key, self, ObjectErrorCode::MissingRequiredEntry))
            })
        }

        pub(crate) fn opt_u64(&self, key: &'static str) -> ObjectResult<Option<u64>> {
            self.opt_get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_u64)
                        .ok_or_else(|| {
                            ObjectErr::new(
                                key,
                                self,
                                ObjectErrorCode::Type {
                                    value,
                                    expected_type: stringify!(u64),
                                },
                            )
                        })
                })
                .transpose()
        }

        pub(crate) fn required_u64(&self, key: &'static str) -> ObjectResult<u64> {
            self.opt_u64(key).and_then(|value| {
                value
                    .ok_or_else(|| ObjectErr::new(key, self, ObjectErrorCode::MissingRequiredEntry))
            })
        }

        pub(crate) fn opt_usize(&self, key: &'static str) -> ObjectResult<Option<usize>> {
            self.opt_get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_usize)
                        .ok_or_else(|| {
                            ObjectErr::new(
                                key,
                                self,
                                ObjectErrorCode::Type {
                                    value,
                                    expected_type: stringify!(usize),
                                },
                            )
                        })
                })
                .transpose()
        }

        pub(crate) fn required_usize(&self, key: &'static str) -> ObjectResult<usize> {
            self.opt_usize(key).and_then(|value| {
                value
                    .ok_or_else(|| ObjectErr::new(key, self, ObjectErrorCode::MissingRequiredEntry))
            })
        }

        pub(crate) fn opt_reference(&self, key: &'static str) -> ObjectResult<Option<&Reference>> {
            self.opt_get(key)
                .map(|value| {
                    value.as_reference().ok_or_else(|| {
                        ObjectErr::new(
                            key,
                            self,
                            ObjectErrorCode::Type {
                                value,
                                expected_type: stringify!(Reference),
                            },
                        )
                    })
                })
                .transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::null::Null;
    use crate::parse::Span;

    // TODO Add tests, e.g. the trailer dictionaries used in xref
    // #[test]
    // fn dictionary_valid() {}

    #[test]
    fn dictionary_invalid() {
        // Synthetic tests

        // Dictionary: Not found
        let parsed_result = Dictionary::parse(b"/Type /Type1");
        let expected_error = ParseRecoverable::new(
            b"/Type /Type1",
            stringify!(Dictionary),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Single quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1");
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Dictionary),
            ParseErrorCode::RecMissingClosing(Box::new(ParseErrorCode::NotFound(ErrorKind::Char))),
        );
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Spaced quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype << /Type /Type2> > >>");
        let expected_error = ParseFailure::new(
            b"> > >>",
            stringify!(Dictionary),
            ParseErrorCode::RecMissingValue(
                b"Subtype".to_vec(),
                Box::new(ParseErrorCode::RecMissingClosing(Box::new(
                    ParseErrorCode::NotFound(ErrorKind::Char),
                ))),
            ),
        );
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Missing value
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype >>");
        let expected_error = ParseFailure::new(
            b">>",
            stringify!(Dictionary),
            ParseErrorCode::RecMissingValue(
                b"Subtype".to_vec(),
                Box::new(ParseErrorCode::NotFoundUnion),
            ),
        );
        assert_err_eq!(parsed_result, expected_error);
    }

    #[test]
    fn dictionary_escape() {
        assert_eq!(
            Dictionary::from_iter([("Key".into(), Null::new(Span::new(5, 4)).into())]),
            Dictionary::default()
        );

        // TODO Add tests
    }
}
