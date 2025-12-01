use crate::prelude::*;
use crate::utils::definition_helpers::find_colon_definitions;
use crate::utils::definition_index::DefinitionIndex;
use crate::words::Words;
use forth_lexer::parser::Lexer;
use forth_lexer::token::Token;
use lsp_server::{Connection, Message};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, PublishDiagnosticsParams, Range, Uri};
use ropey::Rope;
use std::collections::HashSet;

/// Check for undefined words in a Forth file
pub fn check_undefined_words(
    rope: &Rope,
    def_index: &DefinitionIndex,
    builtin_words: &Words,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let progn = rope.to_string();
    let mut lexer = Lexer::new(&progn);

    // Try to parse, but return empty if lexer fails on malformed input
    let tokens = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lexer.parse())) {
        Ok(tokens) => tokens,
        Err(_) => return diagnostics, // Return empty diagnostics if lexer panics
    };

    // Build set of all defined words (built-in + user-defined)
    let mut defined_words = HashSet::new();

    // Add built-in words
    for word in &builtin_words.words {
        defined_words.insert(word.token.to_lowercase());
    }

    // Add user-defined words
    for word in def_index.all_words() {
        defined_words.insert(word.to_lowercase());
    }

    // Collect all definition names from this file to avoid false positives
    let mut local_definitions = HashSet::new();
    for result in find_colon_definitions(&tokens) {
        if result.len() >= 3
            && let Token::Word(data) = &result[1]
        {
            local_definitions.insert(data.value.to_lowercase());
        }
    }

    // Add local definitions to defined words
    for word in local_definitions {
        defined_words.insert(word);
    }

    // Check all word usages
    let mut in_string_literal = false;
    for token in &tokens {
        if let Token::Word(data) = token {
            let word_lower = data.value.to_lowercase();

            // Check if this starts a string literal
            if matches!(word_lower.as_str(), ".\"" | "s\"" | "c\"" | "abort\"") {
                in_string_literal = true;
                defined_words.insert(word_lower.clone()); // Ensure string words are defined
                continue;
            }

            // Check if this ends a string literal
            if in_string_literal && data.value.ends_with('"') {
                in_string_literal = false;
                continue;
            }

            // Skip words inside string literals
            if in_string_literal {
                continue;
            }

            // Skip if word is defined
            if defined_words.contains(&word_lower) {
                continue;
            }

            // Skip numeric literals
            if data.value.parse::<i64>().is_ok() || data.value.parse::<f64>().is_ok() {
                continue;
            }

            // Create diagnostic for undefined word
            let start_pos = Position {
                line: rope.char_to_line(data.start) as u32,
                character: (data.start - rope.line_to_char(rope.char_to_line(data.start))) as u32,
            };
            let end_pos = Position {
                line: rope.char_to_line(data.end) as u32,
                character: (data.end - rope.line_to_char(rope.char_to_line(data.end))) as u32,
            };

            diagnostics.push(Diagnostic {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: None,
                code_description: None,
                source: Some("forth-lsp".to_string()),
                message: format!("Undefined word: {}", data.value),
                related_information: None,
                tags: None,
                data: None,
            });
        }
    }

    diagnostics
}

