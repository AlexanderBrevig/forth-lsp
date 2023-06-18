use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_until},
    character::complete::char,
    combinator::{consumed, map, value},
    error::ParseError,
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult, InputTakeAtPosition, Parser,
};

#[derive(Debug)]
pub enum Token<'a> {
    Word(&'a str),
    String(&'a str),
    Number(&'a str),
    Comment(&'a str),
    Whitespace,
    Colon,
}

pub fn backslash_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Token<'a>, E> {
    match map(pair(char('\\'), is_not("\n\r")), |(consumed, result)| {
        result
    })(i)
    {
        Ok(comm) => Ok((comm.0, Token::Comment(comm.1))),
        Err(err) => Err(err),
    }
}

pub fn inline_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Token<'a>, E> {
    let comment_consumed = consumed(tuple((tag("("), take_until(")"), tag(")"))));
    match map(comment_consumed, |(consumed, _)| consumed)(i) {
        Ok(comm) => Ok((comm.0, Token::Comment(comm.1))),
        Err(err) => Err(err),
    }
}

pub fn string_frag<'a, E: ParseError<&'a str>>(
    i: &'a str,
    frag: &'a str,
) -> IResult<&'a str, Token<'a>, E> {
    let string_consumed = consumed(nom::sequence::separated_pair(
        tag_no_case(frag),
        //TODO: handle \" in strings?
        is_not("\""),
        tag("\""),
    ));
    match map(string_consumed, |(consumed, _)| consumed)(i) {
        Ok(comm) => Ok((comm.0, Token::String(comm.1))),
        Err(err) => Err(err),
    }
}

pub fn string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Token<'a>, E> {
    let prefixes = vec!["s\"", ".\"", "ABORT\"", "S\\\"", "C\""];
    let mut res: Result<(&str, Token), nom::Err<E>> = Ok(("", Token::Whitespace));
    for pfx in prefixes {
        res = string_frag::<E>(i, pfx);
        if res.is_ok() {
            break;
        }
    }
    return res;
}

fn multispace1<'a, E: ParseError<&'a str>>(i: &'a str) -> nom::IResult<&str, Token<'a>, E> {
    match nom::character::complete::multispace1(i) {
        Ok(space) => Ok((space.0, Token::Whitespace)),
        Err(err) => Err(err),
    }
}

fn number<'a, E: ParseError<&'a str>>(i: &'a str) -> nom::IResult<&str, Token<'a>, E> {
    //TODO: actually parse numbers
    todo!("Must parse numbers");
    match nom::bytes::complete::is_a("%x0123456789abcdefABCDEF")(i) {
        Ok(word) => Ok((word.0, Token::Number(word.1))),
        Err(err) => Err(err),
    }
}

fn word<'a, E: ParseError<&'a str>>(i: &'a str) -> nom::IResult<&str, Token<'a>, E> {
    match nom::bytes::complete::is_not(" \t\n")(i) {
        Ok(word) => Ok((word.0, Token::Word(word.1))),
        Err(err) => Err(err),
    }
}

fn colon<'a, E: ParseError<&'a str>>(i: &'a str) -> nom::IResult<&str, Token<'a>, E> {
    match nom::bytes::complete::tag(":")(i) {
        Ok(word) => Ok((word.0, Token::Colon)),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_backslash_comment() {
        let res = backslash_comment::<()>("\\comment here\n");
        assert!(res.is_ok());
    }
    #[test]
    fn can_parse_inline_comment() {
        let res = inline_comment::<()>("( comment here )");
        assert!(res.is_ok());
    }
    #[test]
    fn can_parse_string() {
        assert!(string::<()>("s\" Hello World\"").is_ok());
        assert!(string::<()>("S\" Hello World\"").is_ok());
        assert!(string::<()>(".\" Hello World\"").is_ok());
        assert!(string::<()>("ABORT\" Hello World\"").is_ok());
        assert!(string::<()>("s\\\" Hello World\"").is_ok());
        assert!(string::<()>("C\" Hello World\"").is_ok());
    }
    #[test]
    fn can_parse_forth() {
        let res = nom::combinator::all_consuming(many1(alt((
            backslash_comment::<()>,
            string::<()>,
            inline_comment::<()>,
            colon::<()>,
            number::<()>,
            word::<()>,
            multispace1::<()>,
        ))))(
            ": test ( n - n ) 1 2 dup + ; \\my amazing comment\n.\" Hello ( maybe ) there\"",
        );
        println!("COMB: {:?}", res);
        assert!(res.is_ok());
        assert!(false);
    }
}
