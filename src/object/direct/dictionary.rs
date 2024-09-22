use ::nom::bytes::complete::tag;
use ::nom::combinator::opt;
use ::nom::error::Error as NomError;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::collections::HashMap;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::DictionaryFailure;
use self::error::DictionaryRecoverable;
use super::name::Name;
use super::DirectValue;
use crate::fmt::debug_bytes;
use crate::object::indirect::reference::Reference;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse_error;
use crate::Byte;

/// REFERENCE: [7.3.7 Dictionary objects, p30-31]
#[derive(Debug, Default, Clone)]
pub struct Dictionary(HashMap<Name, DirectValue>);

impl Display for Dictionary {
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

impl PartialEq for Dictionary {
    fn eq(&self, other: &Self) -> bool {
        self.escape() == other.escape()
    }
}

impl Parser for Dictionary {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let mut dictionary = HashMap::default();
        let mut key: Name;
        let mut value: DirectValue;
        let (mut buffer, _) =
            terminated(tag(b"<<"), opt(white_space_or_comment))(buffer).map_err(parse_error!(
                e,
                DictionaryRecoverable::NotFound {
                    code: e.code,
                    input: debug_bytes(buffer)
                }
            ))?;
        // Here, we know that the buffer starts with a dictionary, and the
        // following errors should be propagated as DictionaryFailure
        loop {
            // Check for the end of the dictionary (closing angle brackets)
            if let Ok((remaining, _)) = tag::<_, _, NomError<_>>(b">>")(buffer) {
                buffer = remaining;
                break;
            }
            // Parse the key
            (buffer, key) = Name::parse_semi_quiet::<Name>(buffer).unwrap_or_else(|| {
                Err(ParseErr::Failure(
                    DictionaryFailure::MissingClosing(debug_bytes(buffer)).into(),
                ))
            })?;
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remaining, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remaining;
            }
            // Parse the value
            (buffer, value) =
                DirectValue::parse_semi_quiet::<DirectValue>(buffer).unwrap_or_else(|| {
                    Err(ParseErr::Failure(
                        DictionaryFailure::MissingValue {
                            key: key.clone(),
                            input: debug_bytes(buffer),
                        }
                        .into(),
                    ))
                })?;
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remaining, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remaining;
            }
            // Record the key-value pair
            if let Some(old_value) = dictionary.insert(key.clone(), value.clone()) {
                // Dictionary keys should not be duplicated.
                // REFERENCE: [7.3.7 Dictionary objects, p30]
                //
                // Print only if verbose mode is enabled.
                // TODO Replace with a `log::error!` call
                eprintln!(
                    "Dictionary: Overwriting value for key {}: {} -> {}",
                    key, old_value, value
                );
            };
        }

        let dictionary = Dictionary(dictionary);
        Ok((buffer, dictionary))
    }
}

mod process {
    use super::*;
    use crate::object::direct::null::Null;

    impl Dictionary {
        /// REFERENCE: [7.3.7 Dictionary objects, p30] and [7.3.9 Null object, p33]
        pub(crate) fn escape(&self) -> HashMap<&Name, &DirectValue> {
            // FIXME Take into account values that are references to missing
            // objects, which is the same as having the value null. Also,
            // consider the effect of two references pointing to the same
            // object.
            self.0
                .iter()
                .filter(|(_, value)| *value != &Null.into())
                .collect()
        }
    }
}

mod convert {
    use ::std::ops::Deref;
    use ::std::result::Result as StdResult;

    use self::error::DataTypeError;
    use super::*;
    use crate::object::direct::array::Array;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Numeric;

    impl From<HashMap<Name, DirectValue>> for Dictionary {
        fn from(value: HashMap<Name, DirectValue>) -> Self {
            Self(value)
        }
    }

    impl FromIterator<(Name, DirectValue)> for Dictionary {
        fn from_iter<T: IntoIterator<Item = (Name, DirectValue)>>(iter: T) -> Dictionary {
            Self(HashMap::from_iter(iter))
        }
    }

