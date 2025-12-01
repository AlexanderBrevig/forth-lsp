#[allow(unused_imports)]
use crate::prelude::*;

pub mod data_to_position;
pub mod definition_helpers;
pub mod definition_index;
pub mod diagnostics;
pub mod find_variant_sublists;
pub mod find_variant_sublists_from_to;
pub mod handlers;
pub mod logging;
pub mod ropey;
pub mod server_capabilities;
pub mod token_utils;
pub mod uri_helpers;
pub mod word_lookup;

use lsp_types::TextDocumentPositionParams;
use std::collections::HashMap;

pub trait HashMapGetForLSPParams<T> {
    fn for_position_param(&mut self, params: &TextDocumentPositionParams) -> Option<&mut T>;
}

impl<T> HashMapGetForLSPParams<T> for HashMap<String, T> {
    fn for_position_param(&mut self, params: &TextDocumentPositionParams) -> Option<&mut T> {
        self.get_mut(&params.text_document.uri.to_string())
    }
}
