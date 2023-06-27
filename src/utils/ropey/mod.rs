#[allow(unused_imports)]
use crate::prelude::*;

pub mod get_ix;
pub mod word_at;
pub mod word_on_or_before;

use ropey::RopeSlice;
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