    impl Deref for Dictionary {
        type Target = HashMap<Name, DirectValue>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl IntoIterator for Dictionary {
        type Item = (Name, DirectValue);
        type IntoIter = <HashMap<Name, DirectValue> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl Dictionary {
        pub(crate) fn get(&self, key: &str) -> Option<&DirectValue> {
            self.0.get(&key.into())
        }

        pub(crate) fn get_array<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<&Array>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value.as_array().ok_or_else(|| DataTypeError {
                        key,
                        expected_type: stringify!(u64),
                        value: value.to_string(),
                        dictionary: self.to_string(),
                    })
                })
                .transpose()
        }

        pub(crate) fn get_name<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<&Name>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value.as_name().ok_or_else(|| DataTypeError {
                        key,
                        expected_type: stringify!(u64),
                        value: value.to_string(),
                        dictionary: self.to_string(),
                    })
                })
                .transpose()
        }

        pub(crate) fn get_u64<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<u64>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_u64)
                        .ok_or_else(|| DataTypeError {
                            key,
                            expected_type: stringify!(u64),
                            value: value.to_string(),
                            dictionary: self.to_string(),
                        })
                })
                .transpose()
        }

        pub(crate) fn get_reference<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<&Reference>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value.as_reference().ok_or_else(|| DataTypeError {
                        key,
                        expected_type: stringify!(Reference),
                        value: value.to_string(),
                        dictionary: self.to_string(),
                    })
                })
                .transpose()
        }
    }
}

pub(crate) mod error {
    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    use crate::object::direct::name::Name;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum DictionaryRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum DictionaryFailure {
        #[error("Missing Value for key {key}. Input: {input}")]
        MissingValue { key: Name, input: String },
        #[error("Missing Closing: Expected a name or closing angle brackets. Input: {0}")]
        MissingClosing(String),
    }

    // TODO(TEMP) Replace the two errors below to DictionaryError
    #[derive(Debug, Error, PartialEq, Clone)]
    #[error(
        "Wrong data type. Key {key}. Expected a {expected_type} value, found {value} in \
         {dictionary}"
    )]
    pub struct DataTypeError<'key> {
        pub(crate) key: &'key str,
        pub(crate) expected_type: &'static str,
        pub(crate) value: String,      // TODO (TEMP)
        pub(crate) dictionary: String, // TODO (TEMP)
    }

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    #[error("Missing required entry. Key {key}. Expected a {data_type} value")]
    pub struct MissingEntryError {
        pub(crate) key: &'static str,
        pub(crate) data_type: &'static str,
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::null::Null;
    use crate::parse::error::ParseFailure;

    // TODO Add tests, e.g. the trailer dictionaries used in xref
    // #[test]
    // fn dictionary_valid() {}

    #[test]
    fn dictionary_invalid() {
        // Synthetic tests

        // Dictionary: Not found
        let parsed_result = Dictionary::parse(b"/Type /Type1");
        let expected_error = ParseErr::Error(
            DictionaryRecoverable::NotFound {
                code: ErrorKind::Tag,
                input: "/Type /Type1".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Single quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1");
        let expected_error = ParseErr::Failure(ParseFailure::Dictionary(
            DictionaryFailure::MissingClosing("".to_string()),
        ));
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Spaced quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype << /Type /Type2> > >>");
        let expected_error = ParseErr::Failure(ParseFailure::Dictionary(
            DictionaryFailure::MissingClosing("> > >>".to_string()),
        ));
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Missing value
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype >>");
        let expected_error =
            ParseErr::Failure(ParseFailure::Dictionary(DictionaryFailure::MissingValue {
                key: Name::from("Subtype"),
                input: ">>".to_string(),
            }));
        assert_err_eq!(parsed_result, expected_error);
    }

    #[test]
    fn dictionary_escape() {
        assert_eq!(
            Dictionary::from_iter([("Key".into(), Null.into())]),
            Dictionary::default()
        );

        // TODO Add tests
    }
}
