/// Utility functions for LSP position handling

/// Convert LSP UTF-16 character position to UTF-8 byte index
///
/// LSP uses UTF-16 code units for character positions, but Rust strings use UTF-8.
/// This function converts between the two encodings safely.
///
/// # Arguments
/// * `line` - The line of text to index into
/// * `utf16_col` - The UTF-16 code unit offset (from LSP Position.character)
///
/// # Returns
/// The corresponding UTF-8 byte index, or None if the position is out of bounds
pub fn utf16_pos_to_byte_index(line: &str, utf16_col: usize) -> Option<usize> {
    let mut utf16_count = 0usize;

    for (byte_idx, ch) in line.char_indices() {
        if utf16_count == utf16_col {
            return Some(byte_idx);
        }
        utf16_count += ch.len_utf16();
    }

    // If we've exhausted the string, return the end position if we're exactly at it
    if utf16_count == utf16_col {
        Some(line.len())
    } else {
        // Position is beyond the end or inside a surrogate pair (invalid)
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_string() {
        let line = "Hello, world!";
        assert_eq!(utf16_pos_to_byte_index(line, 0), Some(0));
        assert_eq!(utf16_pos_to_byte_index(line, 5), Some(5));
        assert_eq!(utf16_pos_to_byte_index(line, 13), Some(13));
    }

    #[test]
    fn test_unicode_string() {
        // "Hello 世界" - "世" and "界" are 3 bytes each in UTF-8, 1 UTF-16 code unit each
        let line = "Hello 世界";
        assert_eq!(utf16_pos_to_byte_index(line, 0), Some(0));
        assert_eq!(utf16_pos_to_byte_index(line, 6), Some(6)); // Start of '世'
        assert_eq!(utf16_pos_to_byte_index(line, 7), Some(9)); // Start of '界'
        assert_eq!(utf16_pos_to_byte_index(line, 8), Some(12)); // End of string
    }

    #[test]
    fn test_emoji_string() {
        // "Hi 👋" - emoji is 4 bytes in UTF-8, 2 UTF-16 code units (surrogate pair)
        let line = "Hi 👋";
        assert_eq!(utf16_pos_to_byte_index(line, 0), Some(0));
        assert_eq!(utf16_pos_to_byte_index(line, 3), Some(3)); // Start of emoji
        assert_eq!(utf16_pos_to_byte_index(line, 5), Some(7)); // After emoji (2 UTF-16 units)
    }

    #[test]
    fn test_out_of_bounds() {
        let line = "Hello";
        assert_eq!(utf16_pos_to_byte_index(line, 100), None);
    }

    #[test]
    fn test_end_of_line() {
        let line = "Hello";
        assert_eq!(utf16_pos_to_byte_index(line, 5), Some(5));
    }
}
