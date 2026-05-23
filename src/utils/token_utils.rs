//! Utility functions for working with tokens from the lexer.

use forth_lexer::token::Token;

/// Extract a word name from a token sequence starting at the given index.
///
/// Handles two cases:
/// - Single Word token
/// - Single Number token (valid Forth word name)
///
/// Returns None if the token pattern doesn't match any of these cases.
#[allow(dead_code)]
pub fn extract_word_name(tokens: &[Token], index: usize) -> Option<String> {
    if index >= tokens.len() {
        return None;
    }

    match &tokens[index] {
        // Just a Word
        Token::Word(data) => Some(data.value.to_string()),
        // Just a Number (valid Forth word name)
        Token::Number(data) => Some(data.value.to_string()),
        _ => None,
    }
}

/// Extract a word name along with its selection range from a token sequence.
///
/// Returns a tuple of (name, selection_start, selection_end).
/// See [`extract_word_name`] for the matching logic.
pub fn extract_word_name_with_range(
    tokens: &[Token],
    index: usize,
) -> Option<(String, usize, usize)> {
    if index >= tokens.len() {
        return None;
    }

    match &tokens[index] {
        // Just a Word
        Token::Word(data) => Some((data.value.to_string(), data.start, data.end)),
        // Just a Number (valid Forth word name)
        Token::Number(data) => Some((data.value.to_string(), data.start, data.end)),
        _ => None,
    }
}
