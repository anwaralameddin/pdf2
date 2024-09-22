use ::nom::bytes::complete::tag;
use ::nom::error::Error as NomError;
use ::nom::error::ErrorKind;
use ::nom::Err as NomErr;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::error::ReferenceRecoverable;
use super::id::Id;
use crate::fmt::debug_bytes;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::KW_R;
use crate::parse_error;
use crate::Byte;

/// REFERENCE: [7.3.10 Indirect Objects, p33]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reference(Id);

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.0, KW_R)
    }
}

impl Parser for Reference {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (buffer, id) = Id::parse_semi_quiet(buffer).unwrap_or_else(|| {
            Err(ParseErr::Error(
                ReferenceRecoverable::NotFound {
                    code: ErrorKind::Digit,
                    input: debug_bytes(buffer),
                }
                .into(),
            ))
        })?;
        // At this point, even though we have an Id, it is unclear if it is a
        // reference or a sequence of integers. For example, `12 0` appearing in
        // an array can be part of the indirect reference `12 0 R` or simply a
        // pair of integers in that array.
        let (buffer, _) =
            tag::<_, _, NomError<_>>(KW_R.as_bytes())(buffer).map_err(parse_error!(
                e,
                ReferenceRecoverable::NotFound {
                    code: e.code,
                    input: debug_bytes(e.input),
                }
            ))?;

        let reference = Self(id);
        Ok((buffer, reference))
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<Id> for Reference {
        fn from(value: Id) -> Self {
            Self(value)
        }
    }

    impl Deref for Reference {
        type Target = Id;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub(crate) mod error {

    use ::nom::error::ErrorKind;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum ReferenceRecoverable {
        #[error("Not found: {code:?}. Input: {input}")]
        NotFound { code: ErrorKind, input: String },
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::*;
    use crate::assert_err_eq;
    use crate::parse_assert_eq;
    use crate::GenerationNumber;

    impl Reference {
        pub(crate) unsafe fn new_unchecked(
            object_number: u64,
            generation_number: GenerationNumber,
        ) -> Self {
            Self(Id::new_unchecked(object_number, generation_number))
        }
    }

    #[test]
    fn reference_valid() {
        // Synthetic tests
        let reference = unsafe { Reference::new_unchecked(1, 0) };
        parse_assert_eq!(b"1 0 R", reference, "".as_bytes());
        let reference = unsafe { Reference::new_unchecked(12345, 65535) };
        parse_assert_eq!(b"12345 65535 R<<", reference, "<<".as_bytes());
        parse_assert_eq!(
            b"12345 65535 Rc",
            unsafe { Reference::new_unchecked(12345, 65535) },
            "c".as_bytes()
        );
    }

    #[test]
    fn reference_invalid() {
        // Synthetic tests
        // Reference: Incomplete
        let parse_result = Reference::parse(b"1 0");
        let expected_error = ParseErr::Error(
            ReferenceRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "1 0".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Reference: Id not found
        let parse_result = Reference::parse(b"/Name");
        let expected_error = ParseErr::Error(
            ReferenceRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "/Name".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Reference: Id error
        let parse_result = Reference::parse(b"0 65535 R other objects");
        let expected_error = ParseErr::Error(
            ReferenceRecoverable::NotFound {
                code: ErrorKind::Digit,
                input: "0 65535 R other objects".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);

        // Reference: Not found
        let parse_result = Reference::parse(b"12345 65535 <");
        let expected_error = ParseErr::Error(
            ReferenceRecoverable::NotFound {
                code: ErrorKind::Tag,
                input: "<".to_string(),
            }
            .into(),
        );
        assert_err_eq!(parse_result, expected_error);
    }
}
