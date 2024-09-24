pub(crate) mod error {
    use ::std::num::ParseIntError;
    use ::thiserror::Error;

    use crate::object::direct::name::error::NameEscape;
    use crate::object::direct::name::error::OwnedNameEscape;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum EscapeError<'buffer> {
        #[error(transparent)]
        Name(NameEscape<'buffer>),
        #[error("ParseIntError: {0}")]
        ParseIntError(#[from] ParseIntError),
    }

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum OwnedEscapeError {
        #[error(transparent)]
        Name(#[from] OwnedNameEscape),
        #[error("ParseIntError: {0}")]
        ParseIntError(#[from] ParseIntError),
    }
}

mod convert {
    use super::error::EscapeError;
    use super::error::OwnedEscapeError;
    use crate::object::direct::name::error::NameEscape;

    // TODO (TEMP) Replace this repetitive implementation with a macro
    impl<'buffer> From<NameEscape<'buffer>> for EscapeError<'buffer> {
        fn from(err: NameEscape<'buffer>) -> Self {
            Self::Name(err)
        }
    }

    // TODO (TEMP) Remove this when ProcessErr is refactored to accept lifetime
    // parameters
    impl From<EscapeError<'_>> for OwnedEscapeError {
        fn from(err: EscapeError) -> Self {
            match err {
                EscapeError::Name(name) => Self::Name(name.into()),
                EscapeError::ParseIntError(parse_int_error) => Self::ParseIntError(parse_int_error),
            }
        }
    }
}
