use std::{iter::Peekable, str::Chars};

use crate::token::{Data, Token};

pub enum LexError {}
#[derive(Debug)]
pub struct Lexer<'a> {
    position: usize,
    read_position: usize,
    token_start: usize,
    ch: char,
    raw: &'a str,
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lex = Lexer {
            position: 0,
            read_position: 0,
            token_start: 0,
            ch: '0',
            input: input.chars().peekable(),
            raw: input,
        };
        lex.read_char();

        lex
    }

    pub fn here(&self) -> Data<'a> {
        Data {
            start: self.position,
            end: self.position,
            value: "",
        }
    }

    fn current_token_data(&self) -> Data<'a> {
        let start = self.token_start;
        let end = self.position.min(self.raw.len());
        Data {
            start,
            end,
            value: &self.raw[start..end],
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, LexError> {
        self.skip_whitespace();

        self.token_start = self.position;

        let tok = match self.ch {
            '0'..='9' | '-' | '$' | '#' | '&' | '%' => self.parse_number(),
            '\'' => self.parse_quote(),
            ':' => match self.peek_char() {
                c if is_whitespace(c) => Token::Colon(self.read_ident()),
                _ => Token::Word(self.read_ident()),
            },
            ';' => match self.peek_char() {
                c if is_whitespace(c) => Token::Semicolon(self.read_ident()),
                _ => Token::Word(self.read_ident()),
            },
            '\\' => match self.peek_char() {
                c if is_whitespace(c) => Token::Comment(self.read_comment_to('\n')),
                _ => Token::Word(self.read_ident()),
            },
            '(' => match self.peek_char() {
                c if is_whitespace(c) => {
                    let comment = self.read_comment_to(')');
                    // Stack comments contain '--' to denote stack effects
                    if comment.value.contains("--") {
                        Token::StackComment(comment)
                    } else {
                        Token::Comment(comment)
                    }
                }
                _ => Token::Word(self.read_ident()),
            },
            '\0' => {
                let mut dat = self.here();
                dat.value = "\0";
                Token::Eof(dat)
            }
            _ => Token::Word(self.read_ident()),
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

    fn parse_number(&mut self) -> Token<'a> {
        let mut sign_seen = false;

        if self.ch == '-' {
            self.read_char();
            sign_seen = true;
        }

        let (prefix_len, radix) = match self.ch {
            '$' => (1, 16),
            '&' => (1, 8),
            '%' => (1, 2),
            '0' => match self.peek_char() {
                'x' | 'X' => (2, 16),
                _ => (0, 10),
            },
            // TODO: What about BASE?
            _ => (0, 10),
        };

        for _ in 0..prefix_len {
            self.read_char();
        }

        if !sign_seen && self.ch == '-' {
            self.read_char();
        }

        // Prefix followed by whitespace is Word
        if is_whitespace(self.ch) {
            return Token::Word(self.current_token_data());
        }

        while self.ch.is_digit(radix) {
            self.read_char();
        }

        // Digits followed by whitespace is Number
        if is_whitespace(self.ch) {
            Token::Number(self.current_token_data())
        } else {
            // Digits followed by neither whitespace or digit is Word
            self.read_ident();
            Token::Word(self.current_token_data())
        }
    }

    fn parse_quote(&mut self) -> Token<'a> {
        // ASSUMPTION: self.ch == '\''
        if is_whitespace(self.peek_char()) {
            return Token::Word(self.read_ident());
        }

        self.read_char(); // Read character after '\''
        self.read_char(); // and next character

        if self.ch == '\'' && is_whitespace(self.peek_char()) {
            self.read_char();
            let data = self.current_token_data();
            Token::Number(data)
        } else {
            let ident = self.read_ident();
            Token::Word(ident)
        }
    }

    fn peek_char(&mut self) -> char {
        match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        }
    }

    fn skip_whitespace(&mut self) {
        while is_whitespace(self.ch) && self.ch != '\0' {
            self.read_char();
        }
    }

    fn read_comment_to(&mut self, to: char) -> Data<'a> {
        while self.ch != to && self.ch != '\0' {
            self.read_char();
        }
        if to == ')' {
            self.read_char();
        }

        self.current_token_data()
    }

    fn read_ident(&mut self) -> Data<'a> {
        while !is_whitespace(self.ch) {
            self.read_char();
        }
        self.current_token_data()
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
                    tokens.push(tok);
                }
            }
        }
        tokens
    }
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c.is_ascii_control()
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
    fn test_parse_word_minus() {
        let mut lexer = Lexer::new("-");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 1, "-"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_word_dollar_minus() {
        let mut lexer = Lexer::new("$-");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 2, "$-"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_word_dollar() {
        let mut lexer = Lexer::new("$");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 1, "$"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_word_neg_neg_one() {
        let mut lexer = Lexer::new("--1");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 3, "--1"))];
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
    fn test_parse_negative_number_literal() {
        let mut lexer = Lexer::new("-12");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 3, "-12"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_negative_number_literal_zero() {
        let mut lexer = Lexer::new("-01");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 3, "-01"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_negative_number_oct() {
        let mut lexer = Lexer::new("&-7");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 3, "&-7"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_negative_number_bin() {
        let mut lexer = Lexer::new("-%11");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 4, "-%11"))];
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
    fn test_parse_number_bin_only_valid() {
        let mut lexer = Lexer::new("%12345");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 6, "%12345"))];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_parse_number_only_valid() {
        let mut lexer = Lexer::new("2dup");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 4, "2dup"))];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_parse_number_hex_only_valid() {
        let mut lexer = Lexer::new("$cafefun");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 8, "$cafefun"))];
        assert_eq!(tokens, expected)
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
    fn test_parse_negative_number_before_0xhex() {
        let mut lexer = Lexer::new("-0xFE");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 5, "-0xFE"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_negative_number_after_0hex() {
        let mut lexer = Lexer::new("0x-FE");
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, 5, "0x-FE"))];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_parse_minus_only_valid() {
        let mut lexer = Lexer::new("-0x-A-");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 6, "-0x-A-"))];
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
    fn test_parse_number_char_only_valid() {
        let mut lexer = Lexer::new("' '");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 1, "'")), Word(Data::new(2, 3, "'"))];
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
