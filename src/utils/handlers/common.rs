use crate::prelude::*;
use lsp_types::{Position, TextDocumentIdentifier, TextDocumentPositionParams};

/// Extracted position information from a TextDocumentPositionParams
pub struct ExtractedPosition {
    pub file_uri: String,
    pub line: u32,
    pub character: u32,
}

impl ExtractedPosition {
    /// Extract file URI and position from TextDocumentPositionParams
    pub fn from_text_document_position(params: &TextDocumentPositionParams) -> Result<Self> {
        let file_uri = params.text_document.uri.to_string();

        Ok(ExtractedPosition {
            file_uri,
            line: params.position.line,
            character: params.position.character,
        })
    }

    /// Extract file URI and position from components
    pub fn from_parts(text_document: &TextDocumentIdentifier, position: &Position) -> Result<Self> {
        let file_uri = text_document.uri.to_string();

        Ok(ExtractedPosition {
            file_uri,
            line: position.line,
            character: position.character,
        })
    }

    /// Format position for logging
    pub fn format(&self) -> String {
        format!("{}:{}:{}", self.file_uri, self.line, self.character)
    }
}
