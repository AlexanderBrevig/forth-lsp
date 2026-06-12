use crate::prelude::*;
use crate::utils::data_to_position::to_line_char;
use crate::utils::definition_helpers::find_colon_definitions;
use crate::utils::definition_index::DefinitionIndex;
use crate::words::Words;
use forth_lexer::token::Token;
use lsp_server::{Connection, Message};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString, Position, PublishDiagnosticsParams, Range, Uri,
};
use ropey::Rope;
use std::collections::HashSet;

/// Check for undefined words using pre-parsed tokens
pub fn check_undefined_words_from_tokens(
    tokens: &[Token],
    rope: &Rope,
    def_index: &DefinitionIndex,
    builtin_words: &Words,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

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
    for result in find_colon_definitions(tokens) {
        if result.len() >= 3
            && let Token::Word(data) = &result[1]
        {
            local_definitions.insert(data.value.to_lowercase());
        }
    }

    // Also collect defining word definitions (VARIABLE, CONSTANT, CREATE, etc.)
    let defining_words = [
        "variable",
        "constant",
        "create",
        "value",
        "2variable",
        "2constant",
        "2value",
        "fvariable",
        "fconstant",
        "defer",
        "buffer:",
        "code",
    ];
    for i in 0..tokens.len().saturating_sub(1) {
        if let Token::Word(data) = &tokens[i]
            && defining_words
                .iter()
                .any(|&dw| dw.eq_ignore_ascii_case(data.value))
            && let Some(Token::Word(name_data)) = tokens.get(i + 1)
        {
            local_definitions.insert(name_data.value.to_lowercase());
        }
    }

    // Add local definitions to defined words
    for word in local_definitions {
        defined_words.insert(word);
    }

    // Check all word usages
    let mut in_string_literal = false;
    for token in tokens {
        if let Token::Word(data) = token {
            let word_lower = data.value.to_lowercase();

            // String literals: in Forth, a word ending in `"` introduces a
            // string that runs until the next word ending in `"`. This covers
            // standard words (`."`, `s"`, `c"`, `abort"`), gforth's escaped
            // variants (`s\"`, `.\"`), and debugger words (`break"`), as well
            // as any dialect-specific `..."` word, without an allowlist.
            if in_string_literal {
                // We're inside a string; the closing word also ends in `"`.
                if data.value.ends_with('"') {
                    in_string_literal = false;
                }
                // Either way, skip the string contents / closing word.
                continue;
            }
            if data.value.ends_with('"') {
                in_string_literal = true;
                defined_words.insert(word_lower.clone()); // Treat opener as known
                continue;
            }

            // Skip defining words and definition terminators
            if defining_words
                .iter()
                .any(|&dw| dw.eq_ignore_ascii_case(data.value))
                || data.value.eq_ignore_ascii_case("END-CODE")
            {
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
            let (start_line, start_char) = to_line_char(data.start, rope);
            let (end_line, end_char) = to_line_char(data.end, rope);
            let start_pos = Position {
                line: start_line,
                character: start_char,
            };
            let end_pos = Position {
                line: end_line,
                character: end_char,
            };

            diagnostics.push(Diagnostic {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("undefined-word".to_string())),
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

/// Check for unclosed colon definitions using pre-parsed tokens
pub fn check_unclosed_definitions_from_tokens(tokens: &[Token], rope: &Rope) -> Vec<Diagnostic> {
    use crate::utils::data_to_position::ToPosition;
    use std::collections::HashSet;

    let mut diagnostics = Vec::new();

    // Get all matched colon definition start positions
    let matched_starts: HashSet<usize> = find_colon_definitions(tokens)
        .iter()
        .filter_map(|def| {
            if let Token::Colon(data) = &def[0] {
                Some(data.start)
            } else {
                None
            }
        })
        .collect();

    // Find all Colon tokens that are NOT in matched definitions
    for (i, token) in tokens.iter().enumerate() {
        if let Token::Colon(colon_data) = token {
            if matched_starts.contains(&colon_data.start) {
                continue;
            }

            // This is an unmatched colon - find insertion point
            // Scan forward to next Colon or end of tokens, take the last token before it
            let mut last_token_before_next = None;
            for next_token in &tokens[i + 1..] {
                if matches!(next_token, Token::Colon(_)) {
                    break;
                }
                if !matches!(next_token, Token::Eof(_)) {
                    last_token_before_next = Some(next_token);
                }
            }

            // If we found no tokens after the colon, use the colon itself
            let insert_data = last_token_before_next
                .map(|t| t.get_data())
                .unwrap_or(colon_data);
            let insert_pos = insert_data.to_position_end(rope);

            let colon_range = colon_data.to_range(rope);

            diagnostics.push(Diagnostic {
                range: colon_range,
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("unclosed-definition".to_string())),
                code_description: None,
                source: Some("forth-lsp".to_string()),
                message: "Unclosed definition".to_string(),
                related_information: None,
                tags: None,
                data: Some(serde_json::json!({
                    "insert_line": insert_pos.line,
                    "insert_character": insert_pos.character,
                })),
            });
        }
    }

    diagnostics
}

/// Check for unmatched delimiters using a source string
pub fn check_unmatched_delimiters_from_source(text: &str, rope: &Rope) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for unmatched parentheses in comments
    // In Forth, comments are ( comment ) and the lexer tokenizes them
    // We check if there are unclosed comments by examining the source text
    let mut paren_depth = 0;
    let mut paren_start: Option<usize> = None;

    for (byte_offset, ch) in text.char_indices() {
        if ch == '(' {
            if paren_depth == 0 {
                paren_start = Some(byte_offset);
            }
            paren_depth += 1;
        } else if ch == ')' {
            if paren_depth == 0 {
                // Unmatched closing paren
                let (line, character) = to_line_char(byte_offset, rope);
                let pos = Position { line, character };
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
        let (line, character) = to_line_char(start, rope);
        let start_pos = Position { line, character };
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

/// Get all diagnostics for a file using pre-parsed tokens and source string
/// This avoids re-parsing when tokens are already available
pub fn get_diagnostics_from_tokens(
    tokens: &[Token],
    source: &str,
    rope: &Rope,
    def_index: &DefinitionIndex,
    builtin_words: &Words,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(check_undefined_words_from_tokens(
        tokens,
        rope,
        def_index,
        builtin_words,
    ));
    diagnostics.extend(check_unclosed_definitions_from_tokens(tokens, rope));
    diagnostics.extend(check_unmatched_delimiters_from_source(source, rope));
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
    use forth_lexer::parser::Lexer;
    use ropey::Rope;
    use std::env;

    fn check_undefined_words(
        rope: &Rope,
        def_index: &DefinitionIndex,
        builtin_words: &Words,
    ) -> Vec<Diagnostic> {
        let source = rope.to_string();
        let tokens = Lexer::new(&source).parse();
        check_undefined_words_from_tokens(&tokens, rope, def_index, builtin_words)
    }

    fn get_diagnostics(
        rope: &Rope,
        def_index: &DefinitionIndex,
        builtin_words: &Words,
    ) -> Vec<Diagnostic> {
        let source = rope.to_string();
        let tokens = Lexer::new(&source).parse();
        get_diagnostics_from_tokens(&tokens, &source, rope, def_index, builtin_words)
    }

    #[test]
    fn test_constant_definition_not_flagged() {
        let rope = Rope::from_str("1 constant ONE\n: test ONE ;");
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir
            .join("test_const.forth")
            .to_string_lossy()
            .to_string();
        index.update_file(&file_path, &rope);
        let words = Words::default();

        let diagnostics = check_undefined_words(&rope, &index, &words);

        // ONE should NOT be flagged as undefined - it's defined via CONSTANT
        for d in &diagnostics {
            eprintln!("Diagnostic: {}", d.message);
        }
        assert!(
            !diagnostics.iter().any(|d| d.message.contains("ONE")),
            "ONE should not be flagged as undefined after '1 constant ONE'"
        );
        assert_eq!(diagnostics.len(), 0, "No diagnostics expected");
    }

    #[test]
    fn test_constant_not_flagged_same_file_no_index() {
        // Test the case where the file is NOT in the definition index yet
        // local_definitions should still find CONSTANT definitions
        let rope = Rope::from_str("1 constant ONE\n: test ONE ;");
        let index = DefinitionIndex::new(); // Empty index!
        let words = Words::default();

        let diagnostics = check_undefined_words(&rope, &index, &words);

        assert!(
            !diagnostics.iter().any(|d| d.message.contains("ONE")),
            "ONE should not be flagged - defined via 'constant' in same file"
        );
    }

    #[test]
    fn test_variable_not_flagged_same_file_no_index() {
        let rope = Rope::from_str("variable COUNT\n: test COUNT @ ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = check_undefined_words(&rope, &index, &words);

        assert!(
            !diagnostics.iter().any(|d| d.message.contains("COUNT")),
            "COUNT should not be flagged - defined via 'variable' in same file"
        );
    }

    #[test]
    fn test_value_not_flagged_same_file_no_index() {
        let rope = Rope::from_str("10 value LIMIT\n: test LIMIT ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = check_undefined_words(&rope, &index, &words);

        assert!(
            !diagnostics.iter().any(|d| d.message.contains("LIMIT")),
            "LIMIT should not be flagged - defined via 'value' in same file"
        );
    }

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
    fn test_gforth_escaped_string_not_flagged() {
        // gforth `s\"` creates an escaped string; contents must not be flagged.
        let rope = Rope::from_str(r#": greet s\" Hello world " type ;"#);
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(
            diagnostics.len(),
            0,
            "Escaped string (s\\\") contents should not be flagged: {diagnostics:?}"
        );
    }

    #[test]
    fn test_arbitrary_quote_word_not_flagged() {
        // Any word ending in `"` opens a string (e.g. gforth's debugger
        // `break"`), so its contents must not be flagged as undefined.
        let rope = Rope::from_str(r#": foo break" stopped here " ;"#);
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(
            diagnostics.len(),
            0,
            "Contents after a `...\"` word should not be flagged: {diagnostics:?}"
        );
    }

    #[test]
    fn test_multiple_strings_on_one_line() {
        let rope = Rope::from_str(r#": foo s" alpha" s" beta" ;"#);
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(
            diagnostics.len(),
            0,
            "Neither string's contents should be flagged: {diagnostics:?}"
        );
    }

    #[test]
    fn test_undefined_word_after_string_still_flagged() {
        // The string must close at the next `"`-word so later code is still
        // checked; `zzzundefined` should be reported.
        let rope = Rope::from_str(r#": foo s" hello " zzzundefined ;"#);
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert_eq!(
            diagnostics.len(),
            1,
            "expected one diagnostic: {diagnostics:?}"
        );
        assert!(
            diagnostics[0].message.contains("zzzundefined"),
            "expected zzzundefined to be flagged: {:?}",
            diagnostics[0].message
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
    fn test_unclosed_definition_diagnostic() {
        let rope = Rope::from_str(": foo 1 +");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "Unclosed definition"
                    && d.severity == Some(DiagnosticSeverity::WARNING)),
            "Expected unclosed definition diagnostic, got: {:?}",
            diagnostics
        );

        // Check that data contains insertion position
        let unclosed = diagnostics
            .iter()
            .find(|d| d.message == "Unclosed definition")
            .unwrap();
        assert!(unclosed.data.is_some());
        let data = unclosed.data.as_ref().unwrap();
        assert!(data.get("insert_line").is_some());
        assert!(data.get("insert_character").is_some());
    }

    #[test]
    fn test_closed_definition_no_unclosed_diagnostic() {
        let rope = Rope::from_str(": foo 1 + ;");
        let index = DefinitionIndex::new();
        let words = Words::default();

        let diagnostics = get_diagnostics(&rope, &index, &words);

        assert!(
            !diagnostics
                .iter()
                .any(|d| d.message == "Unclosed definition"),
            "Closed definition should not trigger unclosed diagnostic"
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

    #[test]
    fn test_code_definition_not_flagged() {
        let rope = Rope::from_str("CODE syscall0 MOV RAX RBX END-CODE\n: test syscall0 ;");
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir
            .join("test_code.forth")
            .to_string_lossy()
            .to_string();
        index.update_file(&file_path, &rope);
        let words = Words::default();

        let diagnostics = check_undefined_words(&rope, &index, &words);

        assert!(
            !diagnostics.iter().any(|d| d.message.contains("syscall0")),
            "syscall0 should not be flagged - defined via CODE"
        );
        assert!(
            !diagnostics.iter().any(|d| d.message.contains("END-CODE")),
            "END-CODE should not be flagged as undefined"
        );
    }

    #[test]
    fn test_unmatched_paren_after_multibyte_chars_does_not_crash() {
        // Regression: char_indices() returns byte offsets, but they were
        // passed to char_to_line() which expects char offsets. With enough
        // multi-byte chars the byte offset exceeds len_chars() → panic.
        let prefix: String = "è".repeat(100);
        let src = format!("{}\n)", prefix);
        let rope = Rope::from_str(&src);
        let diagnostics = check_unmatched_delimiters_from_source(&src, &rope);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("Unmatched closing parenthesis")),
        );
    }

    #[test]
    fn test_unclosed_paren_after_multibyte_chars_does_not_crash() {
        let prefix: String = "è".repeat(100);
        let src = format!("{}\n( unclosed", prefix);
        let rope = Rope::from_str(&src);
        let diagnostics = check_unmatched_delimiters_from_source(&src, &rope);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("Unclosed parenthesis")),
        );
    }

    #[test]
    fn test_unmatched_paren_position_after_multibyte_char() {
        // "\ è\n)" — the ')' is on line 1, character 0.
        // Before the fix, byte offset 5 was used as char offset,
        // giving a wrong (or panicking) position.
        let src = "\\ è\n)";
        let rope = Rope::from_str(src);
        let diagnostics = check_unmatched_delimiters_from_source(src, &rope);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 1);
        assert_eq!(diagnostics[0].range.start.character, 0);
    }

    #[test]
    fn test_unclosed_paren_position_after_multibyte_char() {
        // "\ è\n( open" — the '(' is on line 1, character 0.
        let src = "\\ è\n( open";
        let rope = Rope::from_str(src);
        let diagnostics = check_unmatched_delimiters_from_source(src, &rope);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 1);
        assert_eq!(diagnostics[0].range.start.character, 0);
    }

    #[test]
    fn test_unmatched_paren_position_same_line_as_multibyte() {
        // "è )" — the ')' is on line 0, character 2 (not byte 3).
        let src = "è )";
        let rope = Rope::from_str(src);
        let diagnostics = check_unmatched_delimiters_from_source(src, &rope);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 0);
        assert_eq!(diagnostics[0].range.start.character, 2);
    }

    #[test]
    fn test_diagnostics_multibyte_utf8_full_pipeline() {
        // Full diagnostics pipeline with multi-byte UTF-8 (Italian text)
        let src = "\\ tabella è unica\r\n: SAVE ( -- ) ;\r\n\\ così il test\r\n";
        let rope = Rope::from_str(src);
        let mut index = DefinitionIndex::new();
        let file_uri = "file:///test/test.f".to_string();
        index.update_file(&file_uri, &rope);
        let words = Words::default();
        let _ = get_diagnostics(&rope, &index, &words);
    }
}
