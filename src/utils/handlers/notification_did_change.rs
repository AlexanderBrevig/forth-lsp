#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::diagnostics::{get_diagnostics, publish_diagnostics};
use crate::words::Words;

use std::collections::HashMap;

use lsp_server::{Connection, Notification};
use ropey::Rope;

use super::cast_notification;

pub fn handle_did_change_text_document(
    notification: &Notification,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    def_index: &mut DefinitionIndex,
    builtin_words: &Words,
) -> Result<()> {
    match cast_notification::<lsp_types::notification::DidChangeTextDocument>(notification.clone())
    {
        Ok(params) => {
            let file_uri = params.text_document.uri.to_string();
            let rope = files
                .get_mut(&file_uri)
                .expect("Must be able to get rope for lang");
            for change in params.content_changes {
                let range = change.range.unwrap_or_default();
                let start =
                    rope.line_to_char(range.start.line as usize) + range.start.character as usize;
                let end = rope.line_to_char(range.end.line as usize) + range.end.character as usize;
                rope.remove(start..end);
                rope.insert(start, change.text.as_str());
            }
            // Update definition index for the changed file
            def_index.update_file(&file_uri, rope);

            // Publish diagnostics for the changed file
            let diagnostics = get_diagnostics(rope, def_index, builtin_words);
            publish_diagnostics(
                connection,
                params.text_document.uri.clone(),
                diagnostics,
                params.text_document.version,
            )?;

            Ok(())
        }
        Err(err) => {
            log_handler_error!("Did change notification", err);
            Err(err)
        }
    }
}
