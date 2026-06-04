use forth_lexer::token::Data;
use lsp_types::{Position, Range};

pub trait ToPosition {
    fn to_position_start(&self, rope: &ropey::Rope) -> Position;
    fn to_position_end(&self, rope: &ropey::Rope) -> Position;
    fn to_range(&self, rope: &ropey::Rope) -> Range;
}

impl<'a> ToPosition for Data<'a> {
    fn to_position_start(&self, rope: &ropey::Rope) -> Position {
        let (start_line, start_char) = to_line_char(self.start, rope);
        Position {
            line: start_line,
            character: start_char,
        }
    }
    fn to_position_end(&self, rope: &ropey::Rope) -> Position {
        let (start_line, start_char) = to_line_char(self.end, rope);
        Position {
            line: start_line,
            character: start_char,
        }
    }

    fn to_range(&self, rope: &ropey::Rope) -> Range {
        Range {
            start: self.to_position_start(rope),
            end: self.to_position_end(rope),
        }
    }
}

pub fn to_line_char(byte_idx: usize, rope: &ropey::Rope) -> (u32, u32) {
    let chix = rope.byte_to_char(byte_idx);
    let start_line = rope.char_to_line(chix) as u32;
    let start_char = (chix - rope.line_to_char(start_line as usize)) as u32;
    (start_line, start_char)
}

/// Create a Range from two Data objects (begin and end).
/// Useful when you need to create a range spanning multiple tokens.
pub fn data_range_from_to<'a>(begin: &Data<'a>, end: &Data<'a>, rope: &ropey::Rope) -> Range {
    Range {
        start: begin.to_position_start(rope),
        end: end.to_position_end(rope),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forth_lexer::token::Data;
    use lsp_types::Position;
    use ropey::Rope;

    #[test]
    fn test_to_position_start_single_line() {
        let rope = Rope::from_str("hello world");
        let data = Data::new(0, 5, "hello");
        let pos = data.to_position_start(&rope);
        assert_eq!(
            pos,
            Position {
                line: 0,
                character: 0
            }
        );
    }

    #[test]
    fn test_to_position_end_single_line() {
        let rope = Rope::from_str("hello world");
        let data = Data::new(0, 5, "hello");
        let pos = data.to_position_end(&rope);
        assert_eq!(
            pos,
            Position {
                line: 0,
                character: 5
            }
        );
    }

    #[test]
    fn test_to_position_start_multiline() {
        let rope = Rope::from_str("line one\nline two\nline three");
        // "line two" starts at char index 9 and ends at 17
        let data = Data::new(9, 17, "line two");
        let pos = data.to_position_start(&rope);
        assert_eq!(
            pos,
            Position {
                line: 1,
                character: 0
            }
        );
    }

    #[test]
    fn test_to_position_end_multiline() {
        let rope = Rope::from_str("line one\nline two\nline three");
        // "line two" starts at char index 9 and ends at 17
        let data = Data::new(9, 17, "line two");
        let pos = data.to_position_end(&rope);
        assert_eq!(
            pos,
            Position {
                line: 1,
                character: 8
            }
        );
    }

    #[test]
    fn test_to_position_forth_definition() {
        let rope = Rope::from_str(": add1 ( n -- n )\n  1 + \\ adds one\n;");
        // "add1" is at chars 2-6
        let data = Data::new(2, 6, "add1");
        let start = data.to_position_start(&rope);
        let end = data.to_position_end(&rope);
        assert_eq!(
            start,
            Position {
                line: 0,
                character: 2
            }
        );
        assert_eq!(
            end,
            Position {
                line: 0,
                character: 6
            }
        );
    }

    #[test]
    fn test_to_position_at_line_boundary() {
        let rope = Rope::from_str("first\nsecond");
        // "second" starts at char 6 (after "first\n")
        let data = Data::new(6, 12, "second");
        let pos = data.to_position_start(&rope);
        assert_eq!(
            pos,
            Position {
                line: 1,
                character: 0
            }
        );
    }

    #[test]
    fn test_to_range() {
        let rope = Rope::from_str("hello world");
        let data = Data::new(0, 5, "hello");
        let range = data.to_range(&rope);
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 0,
                character: 5
            }
        );
    }

    #[test]
    fn test_to_range_multiline() {
        let rope = Rope::from_str("line one\nline two");
        let data = Data::new(9, 17, "line two");
        let range = data.to_range(&rope);
        assert_eq!(
            range.start,
            Position {
                line: 1,
                character: 0
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 1,
                character: 8
            }
        );
    }

    #[test]
    fn test_data_range_from_to() {
        let rope = Rope::from_str(": add1 1 + ;");
        let colon = Data::new(0, 1, ":");
        let semicolon = Data::new(11, 12, ";");
        let range = data_range_from_to(&colon, &semicolon, &rope);
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 0,
                character: 12
            }
        );
    }

    #[test]
    fn test_data_range_from_to_multiline() {
        let rope = Rope::from_str(": add1\n  1 +\n;");
        let colon = Data::new(0, 1, ":");
        let semicolon = Data::new(13, 14, ";");
        let range = data_range_from_to(&colon, &semicolon, &rope);
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 2,
                character: 1
            }
        );
    }

    #[test]
    fn test_to_position_with_multibyte_utf8() {
        // The lexer emits BYTE offsets in Data; positions must still be in chars.
        // Source: `\ è\n: x ;` — the `è` is 2 bytes / 1 char.
        let src = "\\ è\n: x ;";
        let rope = Rope::from_str(src);
        // Byte offset of `x`: backslash(1)+space(1)+è(2)+nl(1)+colon(1)+space(1) = 7
        let x_byte_start = src.find('x').unwrap();
        let data = Data::new(x_byte_start, x_byte_start + 1, "x");
        let pos = data.to_position_start(&rope);
        assert_eq!(
            pos,
            Position {
                line: 1,
                character: 2
            }
        );
    }
}
