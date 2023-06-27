use ropey::{Rope, RopeSlice};

pub trait WordAt {
    fn word_at(&self, char: usize) -> RopeSlice;
}
impl WordAt for Rope {
    fn word_at(&self, chix: usize) -> RopeSlice {
        if self.char(chix).is_whitespace() {
            return self.slice(chix..chix);
        }
        let mut min = chix;
        while min > 0 && min < self.len_chars() && !self.char(min - 1).is_whitespace() {
            min -= 1;
        }
        let mut max = chix;
        while max < self.len_chars() && !self.char(max + 1).is_whitespace() {
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
}
