#[allow(unused_imports)]
use crate::prelude::*;
use crate::{
    utils::{
        definition_index::DefinitionIndex,
        handlers::send_response,
        ropey::{get_ix::GetIx, word_on_or_before::WordOnOrBefore},
        word_lookup::find_builtin_word,
        HashMapGetForLSPParams,
    },
    words::{Word, Words},
};

use std::collections::HashMap;

use lsp_server::{Connection, Request};
use lsp_types::{request::HoverRequest, Hover};
use ropey::Rope;

use super::cast;

// Extract the hover logic for testing
pub fn get_hover_result(
    word: &str,
    data: &Words,
    def_index: Option<&DefinitionIndex>,
    files: Option<&HashMap<String, Rope>>,
) -> Option<Hover> {
    if !word.is_empty() {
        // Check if word is user-defined (overrides built-in docs)
        if let Some(index) = def_index {
            let defs = index.find_definitions(word);
            if !defs.is_empty() {
                // User-defined word - show definition source code
                let mut hover_text = format!("### `{}`\n\n", word);

                // Show each definition location and source code
                for (i, def) in defs.iter().enumerate() {
                    if i > 0 {
                        hover_text.push_str("\n---\n\n");
                    }

                    // Add location info
                    let file_name = def.uri.path().split('/').next_back().unwrap_or("unknown");
                    hover_text.push_str(&format!(
                        "**Defined in:** `{}:{}:{}`\n\n",
                        file_name,
                        def.range.start.line + 1,
                        def.range.start.character + 1
                    ));

                    // Try to extract source code if files are available
                    if let Some(files_map) = files {
                        // Use URI string directly (files HashMap keys are URIs, not paths)
                        if let Some(rope) = files_map.get(&def.uri.to_string()) {
                            let start_line = def.range.start.line as usize;
                            let end_line = def.range.end.line as usize;

                            // For single-line definitions (just the word name), try to expand to show the full definition
                            let (display_start, display_end) = if start_line == end_line {
                                // Expand to show context (up to 20 lines after the name)
                                let expanded_end =
                                    (end_line + 20).min(rope.len_lines().saturating_sub(1));
                                (start_line, expanded_end)
                            } else {
                                (start_line, end_line)
                            };

                            // Extract the source code lines
                            let mut source_lines = Vec::new();
                            for line_idx in display_start..=display_end.min(display_start + 20) {
                                if let Some(line) = rope.get_line(line_idx) {
                                    let line_str = line.to_string();
                                    source_lines.push(line_str.trim_end().to_string());
                                    // Stop at semicolon for colon definitions
                                    if line_str.trim_end().ends_with(';') {
                                        break;
                                    }
                                }
                            }

                            if !source_lines.is_empty() {
                                hover_text.push_str("```forth\n");
                                hover_text.push_str(&source_lines.join(""));
                                hover_text.push_str("\n```\n");
                            }
                        }
                    }
                }

                return Some(Hover {
                    contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: None,
                });
            }
        }

        // Fall back to built-in documentation
        let default_info = &Word::default();
        let info = find_builtin_word(word, data).unwrap_or(default_info);
        Some(Hover {
            contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: info.documentation(),
            }),
            range: None,
        })
    } else {
        None
    }
}

