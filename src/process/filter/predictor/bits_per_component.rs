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
    use crate::parse::ObjectParser;
    use crate::process::filter::error::FilterErr;
    use crate::process::filter::error::FilterErrorCode;

    impl<'buffer> TryFrom<&'buffer DirectValue<'buffer>> for BitsPerComponent {
        type Error = FilterErr;

        fn try_from(value: &'buffer DirectValue<'buffer>) -> Result<Self, Self::Error> {
            if let DirectValue::Numeric(Numeric::Integer(value)) = value {
                match value.deref() {
                    1 => Ok(Self::One),
                    2 => Ok(Self::Two),
                    4 => Ok(Self::Four),
                    8 => Ok(Self::Eight),
                    _ => Err(FilterErr::new(
                        stringify!(BitsPerComponent),
                        FilterErrorCode::UnsupportedParameter(*value.deref()),
                    )),
                }
            } else {
                Err(FilterErr::new(
                    stringify!(BitsPerComponent),
                    FilterErrorCode::ValueType(stringify!(Integer), value.span()),
                ))
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
