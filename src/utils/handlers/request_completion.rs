#[allow(unused_imports)]
use crate::prelude::*;
use crate::{
    utils::{
        HashMapGetForLSPParams,
        definition_index::DefinitionIndex,
        handlers::send_response,
        ropey::{RopeSliceIsLower, get_ix::GetIx, word_at::WordAt},
    },
    words::Words,
};

use std::collections::{HashMap, HashSet};

use lsp_server::{Connection, Request};
use lsp_types::{CompletionItem, CompletionResponse, request::Completion};
use ropey::Rope;

use super::cast;

// Extract completion logic for testing
pub fn get_completions(
    word_prefix: &str,
    use_lower: bool,
    data: &Words,
    def_index: Option<&DefinitionIndex>,
    files: Option<&HashMap<String, Rope>>,
) -> Option<CompletionResponse> {
    if !word_prefix.is_empty() {
        let mut ret = vec![];
        let mut user_defined_words = HashSet::new();

        // Add user-defined words from index
        if let Some(index) = def_index {
            for word in index.all_words() {
                if word.to_lowercase().starts_with(&word_prefix.to_lowercase()) {
                    user_defined_words.insert(word.clone());
                    let label = if use_lower {
                        word.to_lowercase()
                    } else {
                        word.clone()
                    };

                    // Get definitions to extract documentation
                    let definitions = index.find_definitions(&word);
                    let (detail, documentation) = if !definitions.is_empty() {
                        let def = &definitions[0];
                        let file_name = def
                            .uri
                            .path()
                            .as_str()
                            .split('/')
                            .next_back()
                            .unwrap_or("unknown");

                        // Try to extract source code like hover does
                        let mut doc_text = format!(
                            "**Defined in:** `{}:{}:{}`\n\n",
                            file_name,
                            def.range.start.line + 1,
                            def.range.start.character + 1
                        );

                        // Extract source code if files are available
                        if let Some(files_map) = files
                            && let Some(rope) = files_map.get(&def.uri.to_string())
                        {
                            let start_line = def.range.start.line as usize;
                            let end_line = def.range.end.line as usize;

                            // For single-line definitions (just the word name), expand to show full definition
                            let (display_start, display_end) = if start_line == end_line {
                                let expanded_end =
                                    (end_line + 20).min(rope.len_lines().saturating_sub(1));
                                (start_line, expanded_end)
                            } else {
                                (start_line, end_line)
                            };

                            // Extract source code lines
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
                                doc_text.push_str("```forth\n");
                                doc_text.push_str(&source_lines.join(""));
                                doc_text.push_str("\n```");
                            }
                        }

                        (
                            Some(format!("user-defined in {}", file_name)),
                            Some(lsp_types::Documentation::MarkupContent(
                                lsp_types::MarkupContent {
                                    kind: lsp_types::MarkupKind::Markdown,
                                    value: doc_text,
                                },
                            )),
                        )
                    } else {
                        (Some("user-defined".to_string()), None)
                    };

                    ret.push(CompletionItem {
                        label,
                        detail,
                        documentation,
                        ..Default::default()
                    });
                }
            }
        }

        // Add built-in words (skip if overridden by user)
        let candidates = data.words.iter().filter(|x| {
            x.token
                .to_lowercase()
                .starts_with(&word_prefix.to_lowercase())
        });
        for candidate in candidates {
            // Skip if user has defined this word
            if user_defined_words.contains(&candidate.token.to_lowercase()) {
                continue;
            }

            let label = if use_lower {
                candidate.token.to_lowercase()
            } else {
                candidate.token.to_owned()
            };
            ret.push(CompletionItem {
                label,
                detail: Some(candidate.stack.to_owned()),
                documentation: Some(lsp_types::Documentation::MarkupContent(
                    lsp_types::MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: candidate.documentation(),
                    },
                )),
                ..Default::default()
            });
        }
        Some(CompletionResponse::Array(ret))
    } else {
        None
    }
}

