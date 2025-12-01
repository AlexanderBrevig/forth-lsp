use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::handlers::{common::ExtractedPosition, send_response};

use lsp_server::{Connection, Request};
use lsp_types::{PrepareRenameResponse, Range, request::PrepareRenameRequest};
use ropey::Rope;
use std::collections::HashMap;

use super::cast;

pub fn handle_prepare_rename(
    req: &Request,
    connection: &Connection,
    files: &HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<PrepareRenameRequest>(req.clone()) {
        Ok((id, params)) => {
            log_request_msg!(
                id,
                "prepareRename at {}:{}",
                params.position.line,
                params.position.character
            );

            let pos = ExtractedPosition::from_text_document_position(&params)?;

            let response = get_prepare_rename_response(
                &pos.file_uri,
                pos.line,
                pos.character,
                files,
                def_index,
            );

            send_response(connection, id, response)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => Err(err),
    }
}

fn get_prepare_rename_response(
    file_uri: &str,
    line: u32,
    character: u32,
    files: &HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Option<PrepareRenameResponse> {
    let rope = files.get(file_uri)?;

    if line as usize >= rope.len_lines() {
        return None;
    }

    let ix = rope.line_to_char(line as usize) + character as usize;

    if ix >= rope.len_chars() {
        return None;
    }

    // Find the word boundaries by searching for whitespace
    // Search backwards from cursor to find word start
    let mut word_start = ix;
    while word_start > 0 {
        let ch = rope.char(word_start.saturating_sub(1));
        if ch.is_whitespace() {
            break;
        }
        word_start -= 1;
    }

    // Search forwards from cursor to find word end
    let mut word_end = ix;
    while word_end < rope.len_chars() {
        let ch = rope.char(word_end);
        if ch.is_whitespace() {
            break;
        }
        word_end += 1;
    }

    // Extract the word for validation
    let word_str = rope.slice(word_start..word_end).to_string();

    // Check if this word has any definitions/references
    if def_index.find_all_references(&word_str, true).is_empty() {
        return None;
    }

    // Calculate line and column positions
    let start_line = rope.char_to_line(word_start);
    let start_col = word_start - rope.line_to_char(start_line);

    let end_line = rope.char_to_line(word_end);
    let end_col = word_end - rope.line_to_char(end_line);

    Some(PrepareRenameResponse::Range(Range {
        start: lsp_types::Position {
            line: start_line as u32,
            character: start_col as u32,
        },
        end: lsp_types::Position {
            line: end_line as u32,
            character: end_col as u32,
        },
    }))
}
