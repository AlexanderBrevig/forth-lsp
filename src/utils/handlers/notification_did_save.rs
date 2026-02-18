#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::diagnostics::{get_diagnostics_from_tokens, publish_diagnostics};
use crate::words::Words;

use std::collections::HashMap;

use forth_lexer::parser::Lexer;
use lsp_server::{Connection, Notification};
use ropey::Rope;

use super::cast_notification;

pub fn handle_did_save_text_document(
    notification: &Notification,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    def_index: &mut DefinitionIndex,
    builtin_words: &Words,
) -> Result<()> {
    match cast_notification::<lsp_types::notification::DidSaveTextDocument>(notification.clone()) {
        Ok(params) => {
            let file_uri = params.text_document.uri.to_string();

            // Get the rope for this file
            if let Some(rope) = files.get(&file_uri) {
                // Parse once and reuse tokens for both indexing and diagnostics
                let source = rope.to_string();
                let tokens = Lexer::new(&source).parse();

                log_debug!("Updating definition index on save for: {}", file_uri);
                def_index.update_file_from_tokens(&file_uri, &tokens, rope);

                let diagnostics =
                    get_diagnostics_from_tokens(&tokens, &source, rope, def_index, builtin_words);
                publish_diagnostics(
                    connection,
                    params.text_document.uri.clone(),
                    diagnostics,
                    0, // No version for save notifications
                )?;
            }

            Ok(())
        }
        Err(err) => {
            log_handler_error!("Did save notification", err);
            Err(err)
        }
    }
}
