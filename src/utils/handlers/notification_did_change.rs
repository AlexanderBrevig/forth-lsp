#[allow(unused_imports)]
use crate::prelude::*;

use std::collections::HashMap;

use lsp_server::Notification;
use ropey::Rope;

use super::cast_notification;

pub fn handle_did_change_text_document(
    notification: &Notification,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    match cast_notification::<lsp_types::notification::DidChangeTextDocument>(notification.clone())
    {
        Ok(params) => {
            let rope = files
                .get_mut(&params.text_document.uri.to_string())
                .expect("Must be able to get rope for lang");
            for change in params.content_changes {
                let range = change.range.unwrap_or_default();
                let start =
                    rope.line_to_char(range.start.line as usize) + range.start.character as usize;
                let end = rope.line_to_char(range.end.line as usize) + range.end.character as usize;
                rope.remove(start..end);
                rope.insert(start, change.text.as_str());
            }
            Ok(())
        }
        Err(_) => todo!(),
    }
}
