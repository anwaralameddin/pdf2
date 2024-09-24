/// REFERENCE: [Table 8 — Optional parameters for LZWDecode and FlateDecode
/// filters, p40]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Colors {
    #[default]
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

mod convert {
    use ::std::ops::Deref;

    use super::Colors;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::OwnedDirectValue;
    use crate::process::error::ProcessErr;
    use crate::process::filter::predictor::error::PredictorError;

    impl TryFrom<&OwnedDirectValue> for Colors {
        type Error = ProcessErr;

        fn try_from(value: &OwnedDirectValue) -> Result<Self, Self::Error> {
            if let OwnedDirectValue::Numeric(Numeric::Integer(value)) = value {
                match **value {
                    1 => Ok(Self::One),
                    2 => Ok(Self::Two),
                    3 => Ok(Self::Three),
                    4 => Ok(Self::Four),
                    _ => Err(PredictorError::Unsupported(stringify!(Colors), **value).into()),
                }
            } else {
                Err(PredictorError::DataType(stringify!(Colors), value.clone()).into())
            }
        }
    }

    impl Deref for Colors {
        type Target = usize;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::One => &1,
                Self::Two => &2,
                Self::Three => &3,
                Self::Four => &4,
            }
        }
    }
}
