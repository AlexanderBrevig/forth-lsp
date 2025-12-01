use ropey::{Rope, RopeSlice};

use super::word_at::WordAt;

pub trait WordOnOrBefore {
    fn word_on_or_before(&self, char: usize) -> RopeSlice<'_>;
}

impl WordOnOrBefore for Rope {
    fn word_on_or_before(&self, ix: usize) -> RopeSlice<'_> {
        let word_on_cursor = self.word_at(ix);
        // with helix, you typically end up with having a selected word including the previous space
        // this means we should also look for a word behind the cursor
        //TODO: make look-behind cleaner
        let word_before_cursor = if ix > 0 {
            self.word_at(ix - 1)
        } else {
            word_on_cursor
        };
        if word_on_cursor.len_chars() > 0 {
            word_on_cursor
        } else {
            word_before_cursor
        }
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::*;

    #[test]
    fn word_at_end() {
        let rope = Rope::from_str("Should find this");
        let word = rope.word_on_or_before(10);
        assert_eq!("find", word);
    }
    #[test]
    fn word_at_after() {
        let rope = Rope::from_str("Should find this");
        let word = rope.word_on_or_before(11);
        assert_eq!("find", word);
    }
}
