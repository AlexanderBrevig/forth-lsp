use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Data<T> {
    pub line: usize,
    pub col: isize,
    pub start: usize,
    pub end: usize,
    pub value: T,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Illegal(Data<char>),
    Eof(Data<char>),
    Colon(Data<char>),
    Semicolon(Data<char>),
    Word(Data<String>),
    Number(Data<String>),
    Comment(Data<String>),
    StackComment(Data<String>),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Illegal(_) => write!(f, ""),
            Token::Eof(_) => write!(f, "\0"),
            Token::Word(value)
            | Token::Number(value)
            | Token::StackComment(value)
            | Token::Comment(value) => write!(f, "{value:?}"),
            Token::Colon(_) => write!(f, ":"),
            Token::Semicolon(_) => write!(f, ";"),
        }
    }
}

impl From<Data<char>> for Token {
    fn from(ch: Data<char>) -> Self {
        match ch.value {
            ';' => Self::Semicolon(ch),
            ':' => Self::Colon(ch),
            '\0' => Self::Eof(ch),
            _ => Self::Illegal(ch),
        }
    }
}

impl From<Data<String>> for Token {
    fn from(value: Data<String>) -> Self {
        match value.value.as_str() {
            _ => {
                if value.value.chars().all(|b| b.is_ascii_digit()) {
                    Self::Number(value)
                } else {
                    Self::Word(value)
                }
            }
        }
    }
}
