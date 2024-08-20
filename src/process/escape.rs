pub(crate) mod error {
    use ::std::num::ParseIntError;
    use ::std::str::Utf8Error;
    use ::thiserror::Error;

    use crate::object::direct::name::error::NameEscape;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum EscapeError {
        #[error(transparent)]
        Name(#[from] NameEscape),
        #[error("Utf8Error: {0}")]
        Utf8Error(#[from] Utf8Error),
        #[error("ParseIntError: {0}")]
        ParseIntError(#[from] ParseIntError),
    }
}
