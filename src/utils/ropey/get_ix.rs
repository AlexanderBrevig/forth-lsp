use lsp_types::{request::GotoTypeDefinitionParams, CompletionParams, HoverParams};
use ropey::Rope;

pub trait GetIx<T> {
    fn get_ix(&self, params: &T) -> usize;
}

impl GetIx<CompletionParams> for Rope {
    fn get_ix(&self, params: &CompletionParams) -> usize {
        self.line_to_char(params.text_document_position.position.line as usize)
            + params.text_document_position.position.character as usize
    }
}

impl GetIx<HoverParams> for Rope {
    fn get_ix(&self, params: &HoverParams) -> usize {
        self.line_to_char(params.text_document_position_params.position.line as usize)
            + params.text_document_position_params.position.character as usize
    }
}

impl GetIx<GotoTypeDefinitionParams> for Rope {
    fn get_ix(&self, params: &GotoTypeDefinitionParams) -> usize {
        self.line_to_char(params.text_document_position_params.position.line as usize)
            + params.text_document_position_params.position.character as usize
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_op() {
        //TODO: actually test this
    }
}
