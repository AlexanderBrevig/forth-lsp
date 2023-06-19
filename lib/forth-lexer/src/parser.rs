use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case, take_until},
    character::complete::{char, digit1, hex_digit1},
    combinator::{all_consuming, consumed, map},
    multi::many1,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Word(String),
    String(String),
    Number(String),
    Comment(String),
    Whitespace,
    Colon,
    Semicolon,
}
// Extract a string that does not contain whitespace (space or tab).  Anything else goes.
fn not_whitespace(i: &str) -> nom::IResult<&str, &str> {
    nom::bytes::complete::is_not(" \t")(i)
}

fn backslash_comment(i: &str) -> IResult<&str, Token> {
    match map(pair(char('\\'), is_not("\n\r")), |(_, result)| result)(i) {
        Ok(comm) => Ok((comm.0, Token::Comment(comm.1.to_string()))),
        Err(err) => Err(err),
    }
}

fn inline_comment(i: &str) -> IResult<&str, Token> {
    let comment_consumed = consumed(tuple((tag("("), take_until(")"), tag(")"))));
    match map(comment_consumed, |(consumed, _)| consumed)(i) {
        Ok(comm) => Ok((comm.0, Token::Comment(comm.1.to_string()))),
        Err(err) => Err(err),
    }
}

fn string_frag<'a>(i: &'a str, frag: &'a str) -> IResult<&'a str, Token> {
    let string_consumed = consumed(nom::sequence::separated_pair(
        tag_no_case(frag),
        //TODO: handle \" in strings?
        is_not("\\\""),
        tag("\""),
    ));
    match map(string_consumed, |(consumed, _)| consumed)(i) {
        Ok(comm) => Ok((comm.0, Token::String(comm.1.to_string()))),
        Err(err) => Err(err),
    }
}

fn string(i: &str) -> IResult<&str, Token> {
    let prefixes = vec!["s\"", ".\"", "ABORT\"", "S\\\"", "C\""];
    let mut res: IResult<&str, Token> = Ok(("", Token::Whitespace));
    for pfx in prefixes {
        res = string_frag(i, pfx);
        if res.is_ok() {
            break;
        }
    }
    return res;
}

fn multispace1(i: &str) -> nom::IResult<&str, Token> {
    match nom::character::complete::multispace1(i) {
        Ok(space) => Ok((space.0, Token::Whitespace)),
        Err(err) => Err(err),
    }
}

fn number(i: &str) -> nom::IResult<&str, Token> {
    match alt((
        delimited(
            tag("'"),
            is_a("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
            tag("'"),
        ),
        preceded(is_a("&%"), digit1),
        preceded(tag("$"), hex_digit1),
        preceded(tag("0x"), hex_digit1),
        digit1,
    ))(i)
    {
        Ok(word) => Ok((word.0, Token::Number(word.1.to_string()))),
        Err(err) => Err(err),
    }
}

fn word(i: &str) -> nom::IResult<&str, Token> {
    match nom::bytes::complete::is_not(" \t\n")(i) {
        Ok(word) => Ok((word.0, Token::Word(word.1.to_string()))),
        Err(err) => Err(err),
    }
}

fn colon(i: &str) -> nom::IResult<&str, Token> {
    match nom::bytes::complete::tag(":")(i) {
        Ok(word) => Ok((word.0, Token::Colon)),
        Err(err) => Err(err),
    }
}
fn semicolon(i: &str) -> nom::IResult<&str, Token> {
    match nom::bytes::complete::tag(";")(i) {
        Ok(word) => Ok((word.0, Token::Semicolon)),
        Err(err) => Err(err),
    }
}

fn forth(i: &str) -> IResult<&str, Vec<Token>> {
    all_consuming(many1(alt((
        backslash_comment,
        string,
        inline_comment,
        colon,
        semicolon,
        number,
        word,
        multispace1,
    ))))(i)
}

pub fn lex_forth(i: &str) -> Result<Vec<Token>, Error> {
    let (_, parsed) = forth(i).map_err(|e| e.to_owned())?;
    let ret = parsed
        .into_iter()
        .filter(|x| !matches!(x, Token::Whitespace))
        .collect();
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_backslash_comment() {
        let res = backslash_comment("\\comment here\n");
        assert!(res.is_ok());
    }
    #[test]
    fn can_parse_inline_comment() {
        let res = inline_comment("( comment here )");
        assert!(res.is_ok());
    }
    #[test]
    fn can_parse_string() {
        assert!(string("s\" Hello World\"").is_ok());
        assert!(string("S\" Hello World\"").is_ok());
        assert!(string(".\" Hello World\"").is_ok());
        assert!(string("ABORT\" Hello World\"").is_ok());
        assert!(string("s\\\" Hello World\"").is_ok());
        assert!(string("C\" Hello World\"").is_ok());
    }
    #[test]
    fn can_parse_number() {
        assert!(number("12").is_ok());
        assert!(number("&12").is_ok());
        assert!(number("%0101001").is_ok());
        assert!(number("$ffaadd").is_ok());
        assert!(number("0xFE").is_ok());
        assert!(number("'c'").is_ok());
    }
    #[test]
    fn can_parse_forth() {
        let res = lex_forth(
            ": my:test ( n -- n ) 1 2 dup + ; \\my amazing comment\n.\" Hello ( maybe ) there\"",
        )
        .unwrap();
        use Token::*;
        let expected = vec![
            Colon,
            Word("my:test".into()),
            Comment("( n -- n )".into()),
            Number("1".into()),
            Number("2".into()),
            Word("dup".into()),
            Word("+".into()),
            Semicolon,
            Comment("my amazing comment".into()),
            String(".\" Hello ( maybe ) there\"".into()),
        ];
        println!("COMB: {:?}", res);
        assert_eq!(res, expected);
    }
}