/// Check for unmatched delimiters (parentheses, brackets, etc.)
pub fn check_unmatched_delimiters(rope: &Rope) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for unmatched parentheses in comments
    // In Forth, comments are ( comment ) and the lexer tokenizes them
    // We check if there are unclosed comments by examining the source text
    let text = rope.to_string();
    let mut paren_depth = 0;
    let mut paren_start: Option<usize> = None;

    for (i, ch) in text.char_indices() {
        if ch == '(' {
            if paren_depth == 0 {
                paren_start = Some(i);
            }
            paren_depth += 1;
        } else if ch == ')' {
            if paren_depth == 0 {
                // Unmatched closing paren
                let pos = Position {
                    line: rope.char_to_line(i) as u32,
                    character: (i - rope.line_to_char(rope.char_to_line(i))) as u32,
                };
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: pos,
                        end: Position {
                            line: pos.line,
                            character: pos.character + 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("forth-lsp".to_string()),
                    message: "Unmatched closing parenthesis".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            } else {
                paren_depth -= 1;
            }
        }
    }

    // Report unclosed parentheses
    if paren_depth > 0
        && let Some(start) = paren_start
    {
        let start_pos = Position {
            line: rope.char_to_line(start) as u32,
            character: (start - rope.line_to_char(rope.char_to_line(start))) as u32,
        };
        diagnostics.push(Diagnostic {
            range: Range {
                start: start_pos,
                end: Position {
                    line: start_pos.line,
                    character: start_pos.character + 1,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("forth-lsp".to_string()),
            message: "Unclosed parenthesis".to_string(),
            related_information: None,
            tags: None,
            data: None,
        });
    }

    diagnostics
}

/// Get all diagnostics for a file
pub fn get_diagnostics(
    rope: &Rope,
    def_index: &DefinitionIndex,
    builtin_words: &Words,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for undefined words
    diagnostics.extend(check_undefined_words(rope, def_index, builtin_words));

    // Check for unmatched delimiters
    diagnostics.extend(check_unmatched_delimiters(rope));

    diagnostics
}

/// Publish diagnostics to the LSP client
pub fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<Diagnostic>,
    version: i32,
) -> Result<()> {
    let diagnostic_params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: Some(version),
    };

    let notification = lsp_server::Notification {
        method: "textDocument/publishDiagnostics".to_string(),
        params: serde_json::to_value(diagnostic_params)
            .map_err(|e| Error::Generic(format!("Serialization error: {}", e)))?,
    };

    connection
        .sender
        .send(Message::Notification(notification))
        .map_err(|err| Error::SendError(err.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::definition_index::DefinitionIndex;
    use crate::words::Words;
    use ropey::Rope;
    use std::env;

    #[test]
    fn test_no_diagnostics_for_valid_code() {
        let rope = Rope::from_str(": add1 1 + ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_undefined_word_warning() {
        let rope = Rope::from_str(": test undefined-word ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::WARNING));
        assert!(diagnostics[0].message.contains("undefined-word"));
    }

    #[test]
    fn test_string_literals_not_flagged() {
        let rope = Rope::from_str(r#": greet ." Hello world " CR ;"#);
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        // String literal content ("Hello", "world", quotes) should not be flagged
        assert_eq!(
            diagnostics.len(),
            0,
            "String literal content should not be flagged as undefined"
        );
    }

    #[test]
    fn test_builtin_words_not_flagged() {
        let rope = Rope::from_str(": test DUP SWAP DROP ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_user_defined_words_not_flagged() {
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("diag1.forth").to_string_lossy().to_string();

        let rope1 = Rope::from_str(": myword 1 + ;");
        index.update_file(&file1, &rope1);

        let rope2 = Rope::from_str(": test myword ;");
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope2, &index, &words);

        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_local_definitions_not_flagged() {
        let rope = Rope::from_str(": helper 1 + ;\n: test helper ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        // Should not flag 'helper' as undefined
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_numeric_literals_not_flagged() {
        let rope = Rope::from_str(": test 123 456 -789 3.14 ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_unmatched_closing_paren() {
        let rope = Rope::from_str(": test ) ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.severity == Some(DiagnosticSeverity::ERROR)
                    && d.message.contains("Unmatched closing parenthesis"))
        );
    }

    #[test]
    fn test_unclosed_paren() {
        let rope = Rope::from_str(": test ( unclosed comment ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.severity == Some(DiagnosticSeverity::ERROR)
                    && d.message.contains("Unclosed parenthesis"))
        );
    }

    #[test]
    fn test_matched_parens_no_error() {
        let rope = Rope::from_str(": test ( comment ) 1 + ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert!(
            !diagnostics
                .iter()
                .any(|d| d.message.contains("parenthesis"))
        );
    }

    #[test]
    fn test_multiple_diagnostics() {
        let rope = Rope::from_str(": test undefined1 undefined2 ) ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        // Should have 4 diagnostics:
        // - 2 undefined words (undefined1, undefined2)
        // - 1 undefined word ")" (lexer tokenizes it as a word when malformed)
        // - 1 unmatched closing paren
        assert_eq!(diagnostics.len(), 4);

        // Verify we have both types of diagnostics
        assert!(diagnostics.iter().any(|d| d.message.contains("undefined1")));
        assert!(diagnostics.iter().any(|d| d.message.contains("undefined2")));
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("Unmatched closing parenthesis"))
        );
    }

    #[test]
    fn test_case_insensitive_definitions() {
        let rope = Rope::from_str(": MyWord 1 + ;\n: test MYWORD myword MyWord ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        // Should not flag any usage of MyWord regardless of case
        assert_eq!(diagnostics.len(), 0);
    }
}
