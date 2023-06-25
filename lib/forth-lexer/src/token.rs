use std::{fmt::Display, ops::RangeBounds};

#[derive(Debug, PartialEq, Default, Copy, Clone)]
pub struct Data<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
}

impl<T> Data<T> {
    pub fn new(start: usize, end: usize, value: T) -> Data<T> {
        Data::<T> { start, end, value }
    }
}

impl<T> RangeBounds<usize> for &Data<T> {
    fn start_bound(&self) -> std::ops::Bound<&usize> {
        std::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&usize> {
        std::ops::Bound::Excluded(&self.end)
    }
}

#[derive(Debug, PartialEq, Clone)]
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
        if value.value.chars().all(|b| b.is_ascii_digit()) {
            Self::Number(value)
        } else {
            Self::Word(value)
        }
    }
}