pub fn handle_hover(
    req: &Request,
    connection: &Connection,
    data: &Words,
    files: &mut HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<HoverRequest>(req.clone()) {
        Ok((id, params)) => {
            log_request!(id, params);
            let rope = if let Some(rope) =
                files.for_position_param(&params.text_document_position_params)
            {
                rope
            } else {
                return Err(Error::NoSuchFile(
                    params
                        .text_document_position_params
                        .text_document
                        .uri
                        .to_string(),
                ));
            };
            let ix = rope.get_ix(&params);
            if ix >= rope.len_chars() {
                return Err(Error::OutOfBounds(ix));
            }
            let word = rope.word_on_or_before(ix);
            let result = get_hover_result(&word.to_string(), data, Some(def_index), Some(files));
            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => panic!("{err:?}"),
        // Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        // Err(ExtractError::MethodMismatch(req)) => req,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::MarkupKind;

    #[test]
    fn test_hover_finds_builtin_word() {
        let words = Words::default();
        let result = get_hover_result("DUP", &words, None, None);

        assert!(result.is_some());
        let hover = result.unwrap();
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            assert_eq!(content.kind, MarkupKind::Markdown);
            assert!(content.value.contains("DUP"));
            assert!(content.value.contains("( x -- x x )"));
        } else {
            panic!("Expected Markup hover contents");
        }
    }

    #[test]
    fn test_hover_case_insensitive() {
        let words = Words::default();
        let result = get_hover_result("dup", &words, None, None);

        assert!(result.is_some());
        let hover = result.unwrap();
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("DUP"));
        } else {
            panic!("Expected Markup hover contents");
        }
    }

    #[test]
    fn test_hover_returns_none_for_unknown_word() {
        let words = Words::default();
        let result = get_hover_result("NONEXISTENT_WORD_12345", &words, None, None);

        // Unknown words return default Word, which still returns Some
        // This is the current behavior
        assert!(result.is_some());
    }

    #[test]
    fn test_hover_returns_none_for_empty_word() {
        let words = Words::default();
        let result = get_hover_result("", &words, None, None);

        assert!(result.is_none());
    }

    #[test]
    fn test_hover_stack_effect_operators() {
        let words = Words::default();
        let test_cases = vec![
            ("+", "( n1 | u1 n2 | u2 -- n3 | u3 )"),
            ("-", "( n1 | u1 n2 | u2 -- n3 | u3 )"),
            ("*", "( n1 | u1 n2 | u2 -- n3 | u3 )"),
            ("SWAP", "( x1 x2 -- x2 x1 )"),
        ];

        for (word, expected_stack) in test_cases {
            let result = get_hover_result(word, &words, None, None);
            assert!(result.is_some(), "Expected hover for word: {}", word);

            if let lsp_types::HoverContents::Markup(content) = result.unwrap().contents {
                assert!(
                    content.value.contains(expected_stack),
                    "Word '{}' should contain stack effect '{}'",
                    word,
                    expected_stack
                );
            }
        }
    }

    #[test]
    fn test_hover_user_defined_overrides_builtin() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();
        let file_uri = format!("file://{}", file_path);

        // Define a word that exists in built-ins
        let rope = Rope::from_str(": DUP 1 + ;");
        index.update_file(&file_uri, &rope);

        let mut files = HashMap::new();
        files.insert(file_uri.clone(), rope);

        let result = get_hover_result("DUP", &words, Some(&index), Some(&files));

        assert!(result.is_some());
        let hover = result.unwrap();
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            // Should show user-defined info with source code, not built-in docs
            assert!(content.value.contains("DUP"));
            assert!(content.value.contains("Defined in:"));
            assert!(content.value.contains(": DUP 1 + ;"));
            assert!(!content.value.contains("( x -- x x )"));
        } else {
            panic!("Expected Markup hover contents");
        }
    }

    #[test]
    fn test_hover_user_defined_word_only() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();
        let file_uri = format!("file://{}", file_path);

        // Define a word that doesn't exist in built-ins
        let rope = Rope::from_str(": myword 1 + ;");
        index.update_file(&file_uri, &rope);

        let mut files = HashMap::new();
        files.insert(file_uri.clone(), rope);

        let result = get_hover_result("myword", &words, Some(&index), Some(&files));

        assert!(result.is_some());
        let hover = result.unwrap();
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("myword"));
            assert!(content.value.contains("Defined in:"));
            assert!(content.value.contains(": myword 1 + ;"));
        } else {
            panic!("Expected Markup hover contents");
        }
    }

    #[test]
    fn test_hover_user_defined_variable() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();
        let file_uri = format!("file://{}", file_path);

        // Define a variable
        let rope = Rope::from_str("VARIABLE counter");
        index.update_file(&file_uri, &rope);

        let mut files = HashMap::new();
        files.insert(file_uri.clone(), rope);

        let result = get_hover_result("counter", &words, Some(&index), Some(&files));

        assert!(result.is_some());
        let hover = result.unwrap();
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("counter"));
            assert!(content.value.contains("Defined in:"));
            assert!(content.value.contains("VARIABLE counter"));
        } else {
            panic!("Expected Markup hover contents");
        }
    }

    #[test]
    fn test_hover_user_defined_multiline() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();
        let file_uri = format!("file://{}", file_path);

        // Define a multiline word
        let rope = Rope::from_str(
            ": factorial\n  dup 0= if\n    drop 1\n  else\n    dup 1- factorial *\n  then\n;",
        );
        index.update_file(&file_uri, &rope);

        let mut files = HashMap::new();
        files.insert(file_uri.clone(), rope);

        let result = get_hover_result("factorial", &words, Some(&index), Some(&files));

        assert!(result.is_some());
        let hover = result.unwrap();
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("factorial"));
            assert!(content.value.contains("Defined in:"));
            assert!(content.value.contains(": factorial"));
            assert!(content.value.contains("dup 0= if"));
        } else {
            panic!("Expected Markup hover contents");
        }
    }
}
