mod words;
use std::collections::HashMap;
use std::error::Error;

use lsp_types::request::{Completion, HoverRequest};
use lsp_types::{
    request::GotoDefinition, GotoDefinitionResponse, InitializeParams, ServerCapabilities,
};
use lsp_types::{
    CompletionItem, CompletionResponse, Hover, Location, OneOf, Position, Range,
    TextDocumentSyncKind, Url,
};

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use ropey::{Rope, RopeSlice};

use crate::words::{Word, Words};

trait WordAtChar {
    fn word_at_char(&self, char: usize) -> RopeSlice;
}
impl WordAtChar for Rope {
    fn word_at_char(&self, chix: usize) -> RopeSlice {
        let mut min = chix;
        while min > 0 && min < self.len_chars() && !self.char(min - 1).is_whitespace() {
            min -= 1;
        }
        let mut max = chix;
        while max < self.len_chars() && !self.char(max + 1).is_whitespace() {
            max += 1;
        }
        max += 1;
        self.slice(min..max)
    }
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        completion_provider: Some(lsp_types::CompletionOptions::default()),
        ..Default::default()
    })
    .expect("Must be able to serialize the ServerCapabilities");
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Starting main loop");
    let _params: InitializeParams =
        serde_json::from_value(params).expect("Must be able to deserialize the InitializeParams");
    let mut files = HashMap::<String, Rope>::new();
    let data = Words::default();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", req.method);
                match cast::<Completion>(req.clone()) {
                    Ok((id, params)) => {
                        eprintln!("#{id}: {params:?}");
                        let rope = files
                            .get_mut(&params.text_document_position.text_document.uri.to_string())
                            .expect("Must be able to get rope for lang");
                        let mut ix = rope
                            .line_to_char(params.text_document_position.position.line as usize)
                            + params.text_document_position.position.character as usize;
                        if ix >= rope.len_chars() {
                            return Err(format!("OUT OF BOUNDS! ix {}", ix).into());
                        }
                        if let Some(char_at_ix) = rope.get_char(ix) {
                            if char_at_ix.is_whitespace() {
                                eprintln!("Found space, moving back");
                                // We are currently typing a word, and we're now on a space
                                ix -= 1;
                            } else {
                                eprintln!("Not on space");
                            }
                        }
                        let word = rope.word_at_char(ix);
                        eprintln!("Found word {}", word);
                        let use_lower = if let Some(chr) = word.get_char(0) {
                            chr.is_lowercase()
                        } else {
                            false
                        };
                        let result = if word.len_chars() > 0 {
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
                                            value: candidate.help.to_owned(),
                                        },
                                    )),
                                    ..Default::default()
                                });
                            }
                            Some(CompletionResponse::Array(ret))
                        } else {
                            None
                        };
                        let result = serde_json::to_value(&result)
                            .expect("Must be able to serialize the CompletionResponse");
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                match cast::<HoverRequest>(req.clone()) {
                    Ok((id, params)) => {
                        eprintln!("#{id}: {params:?}");
                        let rope = files
                            .get_mut(
                                &params
                                    .text_document_position_params
                                    .text_document
                                    .uri
                                    .to_string(),
                            )
                            .expect("Must be able to get rope for lang");
                        let ix = rope.line_to_char(
                            params.text_document_position_params.position.line as usize,
                        ) + params.text_document_position_params.position.character
                            as usize;
                        let word = word_on_and_before_cursor(rope, ix);
                        let result = if word.len() > 0 {
                            let default_info = &Word::default();
                            let info = data
                                .words
                                .iter()
                                .filter(|x| {
                                    x.token
                                        .to_lowercase()
                                        .starts_with(word.to_string().to_lowercase().as_str())
                                })
                                .nth(0)
                                .unwrap_or(&default_info);
                            Some(Hover {
                                contents: lsp_types::HoverContents::Markup(
                                    lsp_types::MarkupContent {
                                        kind: lsp_types::MarkupKind::Markdown,
                                        value: format!(
                                            "# `{}`   `{}`\n\n{}",
                                            info.token, info.stack, info.help
                                        ),
                                    },
                                ),
                                range: None,
                            })
                        } else {
                            None
                        };
                        let result = serde_json::to_value(&result)
                            .expect("Must be able to serialize the Hover");
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        eprintln!("#{id}: {params:?}");
                        //TODO: recurse parse follow includes /^[iI][nN][cC][lL][uU][dD][eE] +(.*\..*)/
                        //TODO: find colon defines /: +(?:.|\n)*;/
                        //TODO: generate Vec<Location> from line num and col from the matching <file(s)>
                        let rope = files
                            .get_mut(
                                &params
                                    .text_document_position_params
                                    .text_document
                                    .uri
                                    .to_string(),
                            )
                            .expect("Must be able to get rope for lang");
                        let ix = rope.line_to_char(
                            params.text_document_position_params.position.line as usize,
                        ) + params.text_document_position_params.position.character
                            as usize;
                        let word = word_on_and_before_cursor(rope, ix);

                        let mut cur_index = 0;
                        let mut defn_index: i64 = -1;
                        let mut defn = String::new();
                        let mut ret: Vec<Location> = vec![];
                        let mut chars_iter = rope.chars();
                        while let Some(next_char) = chars_iter.next() {
                            cur_index += 1;
                            if next_char.is_whitespace() {
                                if defn.len() > 0 {
                                    if defn.trim() == word {
                                        //TODO: replace parse input with loop
                                        if let Ok(uri) = Url::parse(
                                            params
                                                .text_document_position_params
                                                .text_document
                                                .uri
                                                .as_str(),
                                        ) {
                                            let start_line_nr =
                                                rope.char_to_line(defn_index as usize);
                                            let start_line_index = defn_index as u32
                                                - rope.line_to_char(start_line_nr) as u32;
                                            let defn_len = rope
                                                .chars()
                                                .skip(defn_index as usize)
                                                .take_while(|x| *x != ';')
                                                .count()
                                                + 1;
                                            let end_defn_index = defn_index as usize + defn_len;
                                            let end_line_nr = rope.char_to_line(end_defn_index - 1);
                                            let end_line_index =
                                                end_defn_index - rope.line_to_char(end_line_nr);
                                            ret.push(Location {
                                                uri,
                                                range: Range {
                                                    start: Position {
                                                        line: start_line_nr as u32,
                                                        character: start_line_index,
                                                    },
                                                    end: Position {
                                                        line: end_line_nr as u32,
                                                        character: end_line_index as u32,
                                                    },
                                                },
                                            });
                                        }
                                    }
                                    defn_index = -1;
                                }
                                defn.clear();
                            } else if next_char == ':' {
                                defn_index = (cur_index - 1) as i64;
                            } else if defn_index != -1 && !next_char.is_whitespace() {
                                defn.push(next_char);
                            }
                        }
                        let result = Some(GotoDefinitionResponse::Array(ret));
                        let result = serde_json::to_value(&result)
                            .expect("Must be able to serialize the GotoDefinitionResponse");
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                // ...
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(not) => {
                eprintln!("got notification: {not:?}");
                match cast_notification::<lsp_types::notification::DidOpenTextDocument>(not.clone())
                {
                    Ok(params) => {
                        let rope = Rope::from_str(params.text_document.text.as_str());
                        files.insert(params.text_document.uri.to_string(), rope);
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };
                match cast_notification::<lsp_types::notification::DidChangeTextDocument>(
                    not.clone(),
                ) {
                    Ok(params) => {
                        let rope = files
                            .get_mut(&params.text_document.uri.to_string())
                            .expect("Must be able to get rope for lang");
                        for change in params.content_changes {
                            let range = change.range.unwrap_or_default();
                            let start = rope.line_to_char(range.start.line as usize)
                                + range.start.character as usize;
                            let end = rope.line_to_char(range.end.line as usize)
                                + range.end.character as usize;
                            rope.remove(start..end);
                            rope.insert(start, change.text.as_str());
                        }
                    }
                    Err(_) => todo!(),
                }
            }
        }
    }
    Ok(())
}

fn word_on_and_before_cursor(rope: &mut Rope, ix: usize) -> String {
    let word_on_cursor = rope.word_at_char(ix).to_string().trim().to_string();
    // with helix, you typically end up with having a selected word including the previous space
    // this means we should also look for a word behind the cursor
    //TODO: make look-behind cleaner
    let word_behind_cursor = rope.word_at_char(ix - 1).to_string().trim().to_string();
    let word = if word_on_cursor.len() >= word_behind_cursor.len() {
        word_on_cursor
    } else {
        word_behind_cursor
    };
    word
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(req: Notification) -> Result<N::Params, ExtractError<Notification>>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    req.extract(N::METHOD)
}
