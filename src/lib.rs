// TODO Remove this attribute when the crate main functionality is implemented.
#![allow(dead_code)]

mod convert;
mod fmt;
mod object;
mod parse;
pub mod pdf;
mod process;
mod xref;

use ::std::num::NonZeroU64;

pub use self::pdf::PdfBuilder;

// Limit the size of the decoded stream to 1 GiB.
const DECODED_LIMIT: usize = 1 << 30;

/// Although u32 would suffice for most cases, allowing for ~.5 GiB files,
/// [7.5.4 Cross-reference table, p56] only restricts bytes offsets to 10
/// digits, allowing for ~9.3 GiB files. Hence, it can be represented as a u64.
type Offset = u64;
/// REFERENCE: [3.33 indirect object, p10]
/// - Object numbers are positive integer objects.
/// - The object number cannot exceed allowed offsets.
/// Hence, it can be represented as a NonZeroU64.
type ObjectNumber = NonZeroU64;
type ObjectNumberOrZero = u64;
/// REFERENCE:
/// - [3.33 indirect object, p10]
/// - [7.5.4 Cross-reference table, p56-57]
/// - Generation numbers are non-negative integer objects.
/// - They are restricted to 5 digits.
/// - They are allowed the maximum value of 65,535.
/// Hence, it can be represented as a u16.
type GenerationNumber = u16;
type SectionNumber = GenerationNumber;
type IndexNumber = u64;
/// REFERENCE: [4.7 byte, p7]
type Byte = u8;
type Bytes = Box<[Byte]>;

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! assert_err_eq {
        ($parse_result:expr, $expected_error:expr) => {
            assert_eq!($parse_result, Err($expected_error.into()));
        };
    }
}
