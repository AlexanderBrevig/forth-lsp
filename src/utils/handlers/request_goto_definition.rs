#[allow(unused_imports)]
use crate::prelude::*;
use crate::{
    utils::{
        HashMapGetForLSPParams,
        definition_index::DefinitionIndex,
        handlers::send_response,
        ropey::{get_ix::GetIx, word_on_or_before::WordOnOrBefore},
    },
    words::Words,
};

use std::collections::HashMap;

use lsp_server::{Connection, Request};
use lsp_types::{GotoDefinitionResponse, request::GotoDefinition};
use ropey::Rope;

use super::cast;

// Test-only imports
#[cfg(test)]
use crate::utils::{
    data_to_position::data_range_from_to, definition_helpers::find_colon_definitions,
    token_utils::extract_word_name, uri_helpers::path_to_uri,
};
#[cfg(test)]
use forth_lexer::{
    parser::Lexer,
    token::{Data, Token},
};
#[cfg(test)]
use lsp_types::Location;

// Extract goto definition logic for testing
#[cfg(test)]
pub fn find_word_definitions(word: &str, files: &HashMap<String, Rope>) -> Vec<Location> {
    let mut ret: Vec<Location> = vec![];

    for (file, rope) in files.iter() {
        let progn = rope.to_string();
        let mut lexer = Lexer::new(progn.as_str());
        let tokens = lexer.parse();

        for result in find_colon_definitions(&tokens) {
            // Check if this definition matches the word we're looking for
            if result.len() >= 2 {
                let Some(name) = extract_word_name(result, 1) else {
                    continue;
                };

                if name.to_lowercase() == word.to_lowercase() {
                    let tok = Token::Illegal(Data::new(0, 0, ""));
                    let begin = result.first().unwrap_or(&tok).get_data();
                    let end = result.last().unwrap_or(&tok).get_data();
                    if let Some(uri) = path_to_uri(file) {
                        ret.push(Location {
                            uri,
                            range: data_range_from_to(begin, end, rope),
                        });
                    }
                }
            }
        }
    }
    ret
}

pub fn handle_goto_definition(
    req: &Request,
    connection: &Connection,
    _data: &Words,
    files: &mut HashMap<String, Rope>,
    def_index: &DefinitionIndex,
) -> Result<()> {
    match cast::<GotoDefinition>(req.clone()) {
        Ok((id, params)) => {
            log_request!(id, params);
            let rope = if let Some(rope) =
                files.for_position_param(&params.text_document_position_params)
            {
                rope
            } else {
                return Err(Error::NoSuchFile(
                    params
                        .text_document_position_params
                        .text_document
                        .uri
                        .to_string(),
                ));
            };
            let ix = rope.get_ix(&params);
            if ix >= rope.len_chars() {
                return Err(Error::OutOfBounds(ix));
            }
            let word = rope.word_on_or_before(ix).to_string();
            log_debug!("Word: {}", word);
            let ret = def_index.find_definitions(&word);
            let result = Some(GotoDefinitionResponse::Array(ret));
            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Goto definition", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env;

    fn create_test_files() -> HashMap<String, Rope> {
        let mut files = HashMap::new();

        // Create an absolute path using temp directory
        let temp_dir = env::temp_dir();
        let file1_path = temp_dir.join("test1.forth").to_string_lossy().to_string();
        let file2_path = temp_dir.join("test2.forth").to_string_lossy().to_string();

        files.insert(
            file1_path,
            Rope::from_str(": add1 ( n -- n ) 1 + ;\n: double 2 * ;"),
        );
        files.insert(
            file2_path,
            Rope::from_str(": square dup * ;\n: cube dup dup * * ;"),
        );
        files
    }

    #[test]
    fn test_find_word_definition_exists() {
        let files = create_test_files();
        let locations = find_word_definitions("add1", &files);

        assert_eq!(locations.len(), 1);
        let loc = &locations[0];
        assert!(loc.uri.to_string().contains("test1.forth"));
        assert_eq!(loc.range.start.line, 0);
        assert_eq!(loc.range.start.character, 0);
    }

    #[test]
    fn test_find_word_definition_case_insensitive() {
        let files = create_test_files();
        let locations_upper = find_word_definitions("DOUBLE", &files);
        let locations_lower = find_word_definitions("double", &files);

        assert_eq!(locations_upper.len(), 1);
        assert_eq!(locations_lower.len(), 1);
        assert_eq!(locations_upper[0].uri, locations_lower[0].uri);
        assert_eq!(locations_upper[0].range, locations_lower[0].range);
    }

    #[test]
    fn test_find_word_definition_in_different_file() {
        let files = create_test_files();
        let locations = find_word_definitions("square", &files);

        assert_eq!(locations.len(), 1);
        assert!(locations[0].uri.to_string().contains("test2.forth"));
    }

    #[test]
    fn test_find_word_definition_not_found() {
        let files = create_test_files();
        let locations = find_word_definitions("nonexistent", &files);

        assert_eq!(locations.len(), 0);
    }

    #[test]
    fn test_find_word_definition_multiline() {
        let mut files = HashMap::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir
            .join("multiline.forth")
            .to_string_lossy()
            .to_string();

        files.insert(
            file_path,
            Rope::from_str(
                ": factorial\n  dup 0= if\n    drop 1\n  else\n    dup 1- factorial *\n  then\n;",
            ),
        );

        let locations = find_word_definitions("factorial", &files);
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].range.start.line, 0);
        assert_eq!(locations[0].range.start.character, 0);
        // Should span to the semicolon on the last line
        assert_eq!(locations[0].range.end.line, 6);
    }

    #[test]
    fn test_find_multiple_definitions_same_name() {
        let mut files = HashMap::new();
        let temp_dir = env::temp_dir();
        let file1_path = temp_dir.join("dup1.forth").to_string_lossy().to_string();
        let file2_path = temp_dir.join("dup2.forth").to_string_lossy().to_string();

        files.insert(file1_path.clone(), Rope::from_str(": test 1 + ;"));
        files.insert(file2_path.clone(), Rope::from_str(": test 2 * ;"));

        let locations = find_word_definitions("test", &files);
        // Should find both definitions
        assert_eq!(locations.len(), 2);
    }

    #[test]
    fn test_find_word_definition_with_comments() {
        let mut files = HashMap::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir
            .join("commented.forth")
            .to_string_lossy()
            .to_string();

        files.insert(
            file_path,
            Rope::from_str(": add1 ( n -- n ) \\ adds one\n  1 + ;"),
        );

        let locations = find_word_definitions("add1", &files);
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].range.start.line, 0);
    }

    #[test]
    fn test_find_word_empty_string() {
        let files = create_test_files();
        let locations = find_word_definitions("", &files);

        assert_eq!(locations.len(), 0);
    }

    #[test]
    fn test_find_word_definition_number_prefix() {
        let mut files = HashMap::new();
        let temp_dir = env::temp_dir();
        let file_path = temp_dir
            .join("number_prefix.forth")
            .to_string_lossy()
            .to_string();

        files.insert(
            file_path,
            Rope::from_str(": 2swap ( a b c d -- c d a b ) rot >r rot r> ;"),
        );

        let locations = find_word_definitions("2swap", &files);
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].range.start.line, 0);
    }
}
