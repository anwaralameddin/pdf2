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
    use crate::object::direct::DirectValue;
    use crate::parse::ObjectParser;
    use crate::process::filter::error::FilterErr;
    use crate::process::filter::error::FilterErrorCode;

    impl<'buffer> TryFrom<&'buffer DirectValue<'buffer>> for Colors {
        type Error = FilterErr;

        fn try_from(value: &'buffer DirectValue<'buffer>) -> Result<Self, Self::Error> {
            if let DirectValue::Numeric(Numeric::Integer(value)) = value {
                match value.deref() {
                    1 => Ok(Self::One),
                    2 => Ok(Self::Two),
                    3 => Ok(Self::Three),
                    4 => Ok(Self::Four),
                    _ => Err(FilterErr::new(
                        stringify!(Colors),
                        FilterErrorCode::UnsupportedParameter(*value.deref()),
                    )),
                }
            } else {
                Err(FilterErr::new(
                    stringify!(Colors),
                    FilterErrorCode::ValueType(stringify!(Integer), value.span()),
                ))
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
