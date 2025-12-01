#[allow(unused_imports)]
use crate::prelude::*;
use crate::{config::Config, formatter::Formatter, utils::handlers::send_response};

use std::collections::HashMap;

use lsp_server::{Connection, Request};
use lsp_types::{DocumentFormattingParams, TextEdit, request::Formatting};
use ropey::Rope;

use super::cast;

/// Handle textDocument/formatting request
pub fn handle_formatting(
    request: &Request,
    connection: &Connection,
    files: &HashMap<String, Rope>,
    config: &Config,
) -> Result<()> {
    match cast::<Formatting>(request.clone()) {
        Ok((id, params)) => {
            eprintln!(
                "Formatting request for: {}",
                params.text_document.uri.path()
            );

            let result = get_formatting_edits(&params, files, config);
            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => panic!("{err:?}"),
    }
}

/// Get formatting text edits for a document
fn get_formatting_edits(
    params: &DocumentFormattingParams,
    files: &HashMap<String, Rope>,
    config: &Config,
) -> Option<Vec<TextEdit>> {
    let file_uri = params.text_document.uri.to_string();
    let rope = files.get(&file_uri)?;

    let formatter = Formatter::new(config.format.clone());
    formatter.format_document(rope).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FormatConfig;
    use lsp_types::TextDocumentIdentifier;

    #[test]
    fn test_get_formatting_edits() {
        let mut files = HashMap::new();
        let source = ":   square   dup   *   ;";
        let rope = Rope::from_str(source);
        files.insert("file:///test.forth".to_string(), rope);

        let config = Config::default();
        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: "file:///test.forth".parse().unwrap(),
            },
            options: Default::default(),
            work_done_progress_params: Default::default(),
        };

        let edits = get_formatting_edits(&params, &files, &config);
        assert!(edits.is_some());

        let edits = edits.unwrap();
        assert_eq!(edits.len(), 1);
        assert!(edits[0].new_text.contains("square"));
    }

    #[test]
    fn test_formatting_with_indent_disabled() {
        let mut files = HashMap::new();
        let source = ": add + ;";
        let rope = Rope::from_str(source);
        files.insert("file:///test.forth".to_string(), rope);

        let config = Config {
            format: FormatConfig {
                indent_control_structures: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: "file:///test.forth".parse().unwrap(),
            },
            options: Default::default(),
            work_done_progress_params: Default::default(),
        };

        let edits = get_formatting_edits(&params, &files, &config);
        assert!(edits.is_some());

        let edits = edits.unwrap();
        assert_eq!(edits.len(), 1);
        // With indent disabled, everything should be on one line
        assert_eq!(edits[0].new_text, ": add + ;\n");
    }

    #[test]
    fn test_formatting_returns_none_for_missing_file() {
        let files = HashMap::new();
        let config = Config::default();

        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: "file:///missing.forth".parse().unwrap(),
            },
            options: Default::default(),
            work_done_progress_params: Default::default(),
        };

        let edits = get_formatting_edits(&params, &files, &config);
        assert!(edits.is_none());
    }
}
