use std::{iter::Peekable, str::Chars};

use nom::AsChar;

use crate::token::{Data, Token};

pub enum LexError {}
#[derive(Debug)]
pub struct Lexer<'a> {
    position: usize,
    read_position: usize,
    ch: char,
    raw: &'a str,
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lex = Lexer {
            position: 0,
            read_position: 0,
            ch: '0',
            input: input.chars().peekable(),
            raw: input,
        };
        lex.read_char();

        lex
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.read_position = 0;
        self.ch = '\0';
    }

    pub fn here(&self) -> Data<'a> {
        Data {
            start: self.position,
            end: self.position,
            value: "",
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, LexError> {
        self.skip_whitespace();

        let tok = match self.ch {
            ':' => Token::Colon(self.read_single_char_token()),
            ';' => Token::Semicolon(self.read_single_char_token()),
            '%' => self.try_parse_number_with_prefix(|c| c.is_digit(2)),
            '&' => self.try_parse_number_with_prefix(|c| c == 'x' || c.is_digit(8)),
            '$' => self.try_parse_number_with_prefix(|c| c.is_hex_digit()),
            '\'' => self.parse_quote_or_word(),
            '0' => self.try_parse_number_with_prefix(|c| c == 'x' || c.is_hex_digit()),
            '0'..='9' => {
                let ident = self.read_number();
                Token::Number(ident)
            }
            '\\' => {
                if self.peek_char().is_whitespace() {
                    let comment = self.read_comment_to('\n');
                    Token::Comment(comment)
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '(' => {
                if self.peek_char().is_whitespace() {
                    let comment = self.read_comment_to(')');
                    // Stack comments contain '--' to denote stack effects
                    if comment.value.contains("--") {
                        Token::StackComment(comment)
                    } else {
                        Token::Comment(comment)
                    }
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '\0' => {
                let mut dat = self.here();
                dat.value = "\0";
                self.read_char();
                Token::Eof(dat)
            }
            _ => {
                let ident = self.read_ident();
                Token::Word(ident)
            }
        };

        Ok(tok)
    }

    fn read_char(&mut self) {
        self.ch = match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        };

        self.input.next();

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn try_parse_number_with_prefix(&mut self, validator: fn(char) -> bool) -> Token<'a> {
        if validator(self.peek_char()) {
            Token::Number(self.read_number())
        } else {
            Token::Word(self.read_ident())
        }
    }

    fn parse_quote_or_word(&mut self) -> Token<'a> {
        let begin = self.position;
        let next = self.peek_char();

        if next.is_whitespace() {
            return Token::Word(self.read_ident());
        }

        self.read_char(); // consume character after quote

        if self.peek_char() == '\'' {
            // Character literal like 'A'
            self.read_char(); // consume closing quote
            let number = Data {
                start: begin,
                end: self.position + 1,
                value: &self.raw[begin..(self.position + 1)],
            };
            self.read_char(); // move past
            return Token::Number(number);
        }

        // Quoted word
        let mut word = self.read_ident();
        word.start = begin;
        word.value = &self.raw[begin..word.end];
        Token::Word(word)
    }

    fn read_single_char_token(&mut self) -> Data<'a> {
        let start = self.position;
        self.read_char();
        Data {
            start,
            end: start + 1,
            value: &self.raw[start..start + 1],
        }
    }

    fn peek_char(&mut self) -> char {
        match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn read_comment_to(&mut self, to: char) -> Data<'a> {
        let start = self.position;
        while self.ch != to && self.ch != '\0' {
            self.read_char();
        }
        if to == ')' {
            self.read_char();
        }

        Data {
            start,
            end: self.position,
            value: &self.raw[start..self.position],
        }
    }

    fn read_ident(&mut self) -> Data<'a> {
        let start = self.position;
        while !self.ch.is_whitespace() && self.ch != '\0' {
            self.read_char();
        }
        Data {
            start,
            end: self.position,
            value: &self.raw[start..self.position],
        }
    }

    fn read_number(&mut self) -> Data<'a> {
        let start = self.position;
        //TODO: parse legal forth numbers
        while self.ch.is_hex_digit()
            || self.ch == '_'
            || self.ch == '&'
            || self.ch == '%'
            || self.ch == 'x'
            || self.ch == '$'
        {
            self.read_char();
        }
        Data {
            start,
            end: self.position,
            value: &self.raw[start..self.position],
        }
    }

    pub fn parse(&mut self) -> Vec<Token<'a>> {
        let mut tokens = vec![];
        #[allow(irrefutable_let_patterns)]
        while let Ok(tok) = self.next_token() {
            match tok {
                Token::Eof(_) => {
                    break;
                }
                _ => {
                    tokens.push(tok.clone());
                }
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn test_parse_proper_def() {
        let mut lexer = Lexer::new(": add1 ( n -- n )\n  1 + \\ adds one\n;");
        let tokens = lexer.parse();
        let expected = vec![
            Colon(Data::new(0, 1, ":")),
            Word(Data::new(2, 6, "add1")),
            StackComment(Data::new(7, 17, "( n -- n )")),
            Number(Data::new(20, 21, "1")),
            Word(Data::new(22, 23, "+")),
            Comment(Data::new(24, 34, "\\ adds one")),
            Semicolon(Data::new(35, 36, ";")),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_simple_def() {
        let mut lexer = Lexer::new(": add1 1 + ;");
        let tokens = lexer.parse();
        let expected = vec![
            Colon(Data::new(0, 1, ":")),
            Word(Data::new(2, 6, "add1")),
            Number(Data::new(7, 8, "1")),
            Word(Data::new(9, 10, "+")),
            Semicolon(Data::new(11, 12, ";")),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_words_and_comments() {
        let mut lexer = Lexer::new("word \\ this is a comment\nword2 ( and this ) word3");
        let tokens = lexer.parse();
        let expected = vec![
            Word(Data::new(0, 4, "word")),
            Comment(Data::new(5, 24, "\\ this is a comment")),
            Word(Data::new(25, 30, "word2")),
            Comment(Data::new(31, 43, "( and this )")),
            Word(Data::new(44, 49, "word3")),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_words_on_lines() {
        let mut lexer = Lexer::new("some\nwords here\0");
        let tokens = lexer.parse();
        let expected = vec![
            Word(Data::new(0, 4, "some")),
            Word(Data::new(5, 10, "words")),
            Word(Data::new(11, 15, "here")),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_literal() {
        let mut lexer = Lexer::new("12");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 2, "12"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_oct() {
        let mut lexer = Lexer::new("&12");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 3, "&12"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_bin() {
        let mut lexer = Lexer::new("%0100101");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 8, "%0100101"))];
        assert_eq!(tokens, expected);
    }

    #[test]
    #[ignore]
    fn test_parse_number_bin_only_valid() {
        //TODO: but ill formed will also parse
        //      %12345 is not a binary number
        let mut lexer = Lexer::new("%12345");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 6, "%12345"))];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_parse_number_hex() {
        let mut lexer = Lexer::new("$FfAaDd");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 7, "$FfAaDd"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_0xhex() {
        let mut lexer = Lexer::new("0xFE");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 4, "0xFE"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_char() {
        let mut lexer = Lexer::new("'c'");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 3, "'c'"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_stack_comment() {
        let mut lexer = Lexer::new("( n1 n2 -- n3 )");
        let tokens = lexer.parse();
        let expected = vec![StackComment(Data::new(0, 15, "( n1 n2 -- n3 )"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_regular_comment() {
        let mut lexer = Lexer::new("( this is just a comment )");
        let tokens = lexer.parse();
        let expected = vec![Comment(Data::new(0, 26, "( this is just a comment )"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_stack_comment_complex() {
        let mut lexer = Lexer::new("( addr len -- addr' len' flag )");
        let tokens = lexer.parse();
        let expected = vec![StackComment(Data::new(
            0,
            31,
            "( addr len -- addr' len' flag )",
        ))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_line_comment() {
        let mut lexer = Lexer::new("\\ this is a line comment");
        let tokens = lexer.parse();
        let expected = vec![Comment(Data::new(0, 24, "\\ this is a line comment"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_word() {
        let mut lexer = Lexer::new("word");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 4, "word"))];
        assert_eq!(tokens, expected)
    }

    #[cfg(feature = "ropey")]
    #[test]
    fn test_to_ropey() {
        let progn = "word1 word2 word3";
        let rope = ropey::Rope::from_str(progn);
        let mut lexer = Lexer::new(progn);
        let tokens = lexer.parse();
        let word2 = if let Some(Token::Word(word)) = tokens.get(1) {
            word.to_owned()
        } else {
            Data::default()
        };
        let x = rope.slice(&word2);
        assert_eq!("word2", word2.value);
        assert_eq!(word2.value, x);
    }
}
