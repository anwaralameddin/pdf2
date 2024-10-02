use ::nom::bytes::complete::tag;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::collections::HashMap;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use super::name::Name;
use super::DirectValue;
use crate::fmt::debug_bytes;
use crate::object::indirect::reference::Reference;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Span;
use crate::parse_recoverable;
use crate::process::escape::Escape;
use crate::Byte;
use crate::Offset;

/// REFERENCE: [7.3.7 Dictionary objects, p30-31]
#[derive(Debug, Clone)]
pub struct Dictionary<'buffer> {
    map: HashMap<Vec<Byte>, DirectValue<'buffer>>,
    span: Span,
}

impl Display for Dictionary<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<<")?;
        for (i, (key, value)) in self.map.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "/{} {}", debug_bytes(key), value)?;
        }
        write!(f, ">>")
    }
}

impl PartialEq for Dictionary<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.escape() == other.escape() && self.span == other.span
    }
}

impl<'buffer> ObjectParser<'buffer> for Dictionary<'buffer> {
    fn parse(buffer: &'buffer [Byte], offset: Offset) -> ParseResult<Self> {
        let remains = &buffer[offset..];
        let remains_len = remains.len();
        let start = offset;

        let (mut remains, recognised) =
            recognize(terminated(tag(b"<<"), opt(white_space_or_comment)))(remains).map_err(
                parse_recoverable!(
                    e,
                    ParseRecoverable::new(
                        e.input,
                        stringify!(Dictionary),
                        ParseErrorCode::NotFound(e.code),
                    )
                ),
            )?;
        let mut offset = offset + recognised.len();
        // Here, we know that the buffer starts with a dictionary, and the
        // following errors should be propagated as DictionaryFailure

        let mut map = HashMap::default();
        let mut key: Name;
        let mut value: DirectValue;
        loop {
            // Check for the end of the dictionary (closing angle brackets)
            if let Ok((buf, _)) = tag::<_, _, NomError<_>>(b">>")(remains) {
                remains = buf;
                break;
            }
            // Parse the key
            key = Name::parse(buffer, offset).map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(Dictionary),
                    ParseErrorCode::RecMissingClosing(Box::new(err.code())),
                )
            })?;
            offset = key.span().end();
            remains = &buffer[offset..];
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((_, recognised)) = recognize(opt(white_space_or_comment))(remains) {
                offset += recognised.len();
            }
            // Parse the value
            value = DirectValue::parse(buffer, offset).map_err(|err| {
                ParseFailure::new(
                    err.buffer(),
                    stringify!(Dictionary),
                    ParseErrorCode::RecMissingValue(key.to_vec(), Box::new(err.code())),
                )
            })?;
            offset = value.span().end();
            remains = &buffer[offset..];
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((buf, recognised)) = recognize(opt(white_space_or_comment))(remains) {
                remains = buf;
                offset += recognised.len();
            }
            // Record the key-value pair

            let escpaed_key = if let Ok(escpaed_key) = key.escape() {
                escpaed_key
            } else {
                eprintln!("Failed to escape key: {}", key);
                key.to_vec()
            };

            if let Some(old_value) = map.insert(escpaed_key.clone(), value) {
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
                    map.get(&escpaed_key)
                );
            };
        }

        let span = Span::new(start, remains_len - remains.len());
        Ok(Self { map, span })
    }

    fn span(&self) -> Span {
        self.span
    }
}

mod escape {
    use super::*;

