use crate::config::FormatConfig;
use anyhow::Result;
use forth_lexer::{parser::Lexer, token::Token};
use lsp_types::{Position, Range, TextEdit};
use ropey::Rope;

/// Formats Forth source code according to the provided configuration
pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Self { config }
    }

    /// Format the entire document and return a TextEdit to replace all content
    pub fn format_document(&self, rope: &Rope) -> Result<Vec<TextEdit>> {
        let source = rope.to_string();
        let formatted = self.format_source(&source)?;

        // Create a single TextEdit that replaces the entire document
        let start = Position::new(0, 0);
        let end = Position::new(rope.len_lines() as u32, 0);

        Ok(vec![TextEdit {
            range: Range::new(start, end),
            new_text: formatted,
        }])
    }

    /// Format the source code string and return formatted string
    pub fn format_source(&self, source: &str) -> Result<String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.parse();

        // Format tokens
        let formatted = if self.config.preserve_definition_newlines {
            self.format_tokens_preserve_newlines(&tokens, source)
        } else {
            self.format_tokens(&tokens)
        };
        Ok(formatted)
    }

    /// Format a colon definition while preserving its internal newlines
    fn format_preserved_definition(
        &self,
        tokens: &[Token],
        colon_idx: usize,
        source: &str,
        output: &mut String,
    ) -> usize {
        let Token::Colon(colon_data) = &tokens[colon_idx] else {
            return colon_idx + 1;
        };

        let indent_str = if self.config.use_spaces {
            " ".repeat(self.config.indent_width)
        } else {
            "\t".to_string()
        };

        self.force_blank_line_between_defns(output);

        // Find matching semicolon
        let mut semicolon_idx = colon_idx + 1;
        while semicolon_idx < tokens.len() {
            if matches!(tokens[semicolon_idx], Token::Semicolon(_)) {
                break;
            }
            semicolon_idx += 1;
        }

        if semicolon_idx < tokens.len() {
            // Extract and preserve original text between : and ;
            let semi_data = tokens[semicolon_idx].get_data();
            let def_text = &source[colon_data.start..semi_data.end];

            // Indent each line
            for (line_idx, line) in def_text.lines().enumerate() {
                if line_idx > 0 {
                    output.push('\n');
                    output.push_str(&indent_str);
                }
                output.push_str(line.trim_start());
            }
            self.force_blank_line_between_defns(output);

            semicolon_idx + 1
        } else {
            // No matching semicolon
            output.push(':');
            colon_idx + 1
        }
    }

    fn force_blank_line_between_defns(&self, output: &mut String) {
        while self.config.blank_line_between_definitions
            && !output.is_empty()
            && !output.ends_with("\n\n")
        {
            output.push('\n');
        }
    }

    /// Check if a word is a defining word that declares a new symbol
    fn is_defining_word(word: &str) -> bool {
        matches!(
            word.to_uppercase().as_str(),
            "CONSTANT"
                | "VARIABLE"
                | "VALUE"
                | "2CONSTANT"
                | "2VARIABLE"
                | "FVARIABLE"
                | "CREATE"
                | "DEFER"
                | "BUFFER:"
        )
    }

    /// Check if a comment is a parenthetical comment (starts with '(')
    fn is_paren_comment(comment: &str) -> bool {
        comment.trim_start().starts_with('(')
    }

    /// Check if a comment is a line comment (starts with '\')
    fn is_line_comment(comment: &str) -> bool {
        comment.trim_start().starts_with('\\')
    }

    /// Format a non-definition token (outside colon definitions)
    fn format_non_definition_token(
        &self,
        token: &Token,
        output: &mut String,
        last_was_defining: &mut bool,
    ) {
        match token {
            Token::Comment(data) | Token::StackComment(data) => {
                let is_line_comment = Self::is_line_comment(data.value);
                let is_paren_comment = Self::is_paren_comment(data.value);

                // Determine if we should add a newline before the comment
                let should_add_newline_before = if is_paren_comment {
                    self.config.newline_before_paren_comments
                } else if is_line_comment {
                    self.config.newline_before_line_comments
                } else {
                    // Stack comments - preserve old behavior (add newline)
                    true
                };

                if should_add_newline_before && !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                } else if !should_add_newline_before
                    && !output.is_empty()
                    && !output.ends_with(' ')
                    && !output.ends_with('\n')
                {
                    // If not adding newline, add space for inline comments
                    output.push(' ');
                }

                output.push_str(data.value);

                // Always add newline after comment in preserve mode
                // (line comments always end lines, paren/stack comments get newlines for readability)
                output.push('\n');
                *last_was_defining = false;
            }
            Token::Word(data) | Token::Number(data) => {
                // Check if this is a defining word
                let is_def_word = if let Token::Word(w) = token {
                    Self::is_defining_word(w.value)
                } else {
                    false
                };

                if !output.is_empty() && !output.ends_with('\n') {
                    output.push(' ');
                }
                output.push_str(data.value);

                // If last token was a defining word, this is the name - add newline after it
                if *last_was_defining {
                    output.push('\n');
                    *last_was_defining = false;
                } else if is_def_word {
                    // Mark that next token will be the name
                    *last_was_defining = true;
                }
            }
            Token::Semicolon(_) => {
                output.push_str(" ;");
                *last_was_defining = false;
            }
            Token::Illegal(_) | Token::Eof(_) => {
                // Skip
                *last_was_defining = false;
            }
            Token::Colon(_) => {
                // Should not be called for colon tokens
                *last_was_defining = false;
            }
        }
    }

    /// Format tokens while preserving newlines within definitions
    fn format_tokens_preserve_newlines(&self, tokens: &[Token], source: &str) -> String {
        let mut output = String::new();
        let mut i = 0;
        let mut last_was_defining = false;

        while i < tokens.len() {
            match &tokens[i] {
                Token::Eof(_) => break,
                Token::Colon(_) => {
                    i = self.format_preserved_definition(tokens, i, source, &mut output);
                    last_was_defining = false;
                }
                _ => {
                    self.format_non_definition_token(
                        &tokens[i],
                        &mut output,
                        &mut last_was_defining,
                    );
                    i += 1;
                }
            }
        }

        // Ensure file ends with newline
        if !output.ends_with('\n') {
            output.push('\n');
        }

        output
    }

    /// Format a list of tokens according to the configuration
    fn format_tokens(&self, tokens: &[Token]) -> String {
        let mut output = String::new();
        let mut indent_level = 0;
        let mut in_definition = false;
        let mut line_start = true;
        let mut prev_was_colon = false;
        let mut is_first_word_after_colon = false;
        let mut just_printed_stack_comment = false;
        let mut awaiting_potential_stack_comment = false;

        let indent_str = if self.config.use_spaces {
            " ".repeat(self.config.indent_width)
        } else {
            "\t".to_string()
        };

        for token in tokens {
            match token {
                Token::Eof(_) => break,

                Token::Colon(_) => {
                    self.force_blank_line_between_defns(&mut output);

                    if !line_start {
                        output.push('\n');
                    }
                    output.push_str(&indent_str.repeat(indent_level));
                    output.push(':');
                    in_definition = true;
                    prev_was_colon = true;
                    is_first_word_after_colon = true;
                    line_start = false;

                    if self.config.space_after_colon {
                        output.push(' ');
                    }

                    if self.config.indent_control_structures {
                        indent_level += 1;
                    }
                }

                Token::Semicolon(_) => {
                    if self.config.indent_control_structures {
                        indent_level = indent_level.saturating_sub(1);
                    }

                    // Always add space before semicolon if there's content before it on the same line
                    if !line_start {
                        output.push_str(&" ".repeat(self.config.word_spacing));
                    }
                    output.push(';');
                    output.push('\n');
                    in_definition = false;
                    prev_was_colon = false;
                    line_start = true;
                }

                Token::StackComment(data) => {
                    // Handle stack comments - they can appear after colon or after first word
                    if awaiting_potential_stack_comment {
                        // Stack comment after definition name
                        if !self.config.stack_comment_on_declaration_line
                            && self.config.indent_control_structures
                        {
                            // Move stack comment to next line
                            output.push('\n');
                            output.push_str(&indent_str.repeat(indent_level));
                            output.push_str(data.value);
                        } else {
                            // Keep on same line
                            output.push(' ');
                            output.push_str(data.value);
                        }
                        just_printed_stack_comment = true;
                        awaiting_potential_stack_comment = false;
                        is_first_word_after_colon = false;
                    } else {
                        // Stack comment elsewhere
                        if !line_start {
                            output.push(' ');
                        }
                        output.push_str(data.value);
                    }
                    prev_was_colon = false;
                }

                Token::Comment(data) => {
                    let is_line_comment = Self::is_line_comment(data.value);
                    let is_paren_comment = Self::is_paren_comment(data.value);

                    // Check if we should force a newline before this comment
                    let should_add_newline_before = if is_paren_comment {
                        self.config.newline_before_paren_comments
                    } else if is_line_comment {
                        self.config.newline_before_line_comments
                    } else {
                        false
                    };

                    if should_add_newline_before {
                        if !line_start {
                            output.push('\n');
                            line_start = true;
                        }
                    } else {
                        // Default behavior: add space before inline comment
                        if !line_start && !prev_was_colon {
                            output.push(' ');
                        }
                    }

                    output.push_str(data.value);

                    // Line comments always end with a newline (they consume the rest of the line)
                    if is_line_comment {
                        output.push('\n');
                        line_start = true;
                    }

                    prev_was_colon = false;
                    is_first_word_after_colon = false;
                    just_printed_stack_comment = false;
                }

                Token::Word(data) | Token::Number(data) => {
                    // If we were waiting for a potential stack comment and got a word instead, add newline first
                    if awaiting_potential_stack_comment && self.config.indent_control_structures {
                        output.push('\n');
                        line_start = true;
                        awaiting_potential_stack_comment = false;
                    }

                    // If we just printed a stack comment, add newline before next word
                    if just_printed_stack_comment && self.config.indent_control_structures {
                        output.push('\n');
                        line_start = true;
                        just_printed_stack_comment = false;
                    }

                    // Handle control structure indentation
                    let word_upper = data.value.to_uppercase();
                    let is_control_start =
                        matches!(word_upper.as_str(), "IF" | "DO" | "BEGIN" | "CASE" | "OF");
                    let is_control_mid = matches!(word_upper.as_str(), "ELSE");
                    let is_control_end = matches!(
                        word_upper.as_str(),
                        "THEN" | "LOOP" | "+LOOP" | "UNTIL" | "REPEAT" | "ENDCASE" | "ENDOF"
                    );

                    // Add newline BEFORE control structures if indentation is enabled
                    if self.config.indent_control_structures
                        && in_definition
                        && !line_start
                        && !is_first_word_after_colon
                        && (is_control_start || is_control_mid || is_control_end)
                    {
                        output.push('\n');
                        line_start = true;

                        // Decrease indent for mid/end control structures
                        if is_control_mid || is_control_end {
                            indent_level = indent_level.saturating_sub(1);
                        }
                    }

                    if line_start {
                        output.push_str(&indent_str.repeat(indent_level));
                        line_start = false;
                    } else if !prev_was_colon {
                        output.push_str(&" ".repeat(self.config.word_spacing));
                    }

                    output.push_str(data.value);
                    prev_was_colon = false;

                    // After first word following colon (definition name)
                    if is_first_word_after_colon {
                        // Mark that we're waiting to see if a stack comment follows
                        is_first_word_after_colon = false;
                        awaiting_potential_stack_comment = true;
                    } else {
                        // Increase indent after control start/mid structures
                        if self.config.indent_control_structures
                            && in_definition
                            && (is_control_start || is_control_mid)
                        {
                            indent_level += 1;
                            output.push('\n');
                            line_start = true;
                        }
                    }
                }

                Token::Illegal(_) => {
                    // Skip illegal tokens
                }
            }
        }

        // Ensure file ends with newline
        if !output.ends_with('\n') {
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let config = FormatConfig {
            indent_control_structures: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ":   add   +  ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": add + ;\n");
    }

    #[test]
    fn test_indent_definition() {
        let config = FormatConfig {
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": square dup * ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": square\n  dup * ;\n");
    }

    #[test]
    fn test_space_after_colon() {
        let config = FormatConfig {
            space_after_colon: false,
            indent_control_structures: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": square dup * ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ":square dup * ;\n");
    }

    #[test]
    fn test_space_before_semicolon() {
        let config = FormatConfig {
            space_before_semicolon: true,
            indent_control_structures: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test 1 2 + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": test 1 2 + ;\n");
    }

    #[test]
    fn test_word_spacing() {
        let config = FormatConfig {
            word_spacing: 2,
            indent_control_structures: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test 1 2 + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": test  1  2  +  ;\n");
    }

    #[test]
    fn test_control_structure_indentation() {
        let config = FormatConfig {
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": abs dup 0 < if negate then ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": abs\n  dup 0 <\n  if\n    negate\n  then ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_multiple_definitions() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);

        let source = ": square dup * ; : cube dup square * ;";
        let formatted = formatter.format_source(source).unwrap();
        // Default adds blank line between definitions
        assert_eq!(
            formatted,
            ": square\n  dup * ;\n\n: cube\n  dup square * ;\n"
        );
    }

    #[test]
    fn test_comments_preserved() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);

        let source = r"\ This is a comment
: test ( a b -- c ) + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert!(formatted.contains(r"\ This is a comment"));
        assert!(formatted.contains("( a b -- c )"));
    }

    #[test]
    fn test_tabs_instead_of_spaces() {
        let config = FormatConfig {
            use_spaces: false,
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": square dup * ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": square\n\tdup * ;\n");
    }

    #[test]
    fn test_custom_indent_width() {
        let config = FormatConfig {
            indent_width: 4,
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": square dup * ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": square\n    dup * ;\n");
    }

    #[test]
    fn test_nested_control_structures() {
        let config = FormatConfig {
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test 0 10 do i 2 mod 0 = if i . then loop ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected =
            ": test\n  0 10\n  do\n    i 2 mod 0 =\n    if\n      i .\n    then\n  loop ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_document_returns_text_edit() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);

        let source = ":   square   dup   *   ;";
        let rope = Rope::from_str(source);
        let edits = formatter.format_document(&rope).unwrap();

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, ": square\n  dup * ;\n");
    }

    #[test]
    fn test_stack_comment_on_declaration_line_default() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);

        let source = ": add ( a b -- c ) + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": add ( a b -- c )\n  + ;\n");
    }

    #[test]
    fn test_stack_comment_on_separate_line() {
        let config = FormatConfig {
            stack_comment_on_declaration_line: false,
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": add ( a b -- c ) + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": add\n  ( a b -- c )\n  + ;\n");
    }

    #[test]
    fn test_stack_comment_separate_line_no_indent() {
        let config = FormatConfig {
            stack_comment_on_declaration_line: false,
            indent_control_structures: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": add ( a b -- c ) + ;";
        let formatted = formatter.format_source(source).unwrap();
        // When indent is disabled, config doesn't apply
        assert_eq!(formatted, ": add ( a b -- c ) + ;\n");
    }

    #[test]
    fn test_regular_comments_unaffected_by_stack_comment_config() {
        let config = FormatConfig {
            stack_comment_on_declaration_line: false,
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = r": test \ inline comment
dup ;";
        let formatted = formatter.format_source(source).unwrap();
        // Regular comments should not be moved
        assert!(formatted.contains(r"\ inline comment"));
    }

    #[test]
    fn test_blank_line_between_definitions_default() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);

        let source = ": square dup * ; : cube dup square * ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should have blank line between definitions
        assert_eq!(
            formatted,
            ": square\n  dup * ;\n\n: cube\n  dup square * ;\n"
        );
    }

    #[test]
    fn test_blank_line_between_definitions_disabled() {
        let config = FormatConfig {
            blank_line_between_definitions: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": square dup * ; : cube dup square * ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should NOT have blank line between definitions
        assert_eq!(formatted, ": square\n  dup * ;\n: cube\n  dup square * ;\n");
    }

    #[test]
    fn test_blank_line_three_definitions() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);

        let source = ": a 1 ; : b 2 ; : c 3 ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": a\n  1 ;\n\n: b\n  2 ;\n\n: c\n  3 ;\n");
    }

    #[test]
    fn test_preserve_definition_newlines() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test\n  1 2 +\n  3 4 *\n  + ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should preserve the newlines within the definition
        assert_eq!(formatted, ": test\n  1 2 +\n  3 4 *\n  + ;\n\n");
    }

    #[test]
    fn test_preserve_newlines_multiple_definitions() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": a\n  1\n  2 + ;\n: b\n  dup * ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should preserve newlines and add blank line between
        assert_eq!(formatted, ": a\n  1\n  2 + ;\n\n: b\n  dup * ;\n\n");
    }

    #[test]
    fn test_preserve_newlines_with_comments() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test\n  \\ comment\n  1 2 + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": test\n  \\ comment\n  1 2 + ;\n\n");
    }

    #[test]
    fn test_preserve_non_definition_content() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        // Test that comments, constants, variables outside definitions are preserved
        let source = "\\ File header comment\n10 CONSTANT MAX\n: double dup * ;\n\\ Footer comment";
        let formatted = formatter.format_source(source).unwrap();

        // Should preserve all non-definition content
        assert!(formatted.contains("\\ File header comment"));
        assert!(formatted.contains("10 CONSTANT MAX"));
        assert!(formatted.contains(": double dup * ;"));
        assert!(formatted.contains("\\ Footer comment"));
    }

    #[test]
    fn test_preserve_variables_and_constants() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = "VARIABLE counter\n100 CONSTANT LIMIT\n: increment counter @ 1 + counter ! ;";
        let formatted = formatter.format_source(source).unwrap();

        assert!(formatted.contains("VARIABLE counter"));
        assert!(formatted.contains("100 CONSTANT LIMIT"));
        assert!(formatted.contains(": increment counter @ 1 + counter ! ;"));
    }

    #[test]
    fn test_constants_on_separate_lines() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        // Multiple constants on one line should be split
        let source = "10 CONSTANT MAX 42 CONSTANT ANSWER";
        let formatted = formatter.format_source(source).unwrap();

        // Each constant should be on its own line
        assert!(formatted.contains("10 CONSTANT MAX\n"));
        assert!(formatted.contains("42 CONSTANT ANSWER\n"));
    }

    #[test]
    fn test_inline_paren_comment_preserved_by_default() {
        let config = FormatConfig {
            indent_control_structures: false,
            newline_before_paren_comments: false, // default
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": add ( regular comment ) + ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should keep comment inline, not force newline before it
        assert_eq!(formatted, ": add ( regular comment ) + ;\n");
    }

    #[test]
    fn test_inline_line_comment_preserved_by_default() {
        let config = FormatConfig {
            indent_control_structures: false,
            newline_before_line_comments: false, // default
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test 1 2 \\ inline comment\n + ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should keep comment inline on same line as code
        assert!(formatted.contains("1 2 \\ inline comment\n"));
    }

    #[test]
    fn test_newline_before_paren_comments_when_enabled() {
        let config = FormatConfig {
            indent_control_structures: false,
            newline_before_paren_comments: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": add ( regular paren comment ) + ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should force newline before paren comment
        assert!(formatted.contains(": add\n( regular paren comment )"));
    }

    #[test]
    fn test_newline_before_line_comments_when_enabled() {
        let config = FormatConfig {
            indent_control_structures: false,
            newline_before_line_comments: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test 1 2 \\ inline comment\n + ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should force newline before line comment
        assert!(formatted.contains("1 2\n\\ inline comment\n"));
    }

    #[test]
    fn test_preserve_newlines_mode_respects_comment_config() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            newline_before_paren_comments: false,
            newline_before_line_comments: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);

        let source = ": test 1 2 ( inline paren ) + \\ inline line\n 3 ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should preserve inline comments even in preserve mode
        assert!(formatted.contains("( inline paren )"));
        assert!(formatted.contains("\\ inline line"));
    }
}
