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
