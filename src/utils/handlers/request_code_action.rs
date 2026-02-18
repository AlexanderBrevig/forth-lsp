#![allow(clippy::mutable_key_type)]
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::handlers::{cast, send_response};
use crate::words::Words;

use std::collections::HashMap;

use lsp_server::{Connection, Request};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, NumberOrString, Position,
    Range, TextEdit, Uri, WorkspaceEdit, request::CodeActionRequest,
};
use ropey::Rope;

/// Levenshtein edit distance between two strings (case-insensitive)
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();
    let m = a.len();
    let n = b.len();

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Find similar words from builtin and user-defined words
fn find_similar_words(
    word: &str,
    builtin_words: &Words,
    def_index: &DefinitionIndex,
    max_results: usize,
) -> Vec<String> {
    let threshold = if word.len() <= 5 { 2 } else { 3 };

    let mut candidates: Vec<(String, usize)> = Vec::new();

    // Check builtin words
    for w in &builtin_words.words {
        let dist = edit_distance(word, w.token);
        if dist > 0 && dist <= threshold {
            candidates.push((w.token.to_string(), dist));
        }
    }

    // Check user-defined words
    for w in def_index.all_words() {
        let dist = edit_distance(word, &w);
        if dist > 0 && dist <= threshold {
            candidates.push((w, dist));
        }
    }

    candidates.sort_by_key(|(_, d)| *d);
    candidates.dedup_by(|a, b| a.0.to_lowercase() == b.0.to_lowercase());
    candidates.truncate(max_results);
    candidates.into_iter().map(|(w, _)| w).collect()
}

/// Generate code actions for the given context
pub fn get_code_actions(
    rope: &Rope,
    uri: &Uri,
    range: &Range,
    context: &lsp_types::CodeActionContext,
    builtin_words: &Words,
    def_index: &DefinitionIndex,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // 1. "Did you mean X?" for undefined word diagnostics
    for diag in &context.diagnostics {
        if let Some(NumberOrString::String(code)) = &diag.code {
            match code.as_str() {
                "undefined-word" => {
                    if let Some(word) = diag.message.strip_prefix("Undefined word: ") {
                        let suggestions = find_similar_words(word, builtin_words, def_index, 3);
                        for suggestion in suggestions {
                            let mut changes = HashMap::new();
                            changes.insert(
                                uri.clone(),
                                vec![TextEdit {
                                    range: diag.range,
                                    new_text: suggestion.clone(),
                                }],
                            );
                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!("Did you mean `{}`?", suggestion),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diag.clone()]),
                                edit: Some(WorkspaceEdit {
                                    changes: Some(changes),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }));
                        }
                    }
                }
                "unclosed-definition" => {
                    // Parse insertion position from diagnostic data
                    if let Some(data) = &diag.data
                        && let (Some(line), Some(character)) = (
                            data.get("insert_line").and_then(|v| v.as_u64()),
                            data.get("insert_character").and_then(|v| v.as_u64()),
                        )
                    {
                        let insert_pos = Position {
                            line: line as u32,
                            character: character as u32,
                        };
                        let mut changes = HashMap::new();
                        changes.insert(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: insert_pos,
                                    end: insert_pos,
                                },
                                new_text: " ;".to_string(),
                            }],
                        );
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Insert missing `;`".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diag.clone()]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }));
                    }
                }
                _ => {}
            }
        }
    }

    // 3. Extract selection into a new word
    if range.start != range.end {
        let start_line = range.start.line as usize;
        let start_char = range.start.character as usize;
        let end_line = range.end.line as usize;
        let end_char = range.end.character as usize;

        // Extract selected text
        let start_idx = rope.line_to_char(start_line) + start_char;
        let end_idx = rope.line_to_char(end_line) + end_char;

        if start_idx < end_idx && end_idx <= rope.len_chars() {
            let selected_text: String = rope.slice(start_idx..end_idx).to_string();
            let trimmed = selected_text.trim();
            if !trimmed.is_empty() {
                let new_def = format!(": extracted {} ;\n", trimmed);

                // Insert the new definition before the line containing the selection start
                let insert_pos = Position {
                    line: range.start.line,
                    character: 0,
                };

                let mut changes = HashMap::new();
                changes.insert(
                    uri.clone(),
                    vec![
                        // Insert new definition before the current line
                        TextEdit {
                            range: Range {
                                start: insert_pos,
                                end: insert_pos,
                            },
                            new_text: new_def,
                        },
                        // Replace selection with the new word name
                        TextEdit {
                            range: *range,
                            new_text: "extracted".to_string(),
                        },
                    ],
                );

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Extract into new word".to_string(),
                    kind: Some(CodeActionKind::REFACTOR_EXTRACT),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }
    }

    actions
}

