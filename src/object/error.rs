use ::thiserror::Error;

use crate::error::DisplayUsingBuffer;
use crate::fmt::debug_bytes;
use crate::parse::Span;
use crate::Byte;
use crate::GenerationNumber;
use crate::ObjectNumber;

pub(crate) type ObjectResult<T> = Result<T, ObjectErr>;

#[derive(Debug, Error, PartialEq, Clone, Copy)]
#[error("Object. Key: {}. Error: {code}. Dictionary: {dictionary_span}", debug_bytes(.key))]
pub struct ObjectErr {
    pub(crate) key: &'static [Byte],
    pub(crate) dictionary_span: Span,
    pub(crate) code: ObjectErrorCode,
}

impl DisplayUsingBuffer for ObjectErr {
    fn display_using_buffer(&self, buffer: &[Byte]) -> String {
        format!(
            "Object. Key: {}. Error: {}. Dictionary: {}",
            debug_bytes(self.key),
            self.code.display_using_buffer(buffer),
            debug_bytes(&buffer[self.dictionary_span])
        )
    }
}

#[derive(Debug, Error, PartialEq, Clone, Copy)]
pub enum ObjectErrorCode {
    #[error("Wrong value. Expetced: {}. Found: {value_span}", debug_bytes(.expected))]
    Value {
        expected: &'static [Byte],
        value_span: Span,
    },
    #[error("Wrong value type. Expetced type: {expected_type}. Found: {value_span}")]
    Type {
        expected_type: &'static str,
        value_span: Span,
    },
    #[error("Cyclic reference: {0} {1}")]
    CyclicReference(ObjectNumber, GenerationNumber),
    #[error("Missing referenced object: {0} {1}")]
    MissingReferencedObject(ObjectNumber, GenerationNumber),
    #[error("Reference resolves to a stream: {0} {1}")]
    ReferenceToStream(ObjectNumber, GenerationNumber),
    #[error("Missing required entry")]
    MissingRequiredEntry,
}

impl DisplayUsingBuffer for ObjectErrorCode {
    fn display_using_buffer(&self, buffer: &[Byte]) -> String {
        match self {
            Self::Value {
                expected,
                value_span,
            } => {
                format!(
                    "Wrong value. Expected: {}. Found: {}",
                    debug_bytes(expected),
                    debug_bytes(&buffer[*value_span])
                )
            }
            Self::Type {
                expected_type,
                value_span,
            } => {
                format!(
                    "Wrong value type. Expected type: {}. Found: {}",
                    expected_type,
                    debug_bytes(&buffer[*value_span])
                )
            }
            _ => self.to_string(),
        }
    }
}

mod convert {
    use super::*;
    impl ObjectErr {
        pub fn new(key: &'static [Byte], dictionary_span: Span, code: ObjectErrorCode) -> Self {
            Self {
                key,
                dictionary_span,
                code,
            }
        }
    }
}
