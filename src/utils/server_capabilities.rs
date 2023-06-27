use lsp_types::{OneOf, ServerCapabilities, TextDocumentSyncKind};

pub fn forth_lsp_capabilities() -> ServerCapabilities {
    ServerCapabilities {
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
    }
}
