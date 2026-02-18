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

pub fn handle_did_open_text_document(
    notification: &Notification,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    def_index: &mut DefinitionIndex,
    builtin_words: &Words,
) -> Result<()> {
    match cast_notification::<lsp_types::notification::DidOpenTextDocument>(notification.clone()) {
        Ok(params) => {
            let file_uri = params.text_document.uri.to_string();
            log_debug!("DidOpen for file: {}", file_uri);
            if let std::collections::hash_map::Entry::Vacant(e) = files.entry(file_uri.clone()) {
                let rope = Rope::from_str(params.text_document.text.as_str());

                // Parse once and reuse tokens for both indexing and diagnostics
                let source = rope.to_string();
                let tokens = Lexer::new(&source).parse();

                def_index.update_file_from_tokens(&file_uri, &tokens, &rope);

                let diagnostics =
                    get_diagnostics_from_tokens(&tokens, &source, &rope, def_index, builtin_words);
                publish_diagnostics(
                    connection,
                    params.text_document.uri.clone(),
                    diagnostics,
                    params.text_document.version,
                )?;

                e.insert(rope);
            }
            Ok(())
        }
        Err(Error::ExtractNotificationError(req)) => Err(Error::ExtractNotificationError(req)),
        Err(err) => {
            log_handler_error!("Did open notification", err);
            Err(err)
        }
    }
}
