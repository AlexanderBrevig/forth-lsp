#[allow(unused_imports)]
use crate::prelude::*;
use crate::{
    utils::{
        data_to_position::ToPosition,
        find_variant_sublists_from_to::FindVariantSublistsFromTo,
        ropey::{get_ix::GetIx, word_on_or_before::WordOnOrBefore},
        HashMapGetForLSPParams,
    },
    words::Words,
};

use std::{collections::HashMap, mem::discriminant};

use forth_lexer::{
    parser::Lexer,
    token::{Data, Token},
};
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{request::GotoDefinition, GotoDefinitionResponse, Location, Range, Url};
use ropey::Rope;

use super::cast;

pub fn handle_goto_definition(
    req: &Request,
    connection: &Connection,
    _data: &Words,
    files: &mut HashMap<String, Rope>,
) -> Result<()> {
    match cast::<GotoDefinition>(req.clone()) {
        Ok((id, params)) => {
            eprintln!("#{id}: {params:?}");
            let mut ret: Vec<Location> = vec![];
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
            for (file, rope) in files.iter() {
                eprintln!("Word: {}", word);
                let progn = rope.to_string();
                let mut lexer = Lexer::new(progn.as_str());
                let tokens = lexer.parse();

                for result in tokens.find_variant_sublists_from_to(
                    discriminant(&Token::Colon(Data::default())),
                    discriminant(&Token::Semicolon(Data::default())),
                ) {
                    eprintln!("{:?}", result);
                    let tok = Token::Illegal(Data::new(0, 0, ""));
                    let begin = result.first().unwrap_or(&tok).get_data();
                    let end = result.last().unwrap_or(&tok).get_data();
                    if let Ok(uri) = Url::from_file_path(file) {
                        ret.push(Location {
                            uri,
                            range: Range {
                                start: begin.to_position_start(rope),
                                end: end.to_position_end(rope),
                            },
                        });
                    } else {
                        eprintln!("Failed to parse URI for {}", file);
                    }
                }
            }
            let result = Some(GotoDefinitionResponse::Array(ret));
            let result = serde_json::to_value(result)
                .expect("Must be able to serialize the GotoDefinitionResponse");
            let resp = Response {
                id,
                result: Some(result),
                error: None,
            };
            connection
                .sender
                .send(Message::Response(resp))
                .map_err(|err| Error::SendError(err.to_string()))?;
            Ok(())
        }
        Err(Error::ExtractRequestError(req)) => Err(Error::ExtractRequestError(req)),
        Err(err) => panic!("{err:?}"),
        // Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
        // Err(ExtractError::MethodMismatch(req)) => req,
    }
}
