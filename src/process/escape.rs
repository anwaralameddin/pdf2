pub(crate) mod error {
    use ::std::num::ParseIntError;
    use ::thiserror::Error;

    use crate::object::direct::name::error::NameEscape;

    #[derive(Debug, Error, PartialEq, Clone)]
    pub enum EscapeError {
        #[error(transparent)]
        Name(#[from] NameEscape),
        #[error("ParseIntError: {0}")]
        ParseIntError(#[from] ParseIntError),
    }
}
