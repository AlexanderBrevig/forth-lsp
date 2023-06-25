mod error;
mod prelude;
mod utils;
mod words;

use crate::prelude::*;
use crate::utils::ropey_get_ix::GetIx;
use crate::utils::ropey_word_at_char::WordAtChar;
use crate::words::{Word, Words};

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use forth_lexer::parser::Lexer;
use lsp_types::request::{Completion, HoverRequest};
use lsp_types::{
    request::GotoDefinition, GotoDefinitionResponse, InitializeParams, ServerCapabilities,
};
use lsp_types::{
    CompletionItem, CompletionResponse, Hover, Location, OneOf, Position, Range,
    TextDocumentSyncKind, Url,
};

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use ropey::Rope;

fn main() -> Result<()> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        // workspace_symbol_provider
        workspace: Some(lsp_types::WorkspaceServerCapabilities {
            workspace_folders: Some(lsp_types::WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(false)),
            }),
            file_operations: None,
        }),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        completion_provider: Some(lsp_types::CompletionOptions::default()),
        ..Default::default()
    })?;
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    eprintln!("Starting main loop");
    let mut files = HashMap::<String, Rope>::new();
    let init: InitializeParams = serde_json::from_value(params)?;
    if let Some(roots) = init.workspace_folders {
        eprintln!("Root: {:?}", roots);
        for root in roots {
            load_dir(root.uri.path(), &mut files)?;
        }
    }
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
                        let mut ix = rope.get_ix(&params);
                        if ix >= rope.len_chars() {
                            return Err(Error::OutOfBounds(ix));
                        }
                        if let Some(char_at_ix) = rope.get_char(ix) {
                            if char_at_ix.is_whitespace() && ix > 0 {
                                ix -= 1;
                            }
                        }
                        let word = rope.word_at_char(ix);
                        eprintln!("Found word {}", word);
                        let use_lower = if let Some(chr) = word.get_char(word.len_chars() - 1) {
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
                        connection
                            .sender
                            .send(Message::Response(resp))
                            .map_err(|err| Error::SendError(err.to_string()))?;
                        continue;
                    }
                    Err(Error::ExtractRequestError(req)) => req,
                    Err(err) => panic!("{err:?}"),
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
                        let ix = rope.get_ix(&params);
                        if ix >= rope.len_chars() {
                            return Err(Error::OutOfBounds(ix));
                        }
                        let word = word_on_or_before_cursor(rope, ix);
                        let result = if !word.is_empty() {
                            let default_info = &Word::default();
                            let info = data
                                .words
                                .iter()
                                .find(|x| {
                                    x.token.to_lowercase()
                                        == (word.to_string().to_lowercase().as_str())
                                })
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
                        connection
                            .sender
                            .send(Message::Response(resp))
                            .map_err(|err| Error::SendError(err.to_string()))?;
                        continue;
                    }
                    Err(Error::ExtractRequestError(req)) => req,
                    Err(err) => panic!("{err:?}"),
                    // Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    // Err(ExtractError::MethodMismatch(req)) => req,
                };
                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        eprintln!("#{id}: {params:?}");
                        //TODO: recurse parse follow includes /^[iI][nN][cC][lL][uU][dD][eE] +(.*\..*)/
                        //TODO: find colon defines /: +(?:.|\n)*;/
                        //TODO: generate Vec<Location> from line num and col from the matching <file(s)>
                        let mut ret: Vec<Location> = vec![];
                        eprintln!(
                            "{:?}",
                            &params.text_document_position_params.text_document.uri
                        );
                        let rope = files
                            .get_mut(
                                &params
                                    .text_document_position_params
                                    .text_document
                                    .uri
                                    .to_string(),
                            )
                            .expect("Must be able to get rope for lang");
                        let ix = rope.get_ix(&params);
                        if ix >= rope.len_chars() {
                            eprintln!(
                                "IX OUT OF BOUNDS {} {} {}",
                                ix,
                                rope.len_chars(),
                                rope.len_bytes()
                            );
                            break;
                        }
                        let word = word_on_or_before_cursor(rope, ix);
                        for (file, rope) in files.iter() {
                            eprintln!("Word: {}", word);
                            let progn = rope.to_string();
                            let mut lexer = Lexer::new(progn.as_str());
                            let tokens = lexer.parse();
                            let bind1 = tokens.clone();
                            let mut start_line = 0u32;
                            let mut start_char = 0u32;
                            let mut end_line = 0u32;
                            let mut end_char = 0u32;
                            let mut found_defn = false;
                            for (x, y) in tokens.into_iter().zip(bind1.iter().skip(1)) {
                                if let forth_lexer::token::Token::Colon(x_dat) = x {
                                    if let forth_lexer::token::Token::Word(y_dat) = y {
                                        if y_dat.value.eq_ignore_ascii_case(word.as_str()) {
                                            eprintln!("Found word defn {:?}", y_dat);
                                            start_line = rope.char_to_line(x_dat.start) as u32;
                                            start_char = (x_dat.start
                                                - rope.line_to_char(start_line as usize))
                                                as u32;
                                            found_defn = true;
                                        } else {
                                            found_defn = false;
                                        }
                                    }
                                }
                                if let forth_lexer::token::Token::Semicolon(y_dat) = y {
                                    if found_defn {
                                        eprintln!("found end {:?}", y_dat);
                                        end_line = rope.char_to_line(y_dat.end) as u32;
                                        end_char = (y_dat.end
                                            - rope.line_to_char(end_line as usize))
                                            as u32;
                                        break;
                                    }
                                }
                            }
                            eprintln!("GOT HERE");
                            if (start_line, start_char) != (end_line, end_char) {
                                eprintln!(
                                    "{} {} {} {}",
                                    start_line, start_char, end_line, end_char
                                );
                                if let Ok(uri) = Url::from_file_path(file) {
                                    ret.push(Location {
                                        uri,
                                        range: Range {
                                            start: Position {
                                                line: start_line,
                                                character: start_char,
                                            },
                                            end: Position {
                                                line: end_line,
                                                character: end_char,
                                            },
                                        },
                                    });
                                } else {
                                    eprintln!("Failed to parse URI for {}", file);
                                }
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
                        connection
                            .sender
                            .send(Message::Response(resp))
                            .map_err(|err| Error::SendError(err.to_string()))?;
                        continue;
                    }
                    Err(Error::ExtractRequestError(req)) => req,
                    Err(err) => panic!("{err:?}"),
                    // Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    // Err(ExtractError::MethodMismatch(req)) => req,
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
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            files.entry(params.text_document.uri.to_string())
                        {
                            let rope = Rope::from_str(params.text_document.text.as_str());
                            e.insert(rope);
                        }
                        continue;
                    }
                    Err(Error::ExtractNotificationError(req)) => req,
                    Err(err) => panic!("{err:?}"),
                    // Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    // Err(ExtractError::MethodMismatch(not)) => not,
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

