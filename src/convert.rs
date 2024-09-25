#[macro_export]
macro_rules! impl_from_ref {
    ($lifetime:lifetime, $from:ty, $variant:ident, $to:ty) => {
        impl<$lifetime> From<$from> for $to {
            fn from(value: $from) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from {
    ($from:ty, $variant:ident, $to:ty) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}
