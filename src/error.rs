#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Generic {0}")]
    Generic(String),
    #[error("SendError {0}")]
    SendError(String),
    #[error("OutOfBounds at ix {0}")]
    OutOfBounds(usize),
    #[error("NoSuchFile {0}")]
    NoSuchFile(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    ExtractNotificationError(#[from] lsp_server::ExtractError<lsp_server::Notification>),

    #[error(transparent)]
    ExtractRequestError(#[from] lsp_server::ExtractError<lsp_server::Request>),

    // #[error(transparent)]
    // ExtractResponseError(#[from] lsp_server::ExtractError<lsp_server::Response>),
    #[error(transparent)]
    ProtocolError(#[from] lsp_server::ProtocolError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}
