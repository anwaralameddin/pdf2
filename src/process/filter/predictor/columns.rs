/// REFERENCE: [Table 8 — Optional parameters for LZWDecode and FlateDecode
/// filters, p40]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Columns(usize);

impl Default for Columns {
    fn default() -> Self {
        Self(1)
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::Columns;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::DirectValue;
    use crate::parse::ObjectParser;
    use crate::process::filter::error::FilterErr;
    use crate::process::filter::error::FilterErrorCode;

    impl Columns {
        pub(in crate::process::filter::predictor) fn new(value: usize) -> Self {
            Self(value)
        }
    }

    impl<'buffer> TryFrom<&'buffer DirectValue<'buffer>> for Columns {
        type Error = FilterErr;

        fn try_from(value: &'buffer DirectValue<'buffer>) -> Result<Self, Self::Error> {
            // TODO Replace with `as_usize`
            if let DirectValue::Numeric(Numeric::Integer(value)) = value {
                let value = usize::try_from(*value.deref()).map_err(|_| {
                    FilterErr::new(
                        stringify!(Columns),
                        FilterErrorCode::UnsupportedParameter(*value.deref()),
                    )
                })?;
                Ok(Self(value))
            } else {
                Err(FilterErr::new(
                    stringify!(Columns),
                    FilterErrorCode::ValueType(stringify!(Integer), value.span()),
                ))
            }
        }
    }

    impl Deref for Columns {
        type Target = usize;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
