/// REFERENCE: [Table 8 — Optional parameters for LZWDecode and FlateDecode
/// filters, p40]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum BitsPerComponent {
    One = 1,
    Two = 2,
    Four = 4,
    #[default]
    Eight = 8,
}

mod convert {
    use ::std::ops::Deref;

    use super::BitsPerComponent;
    use crate::object::direct::numeric::Numeric;
    use crate::object::direct::DirectValue;
    use crate::object::BorrowedBuffer;
    use crate::process::error::ProcessErr;
    use crate::process::filter::predictor::error::PredictorError;

    impl TryFrom<&DirectValue<'_>> for BitsPerComponent {
        type Error = ProcessErr;

        fn try_from(value: &DirectValue) -> Result<Self, Self::Error> {
            if let DirectValue::Numeric(Numeric::Integer(value)) = value {
                match **value {
                    1 => Ok(Self::One),
                    2 => Ok(Self::Two),
                    4 => Ok(Self::Four),
                    8 => Ok(Self::Eight),
                    _ => Err(
                        PredictorError::Unsupported(stringify!(BitsPerComponent), **value).into(),
                    ),
                }
            } else {
                Err(PredictorError::DataType(
                    stringify!(BitsPerComponent),
                    value.clone().to_owned_buffer(),
                )
                .into()) // TODO (TEMP) Avoid to_owned_buffer
            }
        }
    }

    impl Deref for BitsPerComponent {
        type Target = usize;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::One => &1,
                Self::Two => &2,
                Self::Four => &4,
                Self::Eight => &8,
            }
        }
    }
}
