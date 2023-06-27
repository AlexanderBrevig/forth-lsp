#[allow(unused_imports)]
use crate::prelude::*;
use crate::{
    utils::{
        ropey::{get_ix::GetIx, word_at::WordAt, RopeSliceIsLower},
        HashMapGetForLSPParams,
    },
    words::Words,
};

use std::collections::HashMap;

use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{request::Completion, CompletionItem, CompletionResponse};
use ropey::Rope;

use super::cast;

pub fn handle_completion(
    req: &Request,
    connection: &Connection,
    data: &Words,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    match cast::<Completion>(req.clone()) {
        Ok((id, params)) => {
            eprintln!("#{id}: {params:?}");
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
            if let Some(char_at_ix) = rope.get_char(ix) {
                if char_at_ix.is_whitespace() && ix > 0 {
                    ix -= 1;
                }
            }
            let word = rope.word_at(ix);
            let result = if word.len_chars() > 0 {
                eprintln!("Found word {}", word);
                let use_lower = word.is_lowercase();
                let mut ret = vec![];
                let candidates = data.words.iter().filter(|x| {
                    x.token
                        .to_lowercase()
                        .starts_with(word.to_string().to_lowercase().as_str())
                });
                for candidate in candidates {
                    let label = candidate.token.to_owned();
                    let label = if use_lower {
                        label.to_lowercase()
                    } else {
                        label
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
            };
            let result = serde_json::to_value(result)
                .expect("Must be able to serialize the CompletionResponse");
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
    }
}
