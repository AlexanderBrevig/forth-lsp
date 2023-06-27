mod error;
mod prelude;
mod utils;
mod words;

use crate::prelude::*;
use crate::utils::handlers::notification_did_change::handle_did_change_text_document;
use crate::utils::handlers::notification_did_open::handle_did_open_text_document;
use crate::utils::handlers::request_completion::handle_completion;
use crate::utils::handlers::request_goto_definition::handle_goto_definition;
use crate::utils::handlers::request_hover::handle_hover;
use crate::utils::server_capabilities::forth_lsp_capabilities;
use crate::words::Words;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use lsp_server::{Connection, Message};
use lsp_types::InitializeParams;

use ropey::Rope;

fn main() -> Result<()> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(forth_lsp_capabilities())?;
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    eprintln!("Starting main loop");
    let init: InitializeParams = serde_json::from_value(params)?;
    let mut files = HashMap::<String, Rope>::new();
    if let Some(roots) = init.workspace_folders {
        eprintln!("Root: {:?}", roots);
        for root in roots {
            load_dir(root.uri.path(), &mut files)?;
        }
    }
    let data = Words::default();
    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", request.method);
                if handle_hover(&request, &connection, &data, &mut files).is_ok() {
                    continue;
                }
                if handle_completion(&request, &connection, &data, &mut files).is_ok() {
                    continue;
                }
                if handle_goto_definition(&request, &connection, &data, &mut files).is_ok() {
                    continue;
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(notification) => {
                eprintln!("got notification: {:?}", notification.method);
                if handle_did_open_text_document(&notification, &mut files).is_ok() {
                    continue;
                }
                if handle_did_change_text_document(&notification, &mut files).is_ok() {
                    continue;
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
