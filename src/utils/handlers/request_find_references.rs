#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::handlers::{common::ExtractedPosition, send_response};
use crate::utils::ropey::word_on_or_before::WordOnOrBefore;

use lsp_server::{Connection, Request};
use lsp_types::{Location, request::References};
use ropey::Rope;
use std::collections::HashMap;

use super::cast;

// Extract find references logic for testing
pub fn get_references(
    file_path: &str,
    line: u32,
    character: u32,
    include_declaration: bool,
    files: &HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Option<Vec<Location>> {
    let rope = files.get(file_path)?;

    // Check if line is within bounds
    if line as usize >= rope.len_lines() {
        return None;
    }

    let ix = rope.line_to_char(line as usize) + character as usize;

    if ix >= rope.len_chars() {
        return None;
    }

    let word = rope.word_on_or_before(ix).to_string();

    if word.is_empty() {
        return None;
    }

    let references = def_index.find_all_references(&word, include_declaration);

    if references.is_empty() {
        None
    } else {
        Some(references)
    }
}

pub fn handle_find_references(
    req: &Request,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<References>(req.clone()) {
        Ok((id, params)) => {
            let pos =
                ExtractedPosition::from_text_document_position(&params.text_document_position)?;

            eprintln!("#{id}: find references at {}", pos.format());

            let references = get_references(
                &pos.file_uri,
                pos.line,
                pos.character,
                params.context.include_declaration,
                files,
                def_index,
            );

            send_response(connection, id, references)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Find references", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use std::env;

    fn create_test_files() -> (HashMap<String, Rope>, DefinitionIndex) {
        let mut files = HashMap::new();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();

        let file1 = temp_dir.join("ref1.forth").to_string_lossy().to_string();
        let file2 = temp_dir.join("ref2.forth").to_string_lossy().to_string();

        let content1 = Rope::from_str(": add1 1 + ;\n: test add1 add1 ;\n");
        let content2 = Rope::from_str(": double 2 * ;\n: mytest add1 double ;\n");

        index.update_file(&file1, &content1);
        index.update_file(&file2, &content2);

        files.insert(file1.clone(), content1);
        files.insert(file2.clone(), content2);

        (files, index)
    }

    #[test]
    fn test_find_references_basic() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("ref1.forth").to_string_lossy().to_string();

        // Find references to "add1" from its definition
        let refs = get_references(&file1, 0, 2, false, &files, &index);

        assert!(refs.is_some());
        let refs = refs.unwrap();
        // Should find 3 references (2 in file1, 1 in file2)
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_find_references_with_declaration() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("ref1.forth").to_string_lossy().to_string();

        // Find references to "add1" including declaration
        let refs = get_references(&file1, 0, 2, true, &files, &index);

        assert!(refs.is_some());
        let refs = refs.unwrap();
        // Should find 4 locations (1 definition + 3 references)
        assert_eq!(refs.len(), 4);
    }

    #[test]
    fn test_find_references_from_usage() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("ref1.forth").to_string_lossy().to_string();

        // Find references from a usage site (line 1, "add1" in "test add1 add1")
        let refs = get_references(&file1, 1, 8, false, &files, &index);

        assert!(refs.is_some());
        let refs = refs.unwrap();
        // Should still find all 3 references
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_find_references_nonexistent_word() {
        let mut files = HashMap::new();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file = temp_dir.join("noref.forth").to_string_lossy().to_string();

        // Define a word that has no references
        let content = Rope::from_str(": unused ;\n: another 1 + ;");
        index.update_file(&file, &content);
        files.insert(file.clone(), content);

        // Find references for "unused" which has no references
        let refs = get_references(&file, 0, 2, false, &files, &index);

        // Should return None because there are no references to "unused"
        assert!(refs.is_none());
    }

    #[test]
    fn test_find_references_cross_file() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file2 = temp_dir.join("ref2.forth").to_string_lossy().to_string();

        // Find references to "double" from file2
        let refs = get_references(&file2, 0, 2, false, &files, &index);

        assert!(refs.is_some());
        let refs = refs.unwrap();
        // Should find 1 reference (in file2's mytest)
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_find_references_builtin_word() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("ref1.forth").to_string_lossy().to_string();

        // Find references to builtin word "+"
        let refs = get_references(&file1, 0, 9, false, &files, &index);

        assert!(refs.is_some());
        let refs = refs.unwrap();
        // Should find usages of "+"
        assert!(!refs.is_empty());
    }

    #[test]
    fn test_find_references_case_insensitive() {
        let mut files = HashMap::new();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();

        let file = temp_dir.join("case.forth").to_string_lossy().to_string();
        let content = Rope::from_str(": MyWord 1 + ;\n: test MYWORD myword MyWord ;\n");

        index.update_file(&file, &content);
        files.insert(file.clone(), content);

        // Find references to "MyWord"
        let refs = get_references(&file, 0, 2, false, &files, &index);

        assert!(refs.is_some());
        let refs = refs.unwrap();
        // Should find all 3 usages regardless of case
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_find_references_invalid_position() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("ref1.forth").to_string_lossy().to_string();

        // Try position way beyond file content
        let refs = get_references(&file1, 1000, 1000, false, &files, &index);

        // Should return None
        assert!(refs.is_none());
    }
}
