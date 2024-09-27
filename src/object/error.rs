use ::thiserror::Error;

use super::direct::array::Array;
use super::direct::dictionary::Dictionary;
use super::direct::name::Name;
use super::direct::DirectValue;

pub(crate) type ObjectResult<'buffer, T> = Result<T, ObjectErr<'buffer>>;

#[derive(Debug, Error, PartialEq, Clone)]
#[error("Object. Key: {key}. Error: {code} in {dictionary}")]
pub struct ObjectErr<'buffer> {
    key: &'static str,
    dictionary: &'buffer Dictionary<'buffer>,
    code: ObjectErrorCode<'buffer>,
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ObjectErrorCode<'buffer> {
    #[error("Wrong name. Expetced {expected}. Input {value}")]
    Name {
        expected: &'static str,
        value: &'buffer Name<'buffer>,
    },
    #[error("Wrong array. Expetced {expected}. Input {value}")]
    Array {
        expected: &'static str,
        value: &'buffer Array<'buffer>,
    },
    #[error("Wrong value type. Expetced {expected_type}. Input {value}")]
    Type {
        expected_type: &'static str,
        value: &'buffer DirectValue<'buffer>,
    },
    #[error("Missing required entry")]
    MissingRequiredEntry,
}

mod convert {
    use super::*;
    impl<'buffer> ObjectErr<'buffer> {
        pub fn new(
            key: &'static str,
            dictionary: &'buffer Dictionary<'buffer>,
            code: ObjectErrorCode<'buffer>,
        ) -> Self {
            Self {
                key,
                dictionary,
                code,
            }
        }
    }
}
