#[allow(unused_imports)]
use crate::prelude::*;
use crate::{
    utils::{
        ropey::{get_ix::GetIx, word_on_or_before::WordOnOrBefore},
        HashMapGetForLSPParams,
    },
    words::{Word, Words},
};

use std::collections::HashMap;

use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{request::HoverRequest, Hover};
use ropey::Rope;

use super::cast;

pub fn handle_hover(
    req: &Request,
    connection: &Connection,
    data: &Words,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    match cast::<HoverRequest>(req.clone()) {
        Ok((id, params)) => {
            eprintln!("#{id}: {params:?}");
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
            let result = if !word.len_chars() > 0 {
                let default_info = &Word::default();
                let info = data
                    .words
                    .iter()
                    .find(|x| x.token.to_lowercase() == (word.to_string().to_lowercase().as_str()))
                    .unwrap_or(&default_info);
                Some(Hover {
                    contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: info.documentation(),
                    }),
                    range: None,
                })
            } else {
                None
            };
            let result = serde_json::to_value(result).expect("Must be able to serialize the Hover");
            let resp = Response {
                id,
                result: Some(result),
                error: None,
            };
            connection
                .sender
                .send(Message::Response(resp))
                .map_err(|err| Error::SendError(err.to_string()))?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => panic!("{err:?}"),
        // Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        // Err(ExtractError::MethodMismatch(req)) => req,
    }
}
