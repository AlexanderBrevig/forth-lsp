//! Unified word lookup combining user definitions and built-in words.

use crate::utils::definition_index::DefinitionIndex;
use crate::words::{Word, Words};

/// Information about a looked-up word
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WordInfo<'a> {
    /// Word is defined by the user
    UserDefined {
        name: String,
        definition_count: usize,
    },
    /// Word is a built-in Forth word
    Builtin(&'a Word<'a>),
    /// Word not found
    NotFound,
}

/// Look up a word, checking user definitions first, then built-ins.
///
/// This provides a unified way to look up words with the correct precedence:
/// 1. User-defined words (from definition_index)
/// 2. Built-in words (from Words)
///
/// The lookup is case-insensitive.
#[allow(dead_code)]
pub fn lookup_word<'a>(
    word: &str,
    builtin_words: &'a Words,
    def_index: Option<&DefinitionIndex>,
) -> WordInfo<'a> {
    if word.is_empty() {
        return WordInfo::NotFound;
    }

    // Check user-defined words first (they override built-ins)
    if let Some(index) = def_index {
        let defs = index.find_definitions(word);
        if !defs.is_empty() {
            return WordInfo::UserDefined {
                name: word.to_string(),
                definition_count: defs.len(),
            };
        }
    }

    // Fall back to built-in words
    let word_lower = word.to_lowercase();
    builtin_words
        .words
        .iter()
        .find(|w| w.token.to_lowercase() == word_lower)
        .copied()
        .map(WordInfo::Builtin)
        .unwrap_or(WordInfo::NotFound)
}

/// Find a built-in word by name (case-insensitive).
/// Returns None if not found.
pub fn find_builtin_word<'a>(word: &str, builtin_words: &'a Words<'a>) -> Option<&'a Word<'a>> {
    if word.is_empty() {
        return None;
    }

    let word_lower = word.to_lowercase();
    builtin_words
        .words
        .iter()
        .find(|w| w.token.to_lowercase() == word_lower)
        .copied()
}

/// Check if a word is user-defined.
#[allow(dead_code)]
pub fn is_user_defined(word: &str, def_index: &DefinitionIndex) -> bool {
    !def_index.find_definitions(word).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_builtin_word() {
        let words = Words::default();

        // Lookup existing word (case-insensitive)
        let result = lookup_word("DUP", &words, None);
        assert!(matches!(result, WordInfo::Builtin(_)));

        let result = lookup_word("dup", &words, None);
        assert!(matches!(result, WordInfo::Builtin(_)));
    }

    #[test]
    fn test_lookup_not_found() {
        let words = Words::default();

        let result = lookup_word("NONEXISTENT_WORD_12345", &words, None);
        assert!(matches!(result, WordInfo::NotFound));
    }

    #[test]
    fn test_lookup_empty_word() {
        let words = Words::default();

        let result = lookup_word("", &words, None);
        assert!(matches!(result, WordInfo::NotFound));
    }

    #[test]
    fn test_find_builtin_word() {
        let words = Words::default();

        // Find existing word
        let word = find_builtin_word("SWAP", &words);
        assert!(word.is_some());
        assert_eq!(word.unwrap().token, "SWAP");

        // Case-insensitive
        let word = find_builtin_word("swap", &words);
        assert!(word.is_some());

        // Not found
        let word = find_builtin_word("NONEXISTENT", &words);
        assert!(word.is_none());
    }

    #[test]
    fn test_find_builtin_empty() {
        let words = Words::default();
        let word = find_builtin_word("", &words);
        assert!(word.is_none());
    }
}
