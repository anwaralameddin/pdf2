use crate::Byte;

// TODO Refactor the repetitive functions below

pub(crate) fn ascii_to_u16(bytes: &[Byte]) -> Option<u16> {
    let mut number = Some(0u16);
    for &byte in bytes {
        if let b'0'..=b'9' = byte {
            let digit = byte - b'0';
            number = number
                .and_then(|number| number.checked_mul(10))
                .and_then(|number| number.checked_add(digit as u16));
        } else {
            return None;
        }
    }
    number
}

pub(crate) fn ascii_to_u64(bytes: &[Byte]) -> Option<u64> {
    let mut number = Some(0u64);
    for &byte in bytes {
        if let b'0'..=b'9' = byte {
            let digit = byte - b'0';
            number = number
                .and_then(|number| number.checked_mul(10))
                .and_then(|number| number.checked_add(digit as u64));
        } else {
            return None;
        }
    }
    number
}

pub(crate) fn ascii_to_i128(bytes: &[Byte]) -> Option<i128> {
    let mut number = Some(0i128);
    let negative_sign = bytes.first().and_then(|&byte| match byte {
        b'-' => Some(true),
        b'+' => Some(false),
        _ => None,
    });

    for &byte in bytes.iter().skip(negative_sign.is_some() as usize) {
        match byte {
            b'0'..=b'9' => {
                let digit = byte - b'0';
                if let Some(true) = negative_sign {
                    number = number
                        .and_then(|number| number.checked_mul(10))
                        .and_then(|number| number.checked_sub(digit as i128));
                } else {
                    number = number
                        .and_then(|number| number.checked_mul(10))
                        .and_then(|number| number.checked_add(digit as i128));
                }
            }
            _ => return None,
        }
    }
    number
}

// ascii_to_f64 is more restrictive than str::parse::<f64> as exponent
// representation is not supported
pub(crate) fn ascii_to_f64(bytes: &[Byte]) -> Option<f64> {
    let negative_sign = bytes.first().and_then(|&byte| match byte {
        b'-' => Some(true),
        b'+' => Some(false),
        _ => None,
    });

    let mut integer = Some(0i64);
    let mut fraction = 0u64;
    let mut fraction_digits = 0;
    let mut decimal = false;
    for &byte in bytes.iter().skip(negative_sign.is_some() as usize) {
        match byte {
            b'0'..=b'9' => {
                let digit = byte - b'0';
                if decimal {
                    if let Some(value) = fraction
                        .checked_mul(10)
                        .and_then(|fraction| fraction.checked_add(digit as u64))
                    {
                        fraction = value;
                        fraction_digits += 1;
                        continue;
                    } else {
                        // Ignore the rest of the digits
                        break;
                    }
                }
                if let Some(true) = negative_sign {
                    integer = integer
                        .and_then(|integer| integer.checked_mul(10))
                        .and_then(|integer| integer.checked_sub(digit as i64));
                } else {
                    integer = integer
                        .and_then(|integer| integer.checked_mul(10))
                        .and_then(|integer| integer.checked_add(digit as i64));
                }
            }
            b'.' if !decimal => {
                decimal = true;
            }
            _ => return None,
        }
    }

    let integer = integer? as f64;
    // TODO(QUESTION): Can This division overflow?
    let fraction = fraction as f64 / 10u64.pow(fraction_digits) as f64;
    if let Some(true) = negative_sign {
        Some(integer - fraction)
    } else {
        Some(integer + fraction)
    }
}

pub(crate) fn bytes_to_u64(bytes: &[Byte]) -> Option<u64> {
    let mut number = Some(0u64);
    for &byte in bytes {
        number = number
            .and_then(|number| number.checked_shl(8))
            .and_then(|number| number.checked_add(byte as u64));
    }
    number
}

pub(crate) fn hex_val(byte: Byte) -> Option<Byte> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

// TODO Add tests