pub fn handle_code_action(
    req: &Request,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    builtin_words: &Words,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<CodeActionRequest>(req.clone()) {
        Ok((id, params)) => {
            let CodeActionParams {
                text_document,
                range,
                context,
                ..
            } = params;

            let uri = text_document.uri;
            let rope = files
                .get(&uri.to_string())
                .ok_or_else(|| Error::NoSuchFile(uri.to_string()))?;

            let actions = get_code_actions(rope, &uri, &range, &context, builtin_words, def_index);

            let result: Option<Vec<CodeActionOrCommand>> = if actions.is_empty() {
                Some(vec![])
            } else {
                Some(actions)
            };

            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => panic!("{err:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::definition_index::DefinitionIndex;
    use crate::words::Words;
    use lsp_types::{CodeActionContext, Diagnostic, DiagnosticSeverity};
    use ropey::Rope;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("DUP", "DUU"), 1);
        assert_eq!(edit_distance("SWAP", "SWAT"), 1);
        assert_eq!(edit_distance("abc", "xyz"), 3);
        assert_eq!(edit_distance("DUP", "DUP"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
    }

    #[test]
    fn test_find_similar_words() {
        let words = Words::default();
        let index = DefinitionIndex::new();

        let results = find_similar_words("DUU", &words, &index, 3);
        assert!(
            results.iter().any(|w| w.eq_ignore_ascii_case("DUP")),
            "Expected DUP in results: {:?}",
            results
        );

        let results = find_similar_words("SWAR", &words, &index, 3);
        assert!(
            results.iter().any(|w| w.eq_ignore_ascii_case("SWAP")),
            "Expected SWAP in results: {:?}",
            results
        );
    }

    #[test]
    fn test_code_action_did_you_mean() {
        let words = Words::default();
        let index = DefinitionIndex::new();
        let rope = Rope::from_str(": test DUU ;");
        let uri: Uri = "file:///test.forth".parse().unwrap();

        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 7,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("undefined-word".to_string())),
            message: "Undefined word: DUU".to_string(),
            source: Some("forth-lsp".to_string()),
            ..Default::default()
        };

        let context = CodeActionContext {
            diagnostics: vec![diag],
            ..Default::default()
        };

        let range = Range::default();
        let actions = get_code_actions(&rope, &uri, &range, &context, &words, &index);

        assert!(
            !actions.is_empty(),
            "Expected at least one code action for DUU"
        );
        let titles: Vec<String> = actions
            .iter()
            .filter_map(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => Some(ca.title.clone()),
                _ => None,
            })
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("DUP")),
            "Expected DUP suggestion in: {:?}",
            titles
        );
    }

    #[test]
    fn test_code_action_no_suggestion_for_very_different() {
        let words = Words::default();
        let index = DefinitionIndex::new();
        let rope = Rope::from_str(": test xyzzy123 ;");
        let uri: Uri = "file:///test.forth".parse().unwrap();

        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 7,
                },
                end: Position {
                    line: 0,
                    character: 16,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("undefined-word".to_string())),
            message: "Undefined word: xyzzy123".to_string(),
            source: Some("forth-lsp".to_string()),
            ..Default::default()
        };

        let context = CodeActionContext {
            diagnostics: vec![diag],
            ..Default::default()
        };

        let range = Range::default();
        let actions = get_code_actions(&rope, &uri, &range, &context, &words, &index);

        // Should have no quickfix actions (no close matches)
        let quickfixes: Vec<_> = actions
            .iter()
            .filter(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => ca.kind == Some(CodeActionKind::QUICKFIX),
                _ => false,
            })
            .collect();
        assert!(
            quickfixes.is_empty(),
            "Expected no suggestions for xyzzy123"
        );
    }

    #[test]
    fn test_code_action_unclosed_definition() {
        let words = Words::default();
        let index = DefinitionIndex::new();
        let rope = Rope::from_str(": foo 1 +");
        let uri: Uri = "file:///test.forth".parse().unwrap();

        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("unclosed-definition".to_string())),
            message: "Unclosed definition".to_string(),
            source: Some("forth-lsp".to_string()),
            data: Some(serde_json::json!({"insert_line": 0, "insert_character": 9})),
            ..Default::default()
        };

        let context = CodeActionContext {
            diagnostics: vec![diag],
            ..Default::default()
        };

        let range = Range::default();
        let actions = get_code_actions(&rope, &uri, &range, &context, &words, &index);

        assert!(
            !actions.is_empty(),
            "Expected code action for unclosed definition"
        );
        let titles: Vec<String> = actions
            .iter()
            .filter_map(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => Some(ca.title.clone()),
                _ => None,
            })
            .collect();
        assert!(
            titles.iter().any(|t| t.contains(";")),
            "Expected semicolon insertion in: {:?}",
            titles
        );
    }

    #[test]
    fn test_code_action_extract_word() {
        let words = Words::default();
        let index = DefinitionIndex::new();
        let rope = Rope::from_str(": test 1 + 2 * ;");
        let uri: Uri = "file:///test.forth".parse().unwrap();

        let context = CodeActionContext {
            diagnostics: vec![],
            ..Default::default()
        };

        // Select "1 +" (characters 7-10)
        let range = Range {
            start: Position {
                line: 0,
                character: 7,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };

        let actions = get_code_actions(&rope, &uri, &range, &context, &words, &index);

        let extract_actions: Vec<_> = actions
            .iter()
            .filter(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => {
                    ca.kind == Some(CodeActionKind::REFACTOR_EXTRACT)
                }
                _ => false,
            })
            .collect();

        assert_eq!(extract_actions.len(), 1, "Expected one extract action");
        if let CodeActionOrCommand::CodeAction(ca) = &extract_actions[0] {
            assert!(ca.title.contains("Extract"));
            // Verify the edit contains the new definition
            let edit = ca.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let edits = changes.get(&uri).unwrap();
            assert!(edits.iter().any(|e| e.new_text.contains(": extracted")));
            assert!(edits.iter().any(|e| e.new_text == "extracted"));
        }
    }

    #[test]
    fn test_code_action_empty_diagnostics() {
        let words = Words::default();
        let index = DefinitionIndex::new();
        let rope = Rope::from_str(": test 1 + ;");
        let uri: Uri = "file:///test.forth".parse().unwrap();

        let context = CodeActionContext {
            diagnostics: vec![],
            ..Default::default()
        };

        let range = Range::default(); // empty range
        let actions = get_code_actions(&rope, &uri, &range, &context, &words, &index);

        assert!(actions.is_empty(), "Expected no actions for empty context");
    }
}
