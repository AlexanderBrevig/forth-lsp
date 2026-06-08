use std::{iter::Peekable, str::Chars};

use nom::AsChar;

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
            '0'..='9' | '-' | '$' | '#' | '&' | '%' | '_' | '.' => self.parse_number(),
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
        self.read_position += self.ch.len();
    }

    fn read_digits(&mut self, radix: u32) -> usize {
        let mut count = 0;
        while self.ch.is_digit(radix) || self.ch == '_' {
            self.read_char();
            count += 1;
        }
        count
    }

    fn parse_number(&mut self) -> Token<'a> {
        let mut accept_sign = true;

        if self.ch == '-' {
            self.read_char();
            accept_sign = false;
        }

        let (prefix_len, radix) = match self.ch {
            '$' => (1, 16),
            '%' => (1, 2),
            '#' => (1, 10),
            '&' => (1, 10),
            '0' => match self.peek_char() {
                'x' | 'X' => (2, 16),
                _ => (0, 10),
            },
            _ => (0, 10),
        };

        for _ in 0..prefix_len {
            self.read_char();
        }

        if accept_sign && self.ch == '-' {
            self.read_char();
        }

        let mut digit_count = self.read_digits(radix);

        if self.ch == '.' {
            self.read_char();
            digit_count += self.read_digits(radix);
        }

        // Digits followed by whitespace is Number
        if digit_count > 0 && is_whitespace(self.ch) {
            Token::Number(self.current_token_data())
        } else {
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
    fn test_word_multi_byte_utf8() {
        let mut lexer = Lexer::new("👻");
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, 4, "👻"))];
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

    const PREFIXES: [&str; 6] = ["$", "#", "&", "%", "0x", ""];

    const DIGITS: [&str; 6] = [
        "0123456789abcdef",
        "012345789",
        "012345679",
        "01",
        "0123456789ABCDEF",
        "012345689",
    ];

    const NON_DIGITS: [&str; 6] = [
        "0123456789abcdefg",
        "012345789a",
        "012345689a",
        "012",
        "0123456789ABCDEFG",
        "012345689a",
    ];

    fn should_parse_number(s: String) {
        let mut lexer = Lexer::new(&s);
        let tokens = lexer.parse();
        let expected = vec![Number(Data::new(0, s.len(), &s))];
        assert_eq!(tokens, expected);
    }

    fn should_parse_word(s: String) {
        let mut lexer = Lexer::new(&s);
        let tokens = lexer.parse();
        let expected = vec![Word(Data::new(0, s.len(), &s))];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_parse_numbers() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            should_parse_number(format!("{prefix}{digits}"));
        }
    }

    #[test]
    fn test_parse_numbers_sign() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            // Sign in prefix
            should_parse_number(format!("-{prefix}{digits}"));
            should_parse_number(format!("{prefix}-{digits}"));
        }
    }

    #[test]
    fn test_parse_numbers_underscore() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            // Underscore after prefix
            should_parse_number(format!("{prefix}_{digits}_{digits}"));
            should_parse_number(format!("{prefix}_")); // Zero
        }
    }

    #[test]
    fn test_parse_numbers_decimal_point() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            // Decimal point after prefix
            should_parse_number(format!("{prefix}.{digits}"));
            should_parse_number(format!("{prefix}{digits}."));
            should_parse_number(format!("{prefix}{digits}.{digits}"));
        }
    }

    #[test]
    fn test_parse_numbers_only_valid() {
        for (prefix, bad_digits) in PREFIXES.iter().zip(NON_DIGITS.iter()) {
            // Non-digits
            should_parse_word(format!("{prefix}{bad_digits}"));
        }
    }

    #[test]
    fn test_parse_numbers_prefix_only_valid() {
        for prefix in PREFIXES {
            // Prefix without digits
            if !prefix.is_empty() {
                should_parse_word(prefix.to_string());
            }
            should_parse_word(format!("{prefix}-"));
            should_parse_word(format!("-{prefix}"));
            should_parse_word(format!("{prefix}-"));
        }
    }

    #[test]
    fn test_parse_numbers_sign_only_valid() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            // Too many signs
            should_parse_word(format!("--{prefix}{digits}"));
            should_parse_word(format!("-{prefix}-{digits}"));
            should_parse_word(format!("{prefix}-{digits}-{digits}"));
            should_parse_word(format!("{prefix}--{digits}"));
        }
    }

    #[test]
    fn test_parse_numbers_underscore_only_valid() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            // Underscore in prefix
            if !prefix.is_empty() {
                should_parse_word(format!("_{prefix}{digits}"));
                should_parse_word(format!("{prefix}_-{digits}"));
            }
        }
    }

    #[test]
    fn test_parse_numbers_decimal_point_only_valid() {
        for (prefix, digits) in PREFIXES.iter().zip(DIGITS.iter()) {
            // Decimal point before prefix
            if !prefix.is_empty() {
                should_parse_word(format!(".{prefix}{digits}"));
            }

            // Too many decimal points
            should_parse_word(format!("{prefix}{digits}.{digits}.{digits}"));
            should_parse_word(format!("{prefix}.{digits}.{digits}"));

            // Decimal point without digits
            should_parse_word(format!("{prefix}."));
        }
    }
}
