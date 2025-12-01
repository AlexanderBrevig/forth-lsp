#[allow(unused_imports)]
use crate::prelude::*;

pub mod get_ix;
pub mod word_at;
pub mod word_on_or_before;

use ropey::{Rope, RopeSlice};

#[allow(dead_code)]
pub trait RopeBoundsCheck {
    fn check_char_bounds(&self, ix: usize) -> bool;
    fn check_line_bounds(&self, line: usize) -> bool;
    fn get_char_ix_safe(&self, line: u32, character: u32) -> Option<usize>;
}

impl RopeBoundsCheck for Rope {
    /// Check if a character index is within bounds
    fn check_char_bounds(&self, ix: usize) -> bool {
        ix < self.len_chars()
    }

    /// Check if a line index is within bounds
    fn check_line_bounds(&self, line: usize) -> bool {
        line < self.len_lines()
    }

    /// Safely convert line/character position to character index
    /// Returns None if the position is out of bounds
    fn get_char_ix_safe(&self, line: u32, character: u32) -> Option<usize> {
        let line = line as usize;
        if !self.check_line_bounds(line) {
            return None;
        }
        let ix = self.line_to_char(line) + character as usize;
        if !self.check_char_bounds(ix) {
            return None;
        }
        Some(ix)
    }
}

pub trait RopeSliceIsLower {
    fn is_lowercase(&self) -> bool;
}

impl<'a> RopeSliceIsLower for RopeSlice<'a> {
    fn is_lowercase(&self) -> bool {
        if let Some(chr) = self.get_char(self.len_chars() - 1) {
            chr.is_lowercase()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_char_bounds() {
        let rope = Rope::from_str("hello\nworld");
        assert!(rope.check_char_bounds(0));
        assert!(rope.check_char_bounds(5));
        assert!(rope.check_char_bounds(10));
        assert!(!rope.check_char_bounds(11));
        assert!(!rope.check_char_bounds(100));
    }

    #[test]
    fn test_check_line_bounds() {
        let rope = Rope::from_str("hello\nworld\ntest");
        assert!(rope.check_line_bounds(0));
        assert!(rope.check_line_bounds(1));
        assert!(rope.check_line_bounds(2));
        assert!(!rope.check_line_bounds(3));
        assert!(!rope.check_line_bounds(100));
    }

    #[test]
    fn test_get_char_ix_safe() {
        let rope = Rope::from_str("hello\nworld");

        // Valid positions
        assert_eq!(rope.get_char_ix_safe(0, 0), Some(0));
        assert_eq!(rope.get_char_ix_safe(0, 5), Some(5));
        assert_eq!(rope.get_char_ix_safe(1, 0), Some(6));

        // Out of bounds line
        assert_eq!(rope.get_char_ix_safe(10, 0), None);

        // Out of bounds character
        assert_eq!(rope.get_char_ix_safe(0, 100), None);
    }

    #[test]
    fn test_get_char_ix_safe_empty_rope() {
        let rope = Rope::from_str("");
        assert_eq!(rope.get_char_ix_safe(0, 0), None);
        assert_eq!(rope.get_char_ix_safe(1, 0), None);
    }
}