pub fn handle_completion(
    req: &Request,
    connection: &Connection,
    data: &Words,
    files: &mut HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<Completion>(req.clone()) {
        Ok((id, params)) => {
            log_request!(id, params);
            let rope = if let Some(rope) = files.for_position_param(&params.text_document_position)
            {
                rope
            } else {
                return Err(Error::NoSuchFile(
                    params.text_document_position.text_document.uri.to_string(),
                ));
            };
            let mut ix = rope.get_ix(&params);
            if ix >= rope.len_chars() {
                return Err(Error::OutOfBounds(ix));
            }
            if let Some(char_at_ix) = rope.get_char(ix)
                && char_at_ix.is_whitespace()
                && ix > 0
            {
                ix -= 1;
            }
            let word = rope.word_at(ix);
            let result = if word.len_chars() > 0 {
                log_debug!("Found word {}", word);
                let use_lower = word.is_lowercase();
                get_completions(
                    &word.to_string(),
                    use_lower,
                    data,
                    Some(def_index),
                    Some(files),
                )
            } else {
                None
            };
            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Completion", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_finds_matching_words() {
        let words = Words::default();
        let result = get_completions("DU", false, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(!items.is_empty());
            // Should find DUP, 2DUP, ?DUP
            let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
            assert!(labels.contains(&"DUP".to_string()));
            assert!(labels.iter().all(|l| l.to_uppercase().starts_with("DU")));
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_respects_lowercase() {
        let words = Words::default();
        let result = get_completions("du", true, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(!items.is_empty());
            // All items should be lowercase when use_lower is true
            for item in items {
                assert_eq!(item.label, item.label.to_lowercase());
                assert!(item.label.starts_with("du"));
            }
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_respects_uppercase() {
        let words = Words::default();
        let result = get_completions("DU", false, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(!items.is_empty());
            // Items should keep their original case when use_lower is false
            let has_uppercase = items
                .iter()
                .any(|i| i.label.chars().any(|c| c.is_uppercase()));
            assert!(has_uppercase);
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_includes_stack_effects() {
        let words = Words::default();
        let result = get_completions("SWAP", false, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            let swap_item = items.iter().find(|i| i.label == "SWAP");
            assert!(swap_item.is_some());

            let swap = swap_item.unwrap();
            assert!(swap.detail.is_some());
            assert_eq!(swap.detail.as_ref().unwrap(), "( x1 x2 -- x2 x1 )");
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_includes_documentation() {
        let words = Words::default();
        let result = get_completions("+", false, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            let plus_item = items.iter().find(|i| i.label == "+");
            assert!(plus_item.is_some());

            let plus = plus_item.unwrap();
            assert!(plus.documentation.is_some());
            if let Some(lsp_types::Documentation::MarkupContent(content)) = &plus.documentation {
                assert!(content.value.contains("+"));
            }
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_empty_prefix() {
        let words = Words::default();
        let result = get_completions("", false, &words, None, None);

        assert!(result.is_none());
    }

    #[test]
    fn test_completion_no_matches() {
        let words = Words::default();
        let result = get_completions("ZZZZNONEXISTENT", false, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(items.is_empty());
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_single_character() {
        let words = Words::default();
        let result = get_completions("+", false, &words, None, None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(!items.is_empty());
            // Should include +, +!, +LOOP
            let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
            assert!(labels.contains(&"+".to_string()));
            assert!(labels.iter().all(|l| l.starts_with("+")));
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_includes_user_defined_words() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();

        index.update_file(
            &file_path,
            &Rope::from_str(": myword 1 + ;\n: mytest 2 * ;"),
        );

        let result = get_completions("my", false, &words, Some(&index), None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
            assert!(labels.contains(&"myword".to_string()));
            assert!(labels.contains(&"mytest".to_string()));
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_user_defined_overrides_builtin() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();

        // Define a word that exists in built-ins
        index.update_file(&file_path, &Rope::from_str(": DUP 1 + ;"));

        let result = get_completions("du", false, &words, Some(&index), None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            let dup_items: Vec<&CompletionItem> = items
                .iter()
                .filter(|i| i.label.to_lowercase() == "dup")
                .collect();
            // Should only have one dup (user-defined takes precedence over built-in DUP)
            assert_eq!(dup_items.len(), 1);
            // User-defined should be marked differently
            let dup = dup_items[0];
            assert!(dup.detail.is_some());
            assert!(dup.detail.as_ref().unwrap().contains("user-defined"));
        } else {
            panic!("Expected completion array");
        }
    }

    #[test]
    fn test_completion_mixed_builtin_and_user() {
        use crate::utils::definition_index::DefinitionIndex;
        use ropey::Rope;
        use std::env;

        let words = Words::default();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("user.forth").to_string_lossy().to_string();

        index.update_file(&file_path, &Rope::from_str(": myword 1 + ;"));

        let result = get_completions("+", false, &words, Some(&index), None);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
            // Should have built-in + operators
            assert!(labels.contains(&"+".to_string()));
            // Should NOT have myword (doesn't start with +)
            assert!(!labels.contains(&"myword".to_string()));
        } else {
            panic!("Expected completion array");
        }
    }
}