    impl Dictionary<'_> {
        /// REFERENCE: [7.3.7 Dictionary objects, p30] and [7.3.9 Null object, p33]
        pub(crate) fn escape(&self) -> HashMap<&Vec<Byte>, &DirectValue> {
            // FIXME Take into account values that are references to missing
            // objects, which is the same as having the value null. Also,
            // consider the effect of two references pointing to the same
            // object.
            self.map
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

    impl<'buffer> Dictionary<'buffer> {
        pub fn new(
            map: impl IntoIterator<Item = (Vec<Byte>, DirectValue<'buffer>)>,
            span: Span,
        ) -> Self {
            Self {
                map: map.into_iter().collect(),
                span,
            }
        }
    }

    impl<'buffer> Deref for Dictionary<'buffer> {
        type Target = HashMap<Vec<Byte>, DirectValue<'buffer>>;

        fn deref(&self) -> &Self::Target {
            &self.map
        }
    }

    impl<'buffer> IntoIterator for Dictionary<'buffer> {
        type Item = (Vec<Byte>, DirectValue<'buffer>);
        type IntoIter = <HashMap<Vec<Byte>, DirectValue<'buffer>> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.map.into_iter()
        }
    }

    impl<'buffer> Dictionary<'buffer> {
        pub(crate) fn opt_get(&'buffer self, key: &'static [Byte]) -> Option<&DirectValue> {
            self.map.get(key)
        }

        pub(crate) fn required_get(
            &'buffer self,
            key: &'static [Byte],
        ) -> ObjectResult<&DirectValue> {
            self.opt_get(key).ok_or_else(|| {
                ObjectErr::new(key, self.span(), ObjectErrorCode::MissingRequiredEntry)
            })
        }

        pub(crate) fn opt_array(&self, key: &'static [Byte]) -> ObjectResult<Option<&Array>> {
            self.opt_get(key)
                .map(|value| {
                    value.as_array().ok_or_else(|| {
                        ObjectErr::new(
                            key,
                            self.span(),
                            ObjectErrorCode::Type {
                                value_span: value.span(),
                                expected_type: stringify!(Array),
                            },
                        )
                    })
                })
                .transpose()
        }

        pub(crate) fn required_array(&self, key: &'static [Byte]) -> ObjectResult<&Array> {
            self.opt_array(key).and_then(|value| {
                value.ok_or_else(|| {
                    ObjectErr::new(key, self.span(), ObjectErrorCode::MissingRequiredEntry)
                })
            })
        }

        pub(crate) fn opt_name(&self, key: &'static [Byte]) -> ObjectResult<Option<&Name>> {
            self.opt_get(key)
                .map(|value| {
                    value.as_name().ok_or_else(|| {
                        ObjectErr::new(
                            key,
                            self.span(),
                            ObjectErrorCode::Type {
                                value_span: value.span(),
                                expected_type: stringify!(Name),
                            },
                        )
                    })
                })
                .transpose()
        }

        pub(crate) fn required_name(&self, key: &'static [Byte]) -> ObjectResult<&Name> {
            self.opt_name(key).and_then(|value| {
                value.ok_or_else(|| {
                    ObjectErr::new(key, self.span(), ObjectErrorCode::MissingRequiredEntry)
                })
            })
        }

        pub(crate) fn opt_u64(&self, key: &'static [Byte]) -> ObjectResult<Option<u64>> {
            self.opt_get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_u64)
                        .ok_or_else(|| {
                            ObjectErr::new(
                                key,
                                self.span(),
                                ObjectErrorCode::Type {
                                    value_span: value.span(),
                                    expected_type: stringify!(u64),
                                },
                            )
                        })
                })
                .transpose()
        }

        pub(crate) fn required_u64(&self, key: &'static [Byte]) -> ObjectResult<u64> {
            self.opt_u64(key).and_then(|value| {
                value.ok_or_else(|| {
                    ObjectErr::new(key, self.span(), ObjectErrorCode::MissingRequiredEntry)
                })
            })
        }

        pub(crate) fn opt_usize(&self, key: &'static [Byte]) -> ObjectResult<Option<usize>> {
            self.opt_get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_usize)
                        .ok_or_else(|| {
                            ObjectErr::new(
                                key,
                                self.span(),
                                ObjectErrorCode::Type {
                                    value_span: value.span(),
                                    expected_type: stringify!(usize),
                                },
                            )
                        })
                })
                .transpose()
        }

        pub(crate) fn required_usize(&self, key: &'static [Byte]) -> ObjectResult<usize> {
            self.opt_usize(key).and_then(|value| {
                value.ok_or_else(|| {
                    ObjectErr::new(key, self.span(), ObjectErrorCode::MissingRequiredEntry)
                })
            })
        }

        pub(crate) fn opt_reference(
            &self,
            key: &'static [Byte],
        ) -> ObjectResult<Option<&Reference>> {
            self.opt_get(key)
                .map(|value| {
                    value.as_reference().ok_or_else(|| {
                        ObjectErr::new(
                            key,
                            self.span(),
                            ObjectErrorCode::Type {
                                value_span: value.span(),
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
        let parsed_result = Dictionary::parse(b"/Type /Type1", 0);
        let expected_error = ParseRecoverable::new(
            b"/Type /Type1",
            stringify!(Dictionary),
            ParseErrorCode::NotFound(ErrorKind::Tag),
        );
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Single quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1", 0);
        let expected_error = ParseFailure::new(
            b"",
            stringify!(Dictionary),
            ParseErrorCode::RecMissingClosing(Box::new(ParseErrorCode::NotFound(ErrorKind::Char))),
        );
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Spaced quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype << /Type /Type2> > >>", 0);
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
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype >>", 0);
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
            Dictionary::new(
                HashMap::from([(b"Key".to_vec(), Null::new(Span::new(5, 4)).into())]),
                Span::new(0, 9)
            ),
            Dictionary::new([], Span::new(0, 9))
        );

        // TODO Add tests
    }
}
