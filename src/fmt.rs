use crate::parse::character_set::is_white_space;
use crate::Byte;
use crate::MAX_DEBUG_BYTES;

// FIXME Why not convert to printable UTF-8 characters when possible?
// This slows down the program. Apply it only for display purposes.
pub(crate) fn debug_bytes(bytes: &[Byte]) -> String {
    let mut result = String::new();
    for &byte in bytes.iter().take(MAX_DEBUG_BYTES) {
        if byte.is_ascii_graphic() || is_white_space(byte) {
            // Preserve ASCII printable and white-space characters
            result.push(char::from(byte));
        } else {
            // Hexadecimal representation of other bytes
            result.push_str(&format!("\\x{:02X}", byte));
        }
    }
    if MAX_DEBUG_BYTES < bytes.len() {
        result.push_str("...");
    }
    result
}
