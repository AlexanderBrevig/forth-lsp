#[allow(unused_imports)]
use crate::prelude::*;
use crate::utils::{
    data_to_position::to_line_char, handlers::send_response, word_lookup::find_builtin_word,
};
use crate::words::Words;

use std::collections::HashMap;

use forth_lexer::{parser::Lexer, token::Token};
use lsp_server::{Connection, Request};
use lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensResult,
    SemanticTokensServerCapabilities, request::SemanticTokensFullRequest,
};
use ropey::Rope;

use super::cast;

/// Semantic token type indices (must match legend order)
const TOKEN_TYPE_KEYWORD: u32 = 0;
const TOKEN_TYPE_FUNCTION: u32 = 1;
const TOKEN_TYPE_COMMENT: u32 = 2;
const TOKEN_TYPE_NUMBER: u32 = 3;
const TOKEN_TYPE_VARIABLE: u32 = 4;
const TOKEN_TYPE_STRING: u32 = 5;

/// Semantic token modifier bits
const MODIFIER_DEFINITION: u32 = 1 << 0;
const MODIFIER_DEFAULT_LIBRARY: u32 = 1 << 1;

/// Control-flow words that should be highlighted as keywords
const CONTROL_FLOW_WORDS: &[&str] = &[
    "IF", "THEN", "ELSE", "DO", "LOOP", "BEGIN", "UNTIL", "WHILE", "REPEAT", "CASE", "ENDCASE",
    "OF", "ENDOF", "+LOOP", "DOES>", "EXIT", "LEAVE", "UNLOOP", "?DO", "RECURSE",
];

/// Defining words that should be highlighted as keywords
const DEFINING_WORDS: &[&str] = &[
    "VARIABLE",
    "CONSTANT",
    "CREATE",
    "VALUE",
    "2VARIABLE",
    "2CONSTANT",
    "2VALUE",
    "FVARIABLE",
    "FCONSTANT",
    "DEFER",
    "BUFFER:",
];

fn is_control_flow_word(word: &str) -> bool {
    let upper = word.to_uppercase();
    CONTROL_FLOW_WORDS.iter().any(|&w| w == upper)
}

fn is_defining_word(word: &str) -> bool {
    let upper = word.to_uppercase();
    DEFINING_WORDS.iter().any(|&w| w == upper)
}

pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::COMMENT,
            SemanticTokenType::NUMBER,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::STRING,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ],
    }
}

pub fn semantic_tokens_capabilities() -> SemanticTokensServerCapabilities {
    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
        legend: semantic_tokens_legend(),
        full: Some(SemanticTokensFullOptions::Bool(true)),
        range: None,
        work_done_progress_options: Default::default(),
    })
}

