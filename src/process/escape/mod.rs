pub(crate) mod error;

use self::error::EscapeResult;
use crate::Byte;

pub(crate) trait Escape {
    fn escape(&self) -> EscapeResult<Vec<Byte>>;
}

mod tests {
    #[macro_export]
    macro_rules! escape_assert_err {
        ($object:expr, $expected_error:expr) => {
            assert_eq!($object.escape(), Err($expected_error));
        };
    }
}
