use std::{fmt::Display, ops::RangeBounds};

#[derive(Debug, PartialEq, Default, Copy, Clone)]
pub struct Data<'a> {
    pub start: usize,
    pub end: usize,
    pub value: &'a str,
}

impl<'a> Data<'a> {
    pub fn new(start: usize, end: usize, value: &'a str) -> Data {
        Data { start, end, value }
    }
}

impl<'a> RangeBounds<usize> for &Data<'a> {
    fn start_bound(&self) -> std::ops::Bound<&usize> {
        std::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&usize> {
        std::ops::Bound::Excluded(&self.end)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Illegal(Data<'a>),
    Eof(Data<'a>),
    Colon(Data<'a>),
    Semicolon(Data<'a>),
    Word(Data<'a>),
    Number(Data<'a>),
    Comment(Data<'a>),
    StackComment(Data<'a>),
}

impl<'a> Token<'a> {
    pub fn get_data(&self) -> &Data<'a> {
        match self {
            Token::Illegal(dat) => dat,
            Token::Eof(dat) => dat,
            Token::Colon(dat) => dat,
            Token::Semicolon(dat) => dat,
            Token::Word(dat) => dat,
            Token::Number(dat) => dat,
            Token::Comment(dat) => dat,
            Token::StackComment(dat) => dat,
        }
    }
}

impl<'a> Display for Token<'a> {
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

impl<'a> From<Data<'a>> for Token<'a> {
    fn from(value: Data<'a>) -> Self {
        match value.value {
            ";" => Self::Semicolon(value),
            ":" => Self::Colon(value),
            "\0" => Self::Eof(value),
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