pub fn get_semantic_tokens(rope: &Rope, builtin_words: &Words) -> SemanticTokens {
    let progn = rope.to_string();
    let mut lexer = Lexer::new(progn.as_str());
    let tokens = lexer.parse();

    let mut semantic_tokens = vec![];
    let mut prev_line: u32 = 0;
    let mut prev_start: u32 = 0;
    let mut after_colon = false;
    let mut after_defining_word = false;

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let (token_type, modifiers, skip_extra) = match token {
            Token::Illegal(_) | Token::Eof(_) => {
                i += 1;
                continue;
            }
            Token::Colon(_) => {
                after_colon = true;
                (TOKEN_TYPE_KEYWORD, 0u32, false)
            }
            Token::Semicolon(_) => (TOKEN_TYPE_KEYWORD, 0u32, false),
            Token::Comment(_) => (TOKEN_TYPE_COMMENT, 0u32, false),
            Token::StackComment(_) => (TOKEN_TYPE_STRING, 0u32, false),
            Token::Number(num_data) => {
                if after_colon {
                    // Check if this number is part of a combined name like "2swap"
                    if let Some(Token::Word(word_data)) = tokens.get(i + 1)
                        && num_data.end == word_data.start
                    {
                        // Combined token: emit as single FUNCTION+DEFINITION
                        let (line, start_char) = to_line_char(num_data.start, rope);
                        let length = (word_data.end - num_data.start) as u32;
                        let delta_line = line - prev_line;
                        let delta_start = if delta_line == 0 {
                            start_char - prev_start
                        } else {
                            start_char
                        };
                        semantic_tokens.push(SemanticToken {
                            delta_line,
                            delta_start,
                            length,
                            token_type: TOKEN_TYPE_FUNCTION,
                            token_modifiers_bitset: MODIFIER_DEFINITION,
                        });
                        prev_line = line;
                        prev_start = start_char;
                        after_colon = false;
                        i += 2; // skip both number and word
                        continue;
                    }
                    // Just a number as the definition name
                    after_colon = false;
                    (TOKEN_TYPE_FUNCTION, MODIFIER_DEFINITION, false)
                } else {
                    (TOKEN_TYPE_NUMBER, 0u32, false)
                }
            }
            Token::Word(data) => {
                if after_colon {
                    after_colon = false;
                    (TOKEN_TYPE_FUNCTION, MODIFIER_DEFINITION, false)
                } else if after_defining_word {
                    after_defining_word = false;
                    let mut mods = MODIFIER_DEFINITION;
                    if find_builtin_word(data.value, builtin_words).is_some() {
                        mods |= MODIFIER_DEFAULT_LIBRARY;
                    }
                    (TOKEN_TYPE_VARIABLE, mods, false)
                } else if is_control_flow_word(data.value) || is_defining_word(data.value) {
                    if is_defining_word(data.value) {
                        after_defining_word = true;
                    }
                    (TOKEN_TYPE_KEYWORD, 0u32, false)
                } else {
                    let mut mods = 0u32;
                    if find_builtin_word(data.value, builtin_words).is_some() {
                        mods |= MODIFIER_DEFAULT_LIBRARY;
                    }
                    (TOKEN_TYPE_VARIABLE, mods, false)
                }
            }
        };

        let data = token.get_data();
        let (line, start_char) = to_line_char(data.start, rope);
        let length = (data.end - data.start) as u32;
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 {
            start_char - prev_start
        } else {
            start_char
        };

        semantic_tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: modifiers,
        });

        prev_line = line;
        prev_start = start_char;

        let _ = skip_extra;
        i += 1;
    }

    SemanticTokens {
        result_id: None,
        data: semantic_tokens,
    }
}

