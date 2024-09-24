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
    use crate::process::error::ProcessErr;
    use crate::process::filter::predictor::error::PredictorError;

    impl Columns {
        pub(in crate::process::filter::predictor) fn new(value: usize) -> Self {
            Self(value)
        }
    }

    impl TryFrom<&DirectValue> for Columns {
        type Error = ProcessErr;

        fn try_from(value: &DirectValue) -> Result<Self, Self::Error> {
            // TODO Replace with `as_usize`
            if let DirectValue::Numeric(Numeric::Integer(value)) = value {
                let value = usize::try_from(**value)
                    .map_err(|_| PredictorError::Unsupported(stringify!(Columns), **value))?; // TODO (TEMP) Avoid overriding the error
                Ok(Self(value))
            } else {
                // TODO (TEMP) Refactor to avoid cloning
                Err(PredictorError::DataType(stringify!(Columns), value.clone()).into())
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
