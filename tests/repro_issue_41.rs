//! Reproduction attempt for issue #41:
//! "forth-lsp auto-appends a semicolon when you type a colon inside a comment"
//!
//! This drives the real server binary over stdio exactly like an editor would:
//! initialize, open a document that contains a `:` inside a comment, then ask
//! for the things an editor asks for as you type — completion at the colon and
//! a full document formatting. We then inspect everything the server sends back
//! and check whether any of it would insert a stray `;`.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

fn frame(msg: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg)
}

fn send(stdin: &mut ChildStdin, msg: &str) {
    stdin.write_all(frame(msg).as_bytes()).unwrap();
    stdin.flush().unwrap();
}

/// Read one Content-Length framed message; None on EOF.
fn read_message(reader: &mut impl BufRead) -> Option<String> {
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().ok()?;
        }
    }
    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

/// Read messages until one whose body contains `needle` (an `"id":N` marker),
/// collecting every message seen along the way.
fn read_until(reader: &mut impl BufRead, needle: &str, collected: &mut Vec<String>) -> String {
    for _ in 0..50 {
        match read_message(reader) {
            Some(msg) => {
                let hit = msg.contains(needle);
                collected.push(msg.clone());
                if hit {
                    return msg;
                }
            }
            None => break,
        }
    }
    panic!("did not receive a message containing {needle}");
}

fn shutdown(child: &mut Child, stdin: &mut ChildStdin) {
    send(stdin, r#"{"jsonrpc":"2.0","id":99,"method":"shutdown"}"#);
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    let _ = child.wait();
}

#[test]
fn typing_colon_inside_a_comment_does_not_append_semicolon() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn forth-lsp");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // 1. initialize / initialized
    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"workspaceFolders":null}}"#,
    );
    let mut seen = Vec::new();
    read_until(&mut stdout, r#""id":1"#, &mut seen);
    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    // 2. Open a doc with a `:` inside both a line comment and a paren comment.
    //    The colon on line 0 sits inside a `\` line comment; line 1 puts one
    //    inside a `( ... )` comment. A real definition on line 2 is the control.
    let uri = "file:///tmp/repro41.forth";
    let text = "\\ make a thing : like this\n( note the : here ) \n: real dup * ;\n";
    let did_open = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": { "uri": uri, "languageId": "forth", "version": 1, "text": text }
        }
    })
    .to_string();
    send(&mut stdin, &did_open);

    // 3. Completion right after the `:` typed inside the line comment
    //    (line 0, character 16 — just past the colon).
    let completion = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/completion",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 16 }
        }
    })
    .to_string();
    send(&mut stdin, &completion);
    let completion_resp = read_until(&mut stdout, r#""id":2"#, &mut seen);

    // 4. Full-document formatting — the other path that can rewrite text.
    let formatting = serde_json::json!({
        "jsonrpc": "2.0", "id": 3, "method": "textDocument/formatting",
        "params": {
            "textDocument": { "uri": uri },
            "options": { "tabSize": 2, "insertSpaces": true }
        }
    })
    .to_string();
    send(&mut stdin, &formatting);
    let formatting_resp = read_until(&mut stdout, r#""id":3"#, &mut seen);

    shutdown(&mut child, &mut stdin);

    eprintln!("--- completion response ---\n{completion_resp}");
    eprintln!("--- formatting response ---\n{formatting_resp}");

    // Completions must be suppressed entirely inside a comment: the result is
    // null (or, defensively, an empty list with no item that inserts a `;`).
    let completion: serde_json::Value = serde_json::from_str(&completion_resp).unwrap();
    let result = &completion["result"];
    if let Some(items) = result.as_array() {
        assert!(
            items.is_empty(),
            "expected no completions inside a comment, got: {result}"
        );
    } else {
        assert!(
            result.is_null(),
            "expected null completion result inside a comment, got: {result}"
        );
    }

    // Formatting must not turn a comment-embedded colon into a `: ... ;` def.
    let formatting: serde_json::Value = serde_json::from_str(&formatting_resp).unwrap();
    if let Some(edits) = formatting["result"].as_array() {
        for edit in edits {
            let new_text = edit["newText"].as_str().unwrap_or("");
            // The one legitimate `;` is the real definition on line 2.
            let semicolons = new_text.matches(';').count();
            assert_eq!(
                semicolons, 1,
                "formatting introduced extra semicolons; output was:\n{new_text}"
            );
        }
    }
}