pub fn handle_semantic_tokens_full(
    req: &Request,
    connection: &Connection,
    files: &mut HashMap<String, Rope>,
    builtin_words: &Words,
) -> Result<()> {
    match cast::<SemanticTokensFullRequest>(req.clone()) {
        Ok((id, params)) => {
            log_request!(id, params);
            let rope = if let Some(rope) = files.get_mut(&params.text_document.uri.to_string()) {
                rope
            } else {
                return Err(Error::NoSuchFile(params.text_document.uri.to_string()));
            };

            let tokens = get_semantic_tokens(rope, builtin_words);
            let result = Some(SemanticTokensResult::Tokens(tokens));
            send_response(connection, id, result)?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => {
            log_handler_error!("Semantic tokens", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_words() -> Words<'static> {
        Words { words: vec![] }
    }

    #[test]
    fn test_semantic_tokens_empty() {
        let rope = Rope::from_str("");
        let result = get_semantic_tokens(&rope, &empty_words());
        assert!(result.data.is_empty());
    }

    #[test]
    fn test_semantic_tokens_colon_definition() {
        let rope = Rope::from_str(": add1 1 + ;");
        let builtin_words = Words::default();
        let result = get_semantic_tokens(&rope, &builtin_words);

        // : → KEYWORD
        assert_eq!(result.data[0].token_type, TOKEN_TYPE_KEYWORD);
        assert_eq!(result.data[0].delta_line, 0);
        assert_eq!(result.data[0].delta_start, 0);
        assert_eq!(result.data[0].length, 1);

        // add1 → FUNCTION + DEFINITION
        assert_eq!(result.data[1].token_type, TOKEN_TYPE_FUNCTION);
        assert_eq!(result.data[1].token_modifiers_bitset, MODIFIER_DEFINITION);
        assert_eq!(result.data[1].delta_start, 2); // offset from ':'

        // 1 → NUMBER
        assert_eq!(result.data[2].token_type, TOKEN_TYPE_NUMBER);

        // + → VARIABLE + DEFAULT_LIBRARY (builtin)
        assert_eq!(result.data[3].token_type, TOKEN_TYPE_VARIABLE);
        assert_ne!(
            result.data[3].token_modifiers_bitset & MODIFIER_DEFAULT_LIBRARY,
            0
        );

        // ; → KEYWORD
        assert_eq!(result.data[4].token_type, TOKEN_TYPE_KEYWORD);
    }

    #[test]
    fn test_semantic_tokens_with_stack_comment() {
        let rope = Rope::from_str(": add1 ( n -- n ) 1 + ;");
        let result = get_semantic_tokens(&rope, &empty_words());

        // Find the stack comment token
        let stack_comment = result
            .data
            .iter()
            .find(|t| t.token_type == TOKEN_TYPE_STRING);
        assert!(
            stack_comment.is_some(),
            "Should have a STRING token for stack comment"
        );
    }

    #[test]
    fn test_semantic_tokens_with_comment() {
        let rope = Rope::from_str("\\ this is a comment");
        let result = get_semantic_tokens(&rope, &empty_words());

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].token_type, TOKEN_TYPE_COMMENT);
    }

    #[test]
    fn test_semantic_tokens_multiline() {
        let rope = Rope::from_str(": add1\n  1 + ;");
        let result = get_semantic_tokens(&rope, &empty_words());

        // : on line 0
        assert_eq!(result.data[0].delta_line, 0);
        // add1 on line 0
        assert_eq!(result.data[1].delta_line, 0);
        // 1 on line 1
        assert_eq!(result.data[2].delta_line, 1);
        assert_eq!(result.data[2].delta_start, 2); // absolute char on new line
    }

    #[test]
    fn test_semantic_tokens_control_flow() {
        let rope = Rope::from_str(": test 0 if 1 then ;");
        let result = get_semantic_tokens(&rope, &empty_words());

        // Find IF and THEN tokens - they should be KEYWORD
        // Tokens: : test 0 if 1 then ;
        //         0 1    2 3  4 5    6
        assert_eq!(result.data[3].token_type, TOKEN_TYPE_KEYWORD); // if
        assert_eq!(result.data[5].token_type, TOKEN_TYPE_KEYWORD); // then
    }

    #[test]
    fn test_semantic_tokens_number_prefix_name() {
        let rope = Rope::from_str(": 2swap rot >r rot r> ;");
        let result = get_semantic_tokens(&rope, &empty_words());

        // : → KEYWORD
        assert_eq!(result.data[0].token_type, TOKEN_TYPE_KEYWORD);
        // 2swap → FUNCTION + DEFINITION (combined number+word)
        assert_eq!(result.data[1].token_type, TOKEN_TYPE_FUNCTION);
        assert_ne!(
            result.data[1].token_modifiers_bitset & MODIFIER_DEFINITION,
            0
        );
        // The combined token should span the full "2swap"
        assert_eq!(result.data[1].length, 5);
    }

    #[test]
    fn test_semantic_tokens_defining_words() {
        let rope = Rope::from_str("VARIABLE counter");
        let result = get_semantic_tokens(&rope, &empty_words());

        // VARIABLE → KEYWORD
        assert_eq!(result.data[0].token_type, TOKEN_TYPE_KEYWORD);
        // counter → VARIABLE + DEFINITION
        assert_eq!(result.data[1].token_type, TOKEN_TYPE_VARIABLE);
        assert_ne!(
            result.data[1].token_modifiers_bitset & MODIFIER_DEFINITION,
            0
        );
    }
}
