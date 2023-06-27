#[allow(unused_imports)]
use crate::prelude::*;

pub mod notification_did_change;
pub mod notification_did_open;
pub mod request_completion;
pub mod request_goto_definition;
pub mod request_hover;

use lsp_server::{Notification, Request, RequestId};

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
