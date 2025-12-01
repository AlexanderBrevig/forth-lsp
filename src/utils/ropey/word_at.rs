use ropey::{Rope, RopeSlice};

pub trait WordAt {
    fn word_at(&self, char: usize) -> RopeSlice<'_>;
}
impl WordAt for Rope {
    fn word_at(&self, chix: usize) -> RopeSlice<'_> {
        // Bounds check to prevent panic
        if chix >= self.len_chars() {
            return self.slice(0..0);
        }

        if self.char(chix).is_whitespace() {
            return self.slice(chix..chix);
        }

        let mut min = chix;
        while min > 0 && min < self.len_chars() {
            // Safe: we know min > 0, so min - 1 is valid
            if self.char(min - 1).is_whitespace() {
                break;
            }
            min -= 1;
        }

        let mut max = chix;
        let max_chars = self.len_chars();
        while max < max_chars.saturating_sub(1) {
            // Safe: we know max + 1 < len_chars
            if self.char(max + 1).is_whitespace() {
                break;
            }
            max += 1;
        }

        self.slice(min..(max + 1))
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::*;

    #[test]
    fn word_at_zero() {
        let rope = Rope::from_str("find the first word");
        let word = rope.word_at(0);
        assert_eq!("find", word);
    }
    #[test]
    fn word_at_center() {
        let rope = Rope::from_str("Should find this");
        let word = rope.word_at(8);
        assert_eq!("find", word);
    }
    #[test]
    fn word_at_begin() {
        let rope = Rope::from_str("Should find this");
        let word = rope.word_at(7);
        assert_eq!("find", word);
    }
    #[test]
    fn word_at_end() {
        let rope = Rope::from_str("Should find this");
        let word = rope.word_at(10);
        assert_eq!("find", word);
    }
    #[test]
    fn word_at_after() {
        let rope = Rope::from_str("Should find this");
        let word = rope.word_at(11);
        assert_eq!("", word);
    }
    #[test]
    fn word_at_single() {
        let rope = Rope::from_str("Should + find this");
        let word = rope.word_at(7);
        assert_eq!("+", word);
    }

    #[test]
    fn word_at_out_of_bounds() {
        let rope = Rope::from_str("test");
        // Should return empty slice instead of panicking
        let word = rope.word_at(100);
        assert_eq!("", word);
    }

    #[test]
    fn word_at_empty_rope() {
        let rope = Rope::from_str("");
        // Should return empty slice instead of panicking
        let word = rope.word_at(0);
        assert_eq!("", word);
    }

    #[test]
    fn word_at_last_char() {
        let rope = Rope::from_str("test");
        // word_at on last character should work
        let word = rope.word_at(3);
        assert_eq!("test", word);
    }
}
