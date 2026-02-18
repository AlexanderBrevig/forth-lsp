mod config;
mod error;
mod formatter;
mod prelude;
mod utils;
mod words;

use crate::config::Config;
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::handlers::notification_did_change::handle_did_change_text_document;
use crate::utils::handlers::notification_did_open::handle_did_open_text_document;
use crate::utils::handlers::notification_did_save::handle_did_save_text_document;
use crate::utils::handlers::request_completion::handle_completion;
use crate::utils::handlers::request_document_symbols::handle_document_symbols;
use crate::utils::handlers::request_find_references::handle_find_references;
use crate::utils::handlers::request_formatting::handle_formatting;
use crate::utils::handlers::request_goto_definition::handle_goto_definition;
use crate::utils::handlers::request_hover::handle_hover;
use crate::utils::handlers::request_prepare_rename::handle_prepare_rename;
use crate::utils::handlers::request_rename::handle_rename;
use crate::utils::handlers::request_semantic_tokens::handle_semantic_tokens_full;
use crate::utils::handlers::request_signature_help::handle_signature_help;
use crate::utils::handlers::request_workspace_symbols::handle_workspace_symbols;
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

    // Load configuration from workspace root
    let workspace_root = init
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .map(|folder| folder.uri.path().as_str());
    let config = Config::load_from_workspace(workspace_root);

    if let Some(roots) = init.workspace_folders {
        eprintln!("Root: {:?}", roots);
        for root in roots {
            load_dir(root.uri.path().as_str(), &mut files)?;
        }
    }

    // Build initial definition index from loaded files
    let mut def_index = DefinitionIndex::new();
    for (path, rope) in &files {
        def_index.update_file(path, rope);
    }
    eprintln!("Indexed {} files", files.len());

    let data = Words::default();
    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", request.method);
                if handle_hover(&request, &connection, &data, &mut files, &def_index).is_ok() {
                    continue;
                }
                if handle_completion(&request, &connection, &data, &mut files, &def_index).is_ok() {
                    continue;
                }
                if handle_goto_definition(&request, &connection, &data, &mut files, &def_index)
                    .is_ok()
                {
                    continue;
                }
                if handle_find_references(&request, &connection, &mut files, &def_index).is_ok() {
                    continue;
                }
                if handle_prepare_rename(&request, &connection, &files, &def_index).is_ok() {
                    continue;
                }
                if handle_rename(&request, &connection, &mut files, &def_index).is_ok() {
                    continue;
                }
                if handle_signature_help(&request, &connection, &mut files, &data, &def_index)
                    .is_ok()
                {
                    continue;
                }
                if handle_document_symbols(&request, &connection, &mut files).is_ok() {
                    continue;
                }
                if handle_workspace_symbols(&request, &connection, &def_index).is_ok() {
                    continue;
                }
                if handle_formatting(&request, &connection, &files, &config).is_ok() {
                    continue;
                }
                if handle_semantic_tokens_full(&request, &connection, &mut files, &data).is_ok() {
                    continue;
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(notification) => {
                eprintln!("got notification: {:?}", notification.method);
                if handle_did_open_text_document(
                    &notification,
                    &connection,
                    &mut files,
                    &mut def_index,
                    &data,
                )
                .is_ok()
                {
                    continue;
                }
                if handle_did_change_text_document(
                    &notification,
                    &connection,
                    &mut files,
                    &mut def_index,
                    &data,
                )
                .is_ok()
                {
                    continue;
                }
                if handle_did_save_text_document(
                    &notification,
                    &connection,
                    &mut files,
                    &mut def_index,
                    &data,
                )
                .is_ok()
                {
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
                    // Convert path to URI to match DidOpen/DidChange format
                    let file_uri = format!("file://{}", entry);
                    files.insert(file_uri, rope);
                }
            }
        }
    }
    Ok(())
}
