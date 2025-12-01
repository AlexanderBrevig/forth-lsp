#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::handlers::{common::ExtractedPosition, send_response};
use crate::utils::ropey::word_on_or_before::WordOnOrBefore;
use crate::utils::word_lookup::find_builtin_word;
use crate::words::Words;

use lsp_server::{Connection, Request};
use lsp_types::{SignatureHelp, SignatureInformation, request::SignatureHelpRequest};
use ropey::Rope;
use std::collections::HashMap;

use super::cast;

// Extract signature help logic for testing
pub fn get_signature_help(word: &str, builtin_words: &Words) -> Option<SignatureHelp> {
    if word.is_empty() {
        return None;
    }

    // Find the word in built-in words
    let word_info = find_builtin_word(word, builtin_words)?;

    // Extract stack effect from documentation
    let stack_effect = extract_stack_effect(&word_info.documentation());

    let signature = SignatureInformation {
        label: format!("{} {}", word_info.token, stack_effect),
        documentation: Some(lsp_types::Documentation::String(word_info.documentation())),
        parameters: None,
        active_parameter: None,
    };

    Some(SignatureHelp {
        signatures: vec![signature],
        active_signature: Some(0),
        active_parameter: None,
    })
}

// Extract stack effect notation from documentation
// Looks for patterns like "( x -- x x )" or "( n1 n2 -- n3 )"
fn extract_stack_effect(doc: &str) -> String {
    // Find all pairs of parentheses and check which contains "--"
    let mut start_pos = 0;
    while let Some(start) = doc[start_pos..].find('(') {
        let abs_start = start_pos + start;
        if let Some(end) = doc[abs_start..].find(')') {
            let effect = &doc[abs_start..abs_start + end + 1];
            // Check if it contains "--" which is typical of stack effects
            if effect.contains("--") {
                return effect.trim().to_string();
            }
        }
        start_pos = abs_start + 1;
    }
    "".to_string()
}

pub fn handle_signature_help(
    req: &Request,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    builtin_words: &Words,
) -> Result<()> {
    match cast::<SignatureHelpRequest>(req.clone()) {
        Ok((id, params)) => {
            let pos = ExtractedPosition::from_parts(
                &params.text_document_position_params.text_document,
                &params.text_document_position_params.position,
            )?;

            eprintln!("#{id}: signature help at {}", pos.format());

            let rope = files.get(&pos.file_uri).ok_or_else(|| {
                Error::NoSuchFile(
                    params
                        .text_document_position_params
                        .text_document
                        .uri
                        .to_string(),
                )
            })?;

            let line = pos.line as usize;
            let character = pos.character as usize;

            // Check bounds
            if line >= rope.len_lines() {
                send_response(connection, id, None::<SignatureHelp>)?;
                return Ok(());
            }

            let ix = rope.line_to_char(line) + character;

            if ix >= rope.len_chars() {
                send_response(connection, id, None::<SignatureHelp>)?;
                return Ok(());
            }

            let word = rope.word_on_or_before(ix).to_string();
            let signature_help = get_signature_help(&word, builtin_words);

            send_response(connection, id, signature_help)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Signature help", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::Words;

    #[test]
    fn test_signature_help_builtin_word() {
        let words = Words::default();
        let sig = get_signature_help("DUP", &words);

        assert!(sig.is_some());
        let sig = sig.unwrap();
        assert_eq!(sig.signatures.len(), 1);
        assert!(sig.signatures[0].label.contains("DUP"));
        assert!(sig.signatures[0].label.contains("( x -- x x )"));
    }

    #[test]
    fn test_signature_help_case_insensitive() {
        let words = Words::default();
        let sig_upper = get_signature_help("SWAP", &words);
        let sig_lower = get_signature_help("swap", &words);

        assert!(sig_upper.is_some());
        assert!(sig_lower.is_some());

        let sig_upper = sig_upper.unwrap();
        let sig_lower = sig_lower.unwrap();

        assert_eq!(sig_upper.signatures[0].label, sig_lower.signatures[0].label);
    }

    #[test]
    fn test_signature_help_unknown_word() {
        let words = Words::default();
        let sig = get_signature_help("NONEXISTENT_WORD_12345", &words);

        assert!(sig.is_none());
    }

    #[test]
    fn test_signature_help_empty_word() {
        let words = Words::default();
        let sig = get_signature_help("", &words);

        assert!(sig.is_none());
    }

    #[test]
    fn test_signature_help_arithmetic_operators() {
        let words = Words::default();
        let test_words = vec!["+", "-", "*", "/"];

        for word in test_words {
            let sig = get_signature_help(word, &words);
            assert!(sig.is_some(), "Expected signature for '{}'", word);

            let sig = sig.unwrap();
            assert!(sig.signatures[0].label.contains(word));
            // Arithmetic operators should have stack effects
            assert!(sig.signatures[0].label.contains("("));
        }
    }

    #[test]
    fn test_signature_help_has_documentation() {
        let words = Words::default();
        let sig = get_signature_help("DROP", &words);

        assert!(sig.is_some());
        let sig = sig.unwrap();

        // Should have documentation
        assert!(sig.signatures[0].documentation.is_some());

        if let Some(lsp_types::Documentation::String(doc)) = &sig.signatures[0].documentation {
            assert!(!doc.is_empty());
            assert!(doc.contains("DROP"));
        } else {
            panic!("Expected string documentation");
        }
    }

    #[test]
    fn test_extract_stack_effect() {
        assert_eq!(
            extract_stack_effect("DUP ( x -- x x ) duplicates"),
            "( x -- x x )"
        );
        assert_eq!(
            extract_stack_effect("SWAP ( x1 x2 -- x2 x1 ) swaps"),
            "( x1 x2 -- x2 x1 )"
        );
        assert_eq!(extract_stack_effect("No stack effect here"), "");
        assert_eq!(
            extract_stack_effect("Multiple (parens) ( n -- n n ) here"),
            "( n -- n n )"
        );
    }

    #[test]
    fn test_signature_help_active_signature() {
        let words = Words::default();
        let sig = get_signature_help("DUP", &words);

        assert!(sig.is_some());
        let sig = sig.unwrap();

        // Should have active signature set to 0
        assert_eq!(sig.active_signature, Some(0));
    }
}
