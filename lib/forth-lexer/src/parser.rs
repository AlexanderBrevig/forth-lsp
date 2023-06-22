use std::{iter::Peekable, str::Chars};

use nom::AsChar;

use crate::token::{Data, Token};

pub enum LexError {}
#[derive(Debug)]
pub struct Lexer<'a> {
    line: usize,
    col: isize,
    position: usize,
    read_position: usize,
    ch: char,
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lex = Lexer {
            col: 0,
            line: 0,
            position: 0,
            read_position: 0,
            ch: '0',
            input: input.chars().peekable(),
        };
        lex.read_char();
        lex.col = 0;

        lex
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.read_position = 0;
        self.col = 0;
        self.line = 0;
        self.ch = '\0';
    }

    pub fn here<T>(&self) -> Data<T>
    where
        T: Default,
    {
        Data {
            line: self.line,
            col: self.col,
            start: self.position,
            end: self.position,
            value: T::default(),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let tok = match self.ch {
            ':' => {
                let mut dat = self.here::<char>();
                dat.value = ':';
                Token::Colon(dat)
            }
            ';' => {
                let mut dat = self.here::<char>();
                dat.value = ';';
                dat.end = dat.start + 1;
                Token::Semicolon(dat)
            }
            //TODO: comments
            //TODO: strings
            '%' => {
                if self.peek_char().is_digit(2) {
                    let ident = self.read_number();
                    Token::Number(ident)
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '&' => {
                if self.peek_char() == 'x' || self.peek_char().is_digit(8) {
                    let ident = self.read_number();
                    Token::Number(ident)
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '$' => {
                if self.peek_char().is_hex_digit() {
                    let ident = self.read_number();
                    Token::Number(ident)
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '\'' => {
                if !self.peek_char().is_whitespace() {
                    self.read_char();
                    if self.peek_char() == '\'' {
                        let num = self.ch;
                        self.read_char();
                        let number = Data::<String> {
                            line: self.line,
                            col: self.col - 2,
                            start: self.position - 2,
                            end: self.position + 1,
                            value: format!("'{}'", num),
                        };
                        Token::Number(number)
                    } else {
                        let mut ident = self.read_ident();
                        ident.col -= 1;
                        ident.start -= 1;
                        ident.value = format!("{}{}", "'", ident.value);
                        Token::Word(ident)
                    }
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '0' => {
                if self.peek_char() == 'x' || self.peek_char().is_hex_digit() {
                    let ident = self.read_number();
                    Token::Number(ident)
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
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
                    Token::Comment(comment)
                } else {
                    let ident = self.read_ident();
                    Token::Word(ident)
                }
            }
            '\0' => {
                let mut dat = self.here::<char>();
                dat.value = '\0';
                Token::Eof(dat)
            }
            _ => {
                let ident = self.read_ident();
                Token::Word(ident)
            }
        };

        self.read_char();
        Ok(tok)
    }

    fn read_char(&mut self) {
        self.ch = match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        };

        let previous = self.ch;
        self.input.next();
        match previous {
            '\n' => {
                self.line += 1;
                self.col = -1;
            }
            _ => {
                self.col += 1;
            }
        }

        self.position = self.read_position;
        self.read_position += 1;
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

    fn read_comment_to(&mut self, to: char) -> Data<String> {
        let pos = self.position;
        let line = self.line;
        let col = self.col;
        let mut value = String::new();
        while self.ch != to {
            value.push(self.ch);
            self.read_char();
        }
        if to == ')' {
            value.push(self.ch);
            self.read_char();
        }

        Data::<String> {
            line,
            col,
            start: pos,
            end: self.position,
            value,
        }
    }

    fn read_ident(&mut self) -> Data<String> {
        let pos = self.position;
        let line = self.line;
        let col = self.col;
        let mut value = String::new();
        while !self.ch.is_whitespace() && self.ch != '\0' {
            value.push(self.ch);
            self.read_char();
        }
        Data::<String> {
            line,
            col,
            start: pos,
            end: self.position,
            value,
        }
    }

    fn read_number(&mut self) -> Data<String> {
        let pos = self.position;
        let line = self.line;
        let col = self.col;
        let mut value = String::new();
        //TODO: parse legal forth numbers
        while self.ch.is_hex_digit()
            || self.ch == '_'
            || self.ch == '&'
            || self.ch == '%'
            || self.ch == 'x'
            || self.ch == '$'
        {
            value.push(self.ch);
            self.read_char();
        }
        Data::<String> {
            line,
            col,
            start: pos,
            end: self.position,
            value,
        }
    }

    pub fn parse(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while let Ok(tok) = self.next_token() {
            match tok {
                Token::Eof(_) => {
                    break;
                }
                _ => {
                    tokens.push(tok);
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

    fn dat<T>(line: usize, col: isize, start: usize, end: usize, value: T) -> Data<T> {
        Data::<T> {
            line,
            col,
            start,
            end,
            value,
        }
    }

    #[test]
    fn test_parse_proper_def() {
        let mut lexer = Lexer::new(": add1 ( n -- n )\n  1 + \\ adds one\n;");
        let tokens = lexer.parse();
        let expected = vec![
            Colon(dat::<char>(0, 0, 0, 0, ':')),
            Word(dat::<String>(0, 2, 2, 6, "add1".into())),
            Comment(dat::<String>(0, 7, 7, 17, "( n -- n )".into())),
            Number(dat::<String>(1, 2, 20, 21, "1".into())),
            Word(dat::<String>(1, 4, 22, 23, "+".into())),
            Comment(dat::<String>(1, 6, 24, 34, "\\ adds one".into())),
            Semicolon(dat::<char>(2, 0, 35, 36, ';')),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_simple_def() {
        let mut lexer = Lexer::new(": add1 1 + ;");
        let tokens = lexer.parse();
        let expected = vec![
            Colon(dat::<char>(0, 0, 0, 0, ':')),
            Word(dat::<String>(0, 2, 2, 6, "add1".into())),
            Number(dat::<String>(0, 7, 7, 8, "1".into())),
            Word(dat::<String>(0, 9, 9, 10, "+".into())),
            Semicolon(dat::<char>(0, 11, 11, 12, ';')),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_words_and_comments() {
        let mut lexer = Lexer::new("word \\ this is a comment\nword2 ( and this ) word3");
        let tokens = lexer.parse();
        let expected = vec![
            Word(dat::<String>(0, 0, 0, 4, "word".into())),
            Comment(dat::<String>(0, 5, 5, 24, "\\ this is a comment".into())),
            Word(dat::<String>(1, 0, 25, 30, "word2".into())),
            Comment(dat::<String>(1, 6, 31, 43, "( and this )".into())),
            Word(dat::<String>(1, 19, 44, 49, "word3".into())),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_words_on_lines() {
        let mut lexer = Lexer::new("some\nwords here\0");
        let tokens = lexer.parse();
        let expected = vec![
            Word(dat::<String>(0, 0, 0, 4, "some".into())),
            Word(dat::<String>(1, 0, 5, 10, "words".into())),
            Word(dat::<String>(1, 6, 11, 15, "here".into())),
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_literal() {
        let mut lexer = Lexer::new("12");
        let tokens = lexer.parse();
        let expected = vec![Number(dat::<String>(0, 0, 0, 2, "12".into()))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_oct() {
        let mut lexer = Lexer::new("&12");
        let tokens = lexer.parse();
        let expected = vec![Number(dat::<String>(0, 0, 0, 3, "&12".into()))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_bin() {
        let mut lexer = Lexer::new("%0100101");
        let tokens = lexer.parse();
        let expected = vec![Number(dat::<String>(0, 0, 0, 8, "%0100101".into()))];
        assert_eq!(tokens, expected);
    }

    #[test]
    #[ignore]
    fn test_parse_number_bin_only_valid() {
        //TODO: but ill formed will also parse
        //      %12345 is not a binary number
        let mut lexer = Lexer::new("%12345");
        let tokens = lexer.parse();
        let expected = vec![Word(dat::<String>(0, 0, 0, 6, "%12345".into()))];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_parse_number_hex() {
        let mut lexer = Lexer::new("$FfAaDd");
        let tokens = lexer.parse();
        let expected = vec![Number(dat::<String>(0, 0, 0, 7, "$FfAaDd".into()))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_0xhex() {
        let mut lexer = Lexer::new("0xFE");
        let tokens = lexer.parse();
        let expected = vec![Number(dat::<String>(0, 0, 0, 4, "0xFE".into()))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_char() {
        let mut lexer = Lexer::new("'c'");
        let tokens = lexer.parse();
        let expected = vec![Number(dat::<String>(0, 0, 0, 3, "'c'".into()))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_number_word() {
        let mut lexer = Lexer::new("word");
        let tokens = lexer.parse();
        let expected = vec![Word(dat::<String>(0, 0, 0, 4, "word".into()))];
        assert_eq!(tokens, expected)
    }
}
