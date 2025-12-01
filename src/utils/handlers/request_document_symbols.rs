#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::{
    data_to_position::ToPosition, definition_helpers::find_colon_definitions,
    handlers::send_response, token_utils::extract_word_name_with_range,
};

use std::collections::HashMap;

use forth_lexer::{
    parser::Lexer,
    token::{Data, Token},
};
use lsp_server::{Connection, Request};
use lsp_types::{
    request::DocumentSymbolRequest, DocumentSymbol, DocumentSymbolResponse, SymbolKind,
};
use ropey::Rope;

use super::cast;

/// Create a DocumentSymbol for a Forth word definition
fn create_document_symbol(
    name: String,
    begin: &Data,
    end: &Data,
    selection_start: usize,
    selection_end: usize,
    rope: &Rope,
) -> DocumentSymbol {
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail: None,
        kind: SymbolKind::FUNCTION,
        tags: None,
        deprecated: None,
        range: lsp_types::Range {
            start: begin.to_position_start(rope),
            end: end.to_position_end(rope),
        },
        selection_range: lsp_types::Range {
            start: Data::new(selection_start, selection_end, "").to_position_start(rope),
            end: Data::new(selection_start, selection_end, "").to_position_end(rope),
        },
        children: None,
    }
}

// Extract document symbols logic for testing
pub fn get_document_symbols(rope: &Rope) -> Vec<DocumentSymbol> {
    let mut symbols = vec![];
    let progn = rope.to_string();
    let mut lexer = Lexer::new(progn.as_str());
    let tokens = lexer.parse();

    // Find all word definitions (: word ... ;)
    for result in find_colon_definitions(&tokens) {
        // A valid word definition has at least ": name ;"
        // The name can be either a Word or Number token
        // If a Number is followed immediately by a Word (like "2swap"), combine them
        if result.len() >= 2 {
            let Some((name, selection_start, selection_end)) =
                extract_word_name_with_range(result, 1)
            else {
                continue;
            };

            let tok = Token::Illegal(Data::new(0, 0, ""));
            let begin = result.first().unwrap_or(&tok).get_data();
            let end = result.last().unwrap_or(&tok).get_data();

            symbols.push(create_document_symbol(
                name,
                begin,
                end,
                selection_start,
                selection_end,
                rope,
            ));
        }
    }

    symbols
}

pub fn handle_document_symbols(
    req: &Request,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    match cast::<DocumentSymbolRequest>(req.clone()) {
        Ok((id, params)) => {
            log_request!(id, params);
            let rope = if let Some(rope) = files.get_mut(&params.text_document.uri.to_string()) {
                rope
            } else {
                return Err(Error::NoSuchFile(params.text_document.uri.to_string()));
            };

            let symbols = get_document_symbols(rope);
            let result = Some(DocumentSymbolResponse::Nested(symbols));
            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Document symbols", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;

    #[test]
    fn test_document_symbols_single_definition() {
        let rope = Rope::from_str(": add1 1 + ;");
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "add1");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
        assert_eq!(
            symbols[0].range.start,
            Position {
                line: 0,
                character: 0
            }
        );
    }

    #[test]
    fn test_document_symbols_multiple_definitions() {
        let rope = Rope::from_str(": add1 1 + ;\n\n: double 2 * ;\n\n: square dup * ;\n\n");
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "add1");
        assert_eq!(symbols[1].name, "double");
        assert_eq!(symbols[2].name, "square");

        // All should be functions
        assert!(symbols.iter().all(|s| s.kind == SymbolKind::FUNCTION));
    }

    #[test]
    fn test_document_symbols_with_comments() {
        let rope = Rope::from_str(": add1 ( n -- n ) \\ adds one\n  1 + ;");
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "add1");
    }

    #[test]
    fn test_document_symbols_multiline_definition() {
        let rope = Rope::from_str(
            ": factorial\n  dup 0= if\n    drop 1\n  else\n    dup 1- factorial *\n  then\n;",
        );
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "factorial");
        assert_eq!(symbols[0].range.start.line, 0);
        assert_eq!(symbols[0].range.end.line, 6);
    }

    #[test]
    fn test_document_symbols_empty_file() {
        let rope = Rope::from_str("");
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn test_document_symbols_no_definitions() {
        let rope = Rope::from_str("1 2 + . \\ just some code, no definitions");
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn test_document_symbols_selection_range() {
        let rope = Rope::from_str(": test 1 + ;");
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 1);
        // selection_range should point to just the word name
        assert_eq!(
            symbols[0].selection_range.start,
            Position {
                line: 0,
                character: 2
            }
        );
        assert_eq!(
            symbols[0].selection_range.end,
            Position {
                line: 0,
                character: 6
            }
        );
    }

    #[test]
    fn test_document_symbols_complex_file() {
        let rope = Rope::from_str(
            r#"
\ Math utilities
: add1 ( n -- n ) 1 + ;
: sub1 ( n -- n ) 1 - ;

\ Stack manipulation
: 2swap ( a b c d -- c d a b ) rot >r rot r> ;

\ Conditional
: abs ( n -- u ) dup 0< if negate then ;
"#,
        );
        let symbols = get_document_symbols(&rope);

        assert_eq!(symbols.len(), 4);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["add1", "sub1", "2swap", "abs"]);
    }
}
