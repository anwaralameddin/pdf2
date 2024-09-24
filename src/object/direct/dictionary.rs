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
use super::name::OwnedName;
use super::DirectValue;
use super::OwnedDirectValue;
use crate::object::indirect::reference::Reference;
use crate::object::BorrowedBuffer;
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

#[derive(Debug, Default, Clone)]
pub struct OwnedDictionary(HashMap<OwnedName, OwnedDirectValue>);

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

impl Display for OwnedDictionary {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // TODO (TEMP) Dictionary::from involves reallocation
        Display::fmt(&Dictionary::from(self), f)
    }
}

impl PartialEq for Dictionary<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.escape() == other.escape()
    }
}

impl PartialEq for OwnedDictionary {
    fn eq(&self, other: &Self) -> bool {
        // TODO (TEMP) Dictionary::from involves reallocation
        Dictionary::from(self) == Dictionary::from(other)
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
                ParseRecoverable {
                    buffer: e.input,
                    object: stringify!(Dictionary),
                    code: ParseErrorCode::NotFound(e.code),
                }
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
            (buffer, key) = Name::parse(buffer).map_err(|err| ParseFailure {
                buffer: err.buffer(),
                object: stringify!(Dictionary),
                code: ParseErrorCode::RecMissingClosing(Box::new(err.code())),
            })?;
            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remains, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remains;
            }
            // Parse the value
            (buffer, value) = DirectValue::parse(buffer).map_err(|err| ParseFailure {
                buffer: err.buffer(),
                object: stringify!(Dictionary),
                code: ParseErrorCode::RecMissingValueCloned(
                    key.to_owned_buffer(), // TODO (TEMP) Consider refactoring to avoid cloning
                    Box::new(err.code()),
                ),
            })?;

            // opt does not return an error, so there is no need for specific
            // error handling
            if let Ok((remains, _)) = opt(white_space_or_comment)(buffer) {
                buffer = remains;
            }
            // Record the key-value pair
            if let Some(old_value) = dictionary.insert(key, value.clone()) {
                // TODO (TEMP) Consider refactoring to avoid cloning
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

        let dictionary = Self(dictionary);
        Ok((buffer, dictionary))
    }
}

impl Parser<'_> for OwnedDictionary {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        Dictionary::parse(buffer)
            .map(|(remains, dictionary)| (remains, dictionary.to_owned_buffer()))
    }
}

mod process {
    use super::*;
    use crate::object::direct::null::Null;

    impl Dictionary<'_> {
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

    impl OwnedDictionary {
        /// REFERENCE: [7.3.7 Dictionary objects, p30] and [7.3.9 Null object, p33]
        pub(crate) fn escape(&self) -> HashMap<&OwnedName, &OwnedDirectValue> {
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
    use crate::object::direct::array::OwnedArray;
    use crate::object::direct::numeric::Integer;
    use crate::object::direct::numeric::Numeric;
    use crate::object::BorrowedBuffer;

    impl BorrowedBuffer for Dictionary<'_> {
        type OwnedBuffer = OwnedDictionary;

        fn to_owned_buffer(self) -> Self::OwnedBuffer {
            OwnedDictionary(
                self.0
                    .into_iter()
                    .map(|(key, value)| (key.to_owned_buffer(), value.to_owned_buffer()))
                    .collect(),
            )
        }
    }

    impl<'buffer> From<&'buffer OwnedDictionary> for Dictionary<'buffer> {
        fn from(value: &'buffer OwnedDictionary) -> Self {
            Dictionary(
                value
                    .0
                    .iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect(),
            )
        }
    }

    impl<'buffer> From<HashMap<Name<'buffer>, DirectValue<'buffer>>> for Dictionary<'buffer> {
        fn from(value: HashMap<Name<'buffer>, DirectValue<'buffer>>) -> Self {
            Self(value)
        }
    }

