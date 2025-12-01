use crate::utils::find_variant_sublists_from_to::FindVariantSublistsFromTo;
use forth_lexer::token::{Data, Token};
use std::mem::discriminant;

/// Helper to find all colon definitions (`: word ... ;`) in a token stream.
/// This is a common pattern used across multiple handlers.
pub fn find_colon_definitions<'a>(tokens: &'a Vec<Token<'a>>) -> Vec<&'a [Token<'a>]> {
    tokens.find_variant_sublists_from_to(
        discriminant(&Token::Colon(Data::default())),
        discriminant(&Token::Semicolon(Data::default())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_single_definition() {
        let tokens = vec![
            Token::Colon(Data::default()),
            Token::Word(Data::new(0, 0, "")),
            Token::Semicolon(Data::default()),
        ];
        let defs = find_colon_definitions(&tokens);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].len(), 3);
    }

    #[test]
    fn test_find_multiple_definitions() {
        let tokens = vec![
            Token::Colon(Data::default()),
            Token::Word(Data::new(0, 0, "")),
            Token::Semicolon(Data::default()),
            Token::Colon(Data::default()),
            Token::Word(Data::new(1, 1, "")),
            Token::Semicolon(Data::default()),
        ];
        let defs = find_colon_definitions(&tokens);
        assert_eq!(defs.len(), 2);
    }

    #[test]
    fn test_no_definitions() {
        let tokens = vec![
            Token::Word(Data::new(0, 0, "")),
            Token::Number(Data::new(1, 1, "")),
        ];
        let defs = find_colon_definitions(&tokens);
        assert_eq!(defs.len(), 0);
    }
}
