use forth_lexer::token::Data;
use lsp_types::Position;
pub trait ToPosition {
    fn to_position_start(&self, rope: &ropey::Rope) -> Position;
    fn to_position_end(&self, rope: &ropey::Rope) -> Position;
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
}

fn to_line_char(chix: usize, rope: &ropey::Rope) -> (u32, u32) {
    let start_line = rope.char_to_line(chix) as u32;
    let start_char = (chix - rope.line_to_char(start_line as usize)) as u32;
    (start_line, start_char)
}
