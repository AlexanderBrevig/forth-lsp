#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::definition_index::DefinitionIndex;
use crate::utils::handlers::{common::ExtractedPosition, send_response};
use crate::utils::ropey::word_on_or_before::WordOnOrBefore;

#[cfg(test)]
use crate::utils::uri_helpers::path_to_uri;

use lsp_server::{Connection, Request};
use lsp_types::{request::Rename, TextEdit, WorkspaceEdit};
use ropey::Rope;
use std::collections::HashMap;

use super::cast;

// Extract rename logic for testing
pub fn get_rename_edits(
    file_path: &str,
    line: u32,
    character: u32,
    new_name: &str,
    files: &HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Option<WorkspaceEdit> {
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

    eprintln!("Rename: word at {}:{} is '{}'", line, character, word);

    if word.is_empty() {
        return None;
    }

    // Find all references including the definition
    let locations = def_index.find_all_references(&word, true);

    if locations.is_empty() {
        return None;
    }

    // Group edits by file URI
    let mut changes = HashMap::new();

    for location in locations {
        let edits = changes.entry(location.uri.clone()).or_insert_with(Vec::new);

        edits.push(TextEdit {
            range: location.range,
            new_text: new_name.to_string(),
        });
    }

    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

pub fn handle_rename(
    req: &Request,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<Rename>(req.clone()) {
        Ok((id, params)) => {
            let pos =
                ExtractedPosition::from_text_document_position(&params.text_document_position)?;

            log_request_msg!(id, "rename at {} to '{}'", pos.format(), params.new_name);

            let workspace_edit = get_rename_edits(
                &pos.file_uri,
                pos.line,
                pos.character,
                &params.new_name,
                files,
                def_index,
            );

            send_response(connection, id, workspace_edit)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Rename", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::definition_index::DefinitionIndex;
    use ropey::Rope;
    use std::env;

    fn create_test_files() -> (HashMap<String, Rope>, DefinitionIndex) {
        let mut files = HashMap::new();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();

        let file1 = temp_dir.join("rename1.forth").to_string_lossy().to_string();
        let file2 = temp_dir.join("rename2.forth").to_string_lossy().to_string();

        let content1 = Rope::from_str(": add1 1 + ;\n: test add1 add1 ;\n");
        let content2 = Rope::from_str(": double 2 * ;\n: mytest add1 double ;\n");

        index.update_file(&file1, &content1);
        index.update_file(&file2, &content2);

        files.insert(file1.clone(), content1);
        files.insert(file2.clone(), content2);

        (files, index)
    }

    #[test]
    fn test_rename_basic() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("rename1.forth").to_string_lossy().to_string();

        // Rename "add1" to "increment"
        let edit = get_rename_edits(&file1, 0, 2, "increment", &files, &index);

        assert!(edit.is_some());
        let edit = edit.unwrap();
        assert!(edit.changes.is_some());

        let changes = edit.changes.unwrap();
        // Should have edits in both files
        assert!(!changes.is_empty());

        // Count total edits across all files
        let total_edits: usize = changes.values().map(|v| v.len()).sum();
        // Should be 4 edits: 1 definition + 3 references
        assert_eq!(total_edits, 4);
    }

    #[test]
    fn test_rename_from_usage() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("rename1.forth").to_string_lossy().to_string();

        // Rename from a usage site (line 1, character 8 is "add1" in "test add1 add1")
        let edit = get_rename_edits(&file1, 1, 8, "newname", &files, &index);

        assert!(edit.is_some());
        let edit = edit.unwrap();

        let changes = edit.changes.unwrap();
        let total_edits: usize = changes.values().map(|v| v.len()).sum();
        // Should rename all occurrences
        assert_eq!(total_edits, 4);
    }

    #[test]
    fn test_rename_cross_file() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("rename1.forth").to_string_lossy().to_string();

        let edit = get_rename_edits(&file1, 0, 2, "renamed", &files, &index);

        assert!(edit.is_some());
        let edit = edit.unwrap();
        let changes = edit.changes.unwrap();

        // Should have edits in multiple files
        let file1_uri = path_to_uri(&file1).unwrap();
        let file2 = temp_dir.join("rename2.forth").to_string_lossy().to_string();
        let file2_uri = path_to_uri(&file2).unwrap();

        // Both files should have edits
        assert!(changes.contains_key(&file1_uri));
        assert!(changes.contains_key(&file2_uri));
    }

    #[test]
    fn test_rename_preserves_new_name() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("rename1.forth").to_string_lossy().to_string();

        let new_name = "my_new_word";
        let edit = get_rename_edits(&file1, 0, 2, new_name, &files, &index);

        assert!(edit.is_some());
        let edit = edit.unwrap();
        let changes = edit.changes.unwrap();

        // All edits should use the new name
        for edits_list in changes.values() {
            for text_edit in edits_list {
                assert_eq!(text_edit.new_text, new_name);
            }
        }
    }

    #[test]
    fn test_rename_invalid_position() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("rename1.forth").to_string_lossy().to_string();

        // Position way beyond file
        let edit = get_rename_edits(&file1, 1000, 1000, "newname", &files, &index);

        assert!(edit.is_none());
    }

    #[test]
    fn test_rename_no_references() {
        let mut files = HashMap::new();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file = temp_dir
            .join("noref_rename.forth")
            .to_string_lossy()
            .to_string();

        // Define a word with no usages
        let content = Rope::from_str(": unused ;\n: another 1 + ;");
        index.update_file(&file, &content);
        files.insert(file.clone(), content);

        // Rename the unused word
        let edit = get_rename_edits(&file, 0, 2, "renamed", &files, &index);

        assert!(edit.is_some());
        let edit = edit.unwrap();
        let changes = edit.changes.unwrap();

        // Should have 1 edit (just the definition)
        let total_edits: usize = changes.values().map(|v| v.len()).sum();
        assert_eq!(total_edits, 1);
    }

    #[test]
    fn test_rename_case_insensitive() {
        let mut files = HashMap::new();
        let mut index = DefinitionIndex::new();
        let temp_dir = env::temp_dir();
        let file = temp_dir
            .join("case_rename.forth")
            .to_string_lossy()
            .to_string();

        let content = Rope::from_str(": MyWord 1 + ;\n: test MYWORD myword MyWord ;");
        index.update_file(&file, &content);
        files.insert(file.clone(), content);

        // Rename MyWord (should rename all case variations)
        let edit = get_rename_edits(&file, 0, 2, "NewName", &files, &index);

        assert!(edit.is_some());
        let edit = edit.unwrap();
        let changes = edit.changes.unwrap();

        // Should rename all 4 occurrences (1 def + 3 usages)
        let total_edits: usize = changes.values().map(|v| v.len()).sum();
        assert_eq!(total_edits, 4);
    }

    #[test]
    fn test_rename_builtin_word() {
        let (files, index) = create_test_files();
        let temp_dir = env::temp_dir();
        let file = temp_dir.join("rename1.forth").to_string_lossy().to_string();

        // Try to rename a built-in word "+" (which has no definition in our files)
        // Position pointing to "+" in ": add1 1 + ;"
        let edit = get_rename_edits(&file, 0, 9, "plus", &files, &index);

        // Should work - it will rename all usages of "+" in the workspace
        // (even though it's a built-in, we can still rename its usages)
        assert!(edit.is_some());
    }
}
