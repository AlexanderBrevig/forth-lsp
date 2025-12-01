#[allow(unused_imports)]
use crate::prelude::*;

pub mod common;
pub mod notification_did_change;
pub mod notification_did_open;
pub mod notification_did_save;
pub mod request_completion;
pub mod request_document_symbols;
pub mod request_find_references;
pub mod request_goto_definition;
pub mod request_hover;
pub mod request_prepare_rename;
pub mod request_rename;
pub mod request_signature_help;
pub mod request_workspace_symbols;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};

pub fn cast<R>(req: Request) -> Result<(RequestId, R::Params)>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD).map_err(Error::ExtractRequestError)
}

pub fn cast_notification<N>(req: Notification) -> Result<N::Params>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    req.extract(N::METHOD)
        .map_err(Error::ExtractNotificationError)
}

pub fn send_response<T: serde::Serialize>(
    connection: &Connection,
    id: RequestId,
    result: T,
) -> Result<()> {
    let result = serde_json::to_value(result)
        .map_err(|e| Error::Generic(format!("Serialization error: {}", e)))?;
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
