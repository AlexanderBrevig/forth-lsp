//! Utility functions for working with tokens from the lexer.

use forth_lexer::token::Token;

/// Extract a word name from a token sequence starting at the given index.
///
/// Handles three cases:
/// - Number followed by Word with no gap = combined name (e.g., 2SWAP)
/// - Single Word token
/// - Single Number token (valid Forth word name)
///
/// Returns None if the token pattern doesn't match any of these cases.
#[allow(dead_code)]
pub fn extract_word_name(tokens: &[Token], index: usize) -> Option<String> {
    if index >= tokens.len() {
        return None;
    }

    match (&tokens[index], tokens.get(index + 1)) {
        // Number followed by Word with no gap = combined name (e.g., 2SWAP)
        (Token::Number(num_data), Some(Token::Word(word_data)))
            if num_data.end == word_data.start =>
        {
            Some(format!("{}{}", num_data.value, word_data.value))
        }
        // Just a Word
        (Token::Word(data), _) => Some(data.value.to_string()),
        // Just a Number (valid Forth word name)
        (Token::Number(data), _) => Some(data.value.to_string()),
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

    match (&tokens[index], tokens.get(index + 1)) {
        // Number followed by Word with no gap = combined name (e.g., 2SWAP)
        (Token::Number(num_data), Some(Token::Word(word_data)))
            if num_data.end == word_data.start =>
        {
            Some((
                format!("{}{}", num_data.value, word_data.value),
                num_data.start,
                word_data.end,
            ))
        }
        // Just a Word
        (Token::Word(data), _) => Some((data.value.to_string(), data.start, data.end)),
        // Just a Number (valid Forth word name)
        (Token::Number(data), _) => Some((data.value.to_string(), data.start, data.end)),
        _ => None,
    }
}