    impl From<HashMap<OwnedName, OwnedDirectValue>> for OwnedDictionary {
        fn from(value: HashMap<OwnedName, OwnedDirectValue>) -> Self {
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

    impl FromIterator<(OwnedName, OwnedDirectValue)> for OwnedDictionary {
        fn from_iter<T: IntoIterator<Item = (OwnedName, OwnedDirectValue)>>(
            iter: T,
        ) -> OwnedDictionary {
            Self(HashMap::from_iter(iter))
        }
    }

    impl<'buffer> Deref for Dictionary<'buffer> {
        type Target = HashMap<Name<'buffer>, DirectValue<'buffer>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Deref for OwnedDictionary {
        type Target = HashMap<OwnedName, OwnedDirectValue>;

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

    impl IntoIterator for OwnedDictionary {
        type Item = (OwnedName, OwnedDirectValue);
        type IntoIter = <HashMap<OwnedName, OwnedDirectValue> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl Dictionary<'_> {
        pub(crate) fn get(&self, key: &'static str) -> Option<&DirectValue> {
            self.0.get(&Name::from(key))
        }

        pub(crate) fn get_array(
            &self,
            key: &'static str,
        ) -> StdResult<Option<&Array>, DataTypeError<'static>> {
            // <'static>
            self.get(key)
                .map(|value| {
                    value.as_array().ok_or_else(|| DataTypeError {
                        entry: key,
                        expected_type: stringify!(u64),
                        value: value.to_string(),
                        object: self.to_string(),
                    })
                })
                .transpose()
        }

        pub(crate) fn get_name(
            &self,
            key: &'static str,
        ) -> StdResult<Option<&Name>, DataTypeError<'static>> {
            self.get(key)
                .map(|value| {
                    value.as_name().ok_or_else(|| DataTypeError {
                        entry: key,
                        expected_type: stringify!(u64),
                        value: value.to_string(),
                        object: self.to_string(),
                    })
                })
                .transpose()
        }

        pub(crate) fn get_u64(
            &self,
            key: &'static str,
        ) -> StdResult<Option<u64>, DataTypeError<'static>> {
            self.get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_u64)
                        .ok_or_else(|| DataTypeError {
                            entry: key,
                            expected_type: stringify!(u64),
                            value: value.to_string(),
                            object: self.to_string(),
                        })
                })
                .transpose()
        }

        pub(crate) fn get_usize(
            &self,
            key: &'static str,
        ) -> StdResult<Option<usize>, DataTypeError<'static>> {
            self.get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_usize)
                        .ok_or_else(|| DataTypeError {
                            entry: key,
                            expected_type: stringify!(usize),
                            value: value.to_string(),
                            object: self.to_string(),
                        })
                })
                .transpose()
        }

        pub(crate) fn get_reference(
            &self,
            key: &'static str,
        ) -> StdResult<Option<&Reference>, DataTypeError<'static>> {
            self.get(key)
                .map(|value| {
                    value.as_reference().ok_or_else(|| DataTypeError {
                        entry: key,
                        expected_type: stringify!(Reference),
                        value: value.to_string(),
                        object: self.to_string(),
                    })
                })
                .transpose()
        }
    }

    // TODO (TEMP) Avoid this duplication
    impl OwnedDictionary {
        pub(crate) fn get(&self, key: &str) -> Option<&OwnedDirectValue> {
            self.0.get(&key.into())
        }

        pub(crate) fn get_array<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<&OwnedArray>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value.as_array().ok_or_else(|| DataTypeError {
                        entry: key,
                        expected_type: stringify!(u64),
                        value: value.to_string(),
                        object: self.to_string(),
                    })
                })
                .transpose()
        }

        pub(crate) fn get_name<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<&OwnedName>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value.as_name().ok_or_else(|| DataTypeError {
                        entry: key,
                        expected_type: stringify!(u64),
                        value: value.to_string(),
                        object: self.to_string(),
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
                            entry: key,
                            expected_type: stringify!(u64),
                            value: value.to_string(),
                            object: self.to_string(),
                        })
                })
                .transpose()
        }

        pub(crate) fn get_usize<'key>(
            &self,
            key: &'key str,
        ) -> StdResult<Option<usize>, DataTypeError<'key>> {
            self.get(key)
                .map(|value| {
                    value
                        .as_numeric()
                        .and_then(Numeric::as_integer)
                        .and_then(Integer::as_usize)
                        .ok_or_else(|| DataTypeError {
                            entry: key,
                            expected_type: stringify!(usize),
                            value: value.to_string(),
                            object: self.to_string(),
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
                        entry: key,
                        expected_type: stringify!(Reference),
                        value: value.to_string(),
                        object: self.to_string(),
                    })
                })
                .transpose()
        }
    }
}

pub(crate) mod error {
    use ::thiserror::Error;

    // TODO(TEMP) Replace the two errors below to DictionaryError
    #[derive(Debug, Error, PartialEq, Clone)]
    #[error(
        "Data type. entry {entry}. Expected a {expected_type} value, found {value} in {object}"
    )]
    pub struct DataTypeError<'entry> {
        pub(crate) entry: &'entry str,
        pub(crate) expected_type: &'static str,
        pub(crate) value: String, // TODO (TEMP) Refactor to avoid changing the type to String
        pub(crate) object: String, // TODO (TEMP) Refactor to avoid changing the type to String
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

    // TODO Add tests, e.g. the trailer dictionaries used in xref
    // #[test]
    // fn dictionary_valid() {}

    #[test]
    fn dictionary_invalid() {
        // Synthetic tests

        // Dictionary: Not found
        let parsed_result = Dictionary::parse(b"/Type /Type1");
        let expected_error = ParseRecoverable {
            buffer: b"/Type /Type1",
            object: stringify!(Dictionary),
            code: ParseErrorCode::NotFound(ErrorKind::Tag),
        };
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Single quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1");
        let expected_error = ParseFailure {
            buffer: b"",
            object: stringify!(Dictionary),
            code: ParseErrorCode::RecMissingClosing(Box::new(ParseErrorCode::NotFound(
                ErrorKind::Char,
            ))),
        };
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Spaced quotes
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype << /Type /Type2> > >>");
        let expected_error = ParseFailure {
            buffer: b"> > >>",
            object: stringify!(Dictionary),
            code: ParseErrorCode::RecMissingValueCloned(
                OwnedName::from("Subtype"),
                Box::new(ParseErrorCode::RecMissingClosing(Box::new(
                    ParseErrorCode::NotFound(ErrorKind::Char),
                ))),
            ),
        };
        assert_err_eq!(parsed_result, expected_error);

        // Dictionary: Missing value
        let parsed_result = Dictionary::parse(b"<< /Type /Type1 /Subtype >>");
        let expected_error = ParseFailure {
            buffer: b">>",
            object: stringify!(Dictionary),
            code: ParseErrorCode::RecMissingValueCloned(
                OwnedName::from("Subtype"),
                Box::new(ParseErrorCode::NotFoundUnion),
            ),
        };
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
