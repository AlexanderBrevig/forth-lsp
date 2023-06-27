#[allow(unused_imports)]
use crate::prelude::*;

use std::collections::HashMap;

use lsp_server::Notification;
use ropey::Rope;

use super::cast_notification;

pub fn handle_did_open_text_document(
    notification: &Notification,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    match cast_notification::<lsp_types::notification::DidOpenTextDocument>(notification.clone()) {
        Ok(params) => {
            if let std::collections::hash_map::Entry::Vacant(e) =
                files.entry(params.text_document.uri.to_string())
            {
                let rope = Rope::from_str(params.text_document.text.as_str());
                e.insert(rope);
            }
            Ok(())
        }
        Err(Error::ExtractNotificationError(req)) => Err(Error::ExtractNotificationError(req)),
        Err(err) => panic!("{err:?}"),
    }
}
