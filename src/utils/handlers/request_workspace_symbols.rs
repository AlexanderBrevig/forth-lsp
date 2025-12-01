#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::handlers::send_response;

use lsp_server::{Connection, Request};
use lsp_types::{SymbolInformation, SymbolKind, request::WorkspaceSymbolRequest};

use super::cast;

// Extract workspace symbol logic for testing
pub fn get_workspace_symbols(query: &str, def_index: &DefinitionIndex) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    // If query is empty, return all symbols
    // Otherwise, filter by query (case-insensitive substring match)
    for word in def_index.all_words() {
        if query.is_empty() || word.to_lowercase().contains(&query.to_lowercase()) {
            // Get all locations for this word
            let locations = def_index.find_definitions(&word);

            // Create a SymbolInformation for each location
            for location in locations {
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: word.clone(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    location,
                    container_name: None,
                });
            }
        }
    }

    symbols
}

pub fn handle_workspace_symbols(
    req: &Request,
    connection: &Connection,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<WorkspaceSymbolRequest>(req.clone()) {
        Ok((id, params)) => {
            log_request_msg!(id, "workspace symbol query: {:?}", params.query);

            let symbols = get_workspace_symbols(&params.query, def_index);

            send_response(connection, id, symbols)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Workspace symbols", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use std::env;

    fn create_test_index() -> DefinitionIndex {
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();

        let file1 = temp_dir.join("file1.forth").to_string_lossy().to_string();
        let file2 = temp_dir.join("file2.forth").to_string_lossy().to_string();

        index.update_file(&file1, &Rope::from_str(": add1 1 + ;\n: double 2 * ;"));
        index.update_file(&file2, &Rope::from_str(": square dup * ;\n: mytest 1 + ;"));

        index
    }

    #[test]
    fn test_workspace_symbols_empty_query() {
        let index = create_test_index();
        let symbols = get_workspace_symbols("", &index);

        // Should return all symbols
        assert_eq!(symbols.len(), 4);
        let names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"add1".to_string()));
        assert!(names.contains(&"double".to_string()));
        assert!(names.contains(&"square".to_string()));
        assert!(names.contains(&"mytest".to_string()));
    }

    #[test]
    fn test_workspace_symbols_filter_by_query() {
        let index = create_test_index();
        let symbols = get_workspace_symbols("test", &index);

        // Should only return symbols containing "test"
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "mytest");
    }

    #[test]
    fn test_workspace_symbols_case_insensitive() {
        let index = create_test_index();
        let symbols_upper = get_workspace_symbols("ADD", &index);
        let symbols_lower = get_workspace_symbols("add", &index);

        assert_eq!(symbols_upper.len(), 1);
        assert_eq!(symbols_lower.len(), 1);
        assert_eq!(symbols_upper[0].name, symbols_lower[0].name);
    }

    #[test]
    fn test_workspace_symbols_partial_match() {
        let index = create_test_index();
        let symbols = get_workspace_symbols("dd", &index);

        // Should match "add1"
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "add1");
    }

    #[test]
    fn test_workspace_symbols_no_matches() {
        let index = create_test_index();
        let symbols = get_workspace_symbols("nonexistent", &index);

        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn test_workspace_symbols_all_are_functions() {
        let index = create_test_index();
        let symbols = get_workspace_symbols("", &index);

        // All Forth word definitions should be marked as functions
        assert!(symbols.iter().all(|s| s.kind == SymbolKind::FUNCTION));
    }

    #[test]
    fn test_workspace_symbols_have_locations() {
        let index = create_test_index();
        let symbols = get_workspace_symbols("add1", &index);

        assert_eq!(symbols.len(), 1);
        // Should have valid location
        assert!(symbols[0].location.uri.to_string().contains("file1.forth"));
        assert_eq!(symbols[0].location.range.start.line, 0);
    }

    #[test]
    fn test_workspace_symbols_duplicate_definitions() {
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();

        let file1 = temp_dir.join("dup1.forth").to_string_lossy().to_string();
        let file2 = temp_dir.join("dup2.forth").to_string_lossy().to_string();

        index.update_file(&file1, &Rope::from_str(": test 1 + ;"));
        index.update_file(&file2, &Rope::from_str(": test 2 * ;"));

        let symbols = get_workspace_symbols("test", &index);

        // Should find both definitions
        assert_eq!(symbols.len(), 2);
        assert!(
            symbols[0].location.uri.to_string().contains("dup1.forth")
                || symbols[0].location.uri.to_string().contains("dup2.forth")
        );
        assert!(
            symbols[1].location.uri.to_string().contains("dup1.forth")
                || symbols[1].location.uri.to_string().contains("dup2.forth")
        );
    }
}
