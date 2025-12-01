use lsp_types::{CompletionParams, HoverParams, request::GotoTypeDefinitionParams};
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
    use super::*;
    use lsp_types::{Position, TextDocumentIdentifier, TextDocumentPositionParams};
    use ropey::Rope;

    #[test]
    fn test_get_ix_completion_single_line() {
        let rope = Rope::from_str("hello world");
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: "file:///test.forth".parse().unwrap(),
                },
                position: Position {
                    line: 0,
                    character: 6,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        assert_eq!(rope.get_ix(&params), 6);
    }

    #[test]
    fn test_get_ix_completion_multiline() {
        let rope = Rope::from_str("line one\nline two\nline three");
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: "file:///test.forth".parse().unwrap(),
                },
                position: Position {
                    line: 1,
                    character: 5,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        // line 0: "line one\n" = 9 chars (0-8)
        // line 1: "line two\n" starts at char 9, position 5 = char 14
        assert_eq!(rope.get_ix(&params), 14);
    }

    #[test]
    fn test_get_ix_hover_start_of_file() {
        let rope = Rope::from_str(": add1 1 + ;");
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: "file:///test.forth".parse().unwrap(),
                },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: Default::default(),
        };
        assert_eq!(rope.get_ix(&params), 0);
    }

    #[test]
    fn test_get_ix_hover_multiline() {
        let rope = Rope::from_str(": test\n  1 2 3\n;");
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: "file:///test.forth".parse().unwrap(),
                },
                position: Position {
                    line: 2,
                    character: 0,
                },
            },
            work_done_progress_params: Default::default(),
        };
        // line 0: ": test\n" = 7 chars
        // line 1: "  1 2 3\n" = 8 chars (total 15)
        // line 2: ";" starts at char 15
        assert_eq!(rope.get_ix(&params), 15);
    }
}