fn load_dir(
    root: &str, //lsp_types::WorkspaceFolder,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    if let Ok(paths) = fs::read_dir(root) {
        for path in paths {
            if let Some(entry) = path?.path().to_str() {
                if fs::metadata(entry)?.is_dir() {
                    load_dir(entry, files)?;
                } else if Path::new(entry).extension().and_then(OsStr::to_str) == Some("forth") {
                    eprintln!("FORTH load {}", entry);
                    let raw_content = fs::read(entry)?;
                    let content = String::from_utf8_lossy(&raw_content);
                    let rope = Rope::from_str(&content);
                    files.insert(entry.to_string(), rope);
                }
            }
        }
    }
    Ok(())
}

fn word_on_or_before_cursor(rope: &Rope, ix: usize) -> String {
    let word_on_cursor = rope.word_at_char(ix);
    // with helix, you typically end up with having a selected word including the previous space
    // this means we should also look for a word behind the cursor
    //TODO: make look-behind cleaner
    let word_behind_cursor = if ix > 0 {
        rope.word_at_char(ix - 1)
    } else {
        word_on_cursor
    };
    eprintln!(
        "Word on `{}` before `{}`",
        word_on_cursor, word_behind_cursor
    );
    let word = if word_on_cursor.len_chars() >= word_behind_cursor.len_chars() {
        word_on_cursor
    } else {
        word_behind_cursor
    };
    word.to_string()
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params)>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
        .map_err(error::Error::ExtractRequestError)
}

fn cast_notification<N>(req: Notification) -> Result<N::Params>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    req.extract(N::METHOD)
        .map_err(error::Error::ExtractNotificationError)
}
