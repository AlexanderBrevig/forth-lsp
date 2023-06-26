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
            ':' => {
                let mut dat = self.here();
                dat.value = &self.raw[self.position..self.read_position];
                Token::Colon(dat)
            }
            ';' => {
                let mut dat = self.here();
                dat.value = &self.raw[self.position..self.read_position];
                dat.end = dat.start + 1;
                Token::Semicolon(dat)
            }
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
                let begin = self.position;
                if !self.peek_char().is_whitespace() {
                    self.read_char();
                    if self.peek_char() == '\'' {
                        self.read_char();
                        let number = Data {
                            start: self.position - 2,
                            end: self.position + 1,
                            value: &self.raw[begin..(self.position + 1)],
                        };
                        Token::Number(number)
                    } else {
                        let mut ident = self.read_ident();
                        ident.start -= 1;
                        ident.value = &self.raw[begin..self.position];
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
                let mut dat = self.here();
                dat.value = "\0";
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

        self.input.next();

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

    fn read_comment_to(&mut self, to: char) -> Data<'a> {
        let start = self.position;
        let mut value = String::new();
        while self.ch != to {
            value.push(self.ch);
            self.read_char();
        }
        if to == ')' {
            value.push(self.ch);
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
        let mut value = String::new();
        while !self.ch.is_whitespace() && self.ch != '\0' {
            value.push(self.ch);
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
        Data {
            start,
            end: self.position,
            value: &self.raw[start..self.position],
        }
    }

    pub fn parse(&mut self) -> Vec<Token<'a>> {
        let mut tokens = vec![];
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
            Colon(Data::new(0, 0, ":")),
            Word(Data::new(2, 6, "add1")),
            Comment(Data::new(7, 17, "( n -- n )")),
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
            Colon(Data::new(0, 0, ":")),
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
