use crate::prelude::*;
use lsp_types::{Position, TextDocumentIdentifier, TextDocumentPositionParams};

/// Extracted position information from a TextDocumentPositionParams
pub struct ExtractedPosition {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

impl ExtractedPosition {
    /// Extract file path and position from TextDocumentPositionParams
    pub fn from_text_document_position(params: &TextDocumentPositionParams) -> Result<Self> {
        let file_path = params
            .text_document
            .uri
            .to_file_path()
            .map_err(|_| Error::Generic("Invalid file path".to_string()))?
            .to_string_lossy()
            .to_string();

        Ok(ExtractedPosition {
            file_path,
            line: params.position.line,
            character: params.position.character,
        })
    }

    /// Extract file path and position from components
    pub fn from_parts(text_document: &TextDocumentIdentifier, position: &Position) -> Result<Self> {
        let file_path = text_document
            .uri
            .to_file_path()
            .map_err(|_| Error::Generic("Invalid file path".to_string()))?
            .to_string_lossy()
            .to_string();

        Ok(ExtractedPosition {
            file_path,
            line: position.line,
            character: position.character,
        })
    }

    /// Format position for logging
    pub fn format(&self) -> String {
        format!("{}:{}:{}", self.file_path, self.line, self.character)
    }
}
