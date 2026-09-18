use crate::config::{BlankLinesConfig, FormatConfig};
use anyhow::Result;
use forth_lexer::{parser::Lexer, token::Token};
use lsp_types::{Position, Range, TextEdit};
use ropey::Rope;

/// Formatter trait for formatting Forth source code
pub trait Formatter {
    /// Format the entire document and return a TextEdit to replace all content
    fn format_document(&self, rope: &Rope) -> Result<Vec<TextEdit>>;

    /// Format the source code string and return formatted string
    fn format_source(&self, source: &str) -> Result<String>;
}

/// Creates a formatter based on if Config.format.enabled is true
pub fn create_formatter(config: &crate::config::Config) -> Box<dyn Formatter> {
    if config.format.enabled {
        eprintln!("[DEBUG] Using DefaultFormatter");
        Box::new(DefaultFormatter::new(
            config.format.clone(),
            config.builtin.skip_words.clone(),
        ))
    } else {
        eprintln!("[DEBUG] Using NullFormatter");
        Box::new(NullFormatter::new())
    }
}

/// Formats Forth source code according to the provided configuration
pub struct DefaultFormatter {
    config: FormatConfig,
    skip_words: Vec<String>,
}

impl DefaultFormatter {
    pub fn new(config: FormatConfig, skip_words: Vec<String>) -> Self {
        Self { config, skip_words }
    }

    /// Check if a word is a skip word that takes an argument
    fn is_skip_word(&self, word: &str) -> bool {
        crate::config::is_skip_word(&self.skip_words, word)
    }
}

impl Formatter for DefaultFormatter {
    fn format_document(&self, rope: &Rope) -> Result<Vec<TextEdit>> {
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

    fn format_source(&self, source: &str) -> Result<String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.parse();

        // Format tokens
        let mut formatted = if self.config.preserve_definition_newlines {
            self.format_tokens_preserve_newlines(&tokens, source)
        } else {
            self.format_tokens(&tokens, source)
        };

        // Independent of the setting value, all blank lines in the end of file must be removed
        // (but the file should end with newline character)
        while formatted.ends_with(' ')
            || formatted.ends_with('\t')
            || formatted.ends_with('\r')
            || formatted.ends_with('\n')
        {
            formatted.pop();
        }
        if !formatted.is_empty() {
            formatted.push('\n');
        }

        Ok(formatted)
    }
}

impl DefaultFormatter {
    fn count_gap_blank_lines(gap: &str) -> usize {
        let newlines = gap.chars().filter(|&c| c == '\n').count();
        if newlines >= 2 { newlines - 1 } else { 0 }
    }

    /// Source text between token `i - 1` and token `i` ("" if the ranges overlap)
    fn token_gap<'a>(tokens: &[Token], source: &'a str, i: usize) -> &'a str {
        let prev_end = tokens[i - 1].get_data().end;
        let curr_start = tokens[i].get_data().start;
        if curr_start >= prev_end {
            &source[prev_end..curr_start]
        } else {
            ""
        }
    }

    fn is_doc_comment_start(tokens: &[Token], source: &str, idx: usize) -> bool {
        if !matches!(tokens[idx], Token::Comment(_) | Token::StackComment(_)) {
            return false;
        }
        if idx > 0 && matches!(tokens[idx - 1], Token::Comment(_) | Token::StackComment(_)) {
            let prev_gap = Self::token_gap(tokens, source, idx);
            if Self::count_gap_blank_lines(prev_gap) == 0 {
                return false;
            }
        }
        let mut curr = idx;
        while curr < tokens.len() {
            if curr + 1 >= tokens.len() {
                return false;
            }
            let gap = Self::token_gap(tokens, source, curr + 1);
            if Self::count_gap_blank_lines(gap) > 0 {
                return false;
            }
            match &tokens[curr + 1] {
                Token::Comment(_) | Token::StackComment(_) => {
                    curr += 1;
                }
                Token::Colon(_) => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
        false
    }

    fn is_colon_preceded_by_doc_comments(tokens: &[Token], source: &str, colon_idx: usize) -> bool {
        if colon_idx == 0 {
            return false;
        }
        if !matches!(
            tokens[colon_idx - 1],
            Token::Comment(_) | Token::StackComment(_)
        ) {
            return false;
        }
        let gap = Self::token_gap(tokens, source, colon_idx);
        Self::count_gap_blank_lines(gap) == 0
    }

    fn target_blank_lines_before_definition(
        &self,
        seen_first_definition: bool,
        source_blank_lines: usize,
    ) -> usize {
        match self.config.blank_lines {
            BlankLinesConfig::No => {
                if seen_first_definition && self.config.blank_line_between_definitions {
                    1
                } else {
                    0
                }
            }
            BlankLinesConfig::Collapse => {
                if (seen_first_definition && self.config.blank_line_between_definitions)
                    || source_blank_lines > 0
                {
                    1
                } else {
                    0
                }
            }
            BlankLinesConfig::Preserve => {
                if seen_first_definition && self.config.blank_line_between_definitions {
                    std::cmp::max(1, source_blank_lines)
                } else {
                    source_blank_lines
                }
            }
        }
    }

    fn set_trailing_blank_lines(output: &mut String, count: usize) {
        if output.is_empty() {
            return;
        }
        while output.ends_with(' ') || output.ends_with('\t') {
            output.pop();
        }
        while output.ends_with('\n') {
            output.pop();
        }
        for _ in 0..=count {
            output.push('\n');
        }
    }

    /// Adjust the trailing blank lines in `output` based on the source gap
    /// before token `i`, honoring the `blank_lines` setting. Returns true if
    /// the adjustment ran, i.e. `output` now ends at a line start.
    fn apply_gap_blank_lines(
        &self,
        tokens: &[Token],
        source: &str,
        i: usize,
        seen_first_definition: bool,
        output: &mut String,
    ) -> bool {
        if i == 0 || output.is_empty() {
            return false;
        }
        let gap = Self::token_gap(tokens, source, i);
        let newlines = gap.chars().filter(|&c| c == '\n').count();
        let source_blank_lines = Self::count_gap_blank_lines(gap);

        let target = if Self::is_doc_comment_start(tokens, source, i) {
            let force_blank = seen_first_definition || !output.trim().is_empty();
            self.target_blank_lines_before_definition(force_blank, source_blank_lines)
        } else if matches!(tokens[i], Token::Colon(_)) {
            if Self::is_colon_preceded_by_doc_comments(tokens, source, i) {
                0
            } else {
                self.target_blank_lines_before_definition(seen_first_definition, source_blank_lines)
            }
        } else if newlines > 0 {
            let in_doc_block = matches!(tokens[i - 1], Token::Comment(_) | Token::StackComment(_))
                && matches!(tokens[i], Token::Comment(_) | Token::StackComment(_))
                && source_blank_lines == 0;
            if in_doc_block {
                0
            } else {
                match self.config.blank_lines {
                    BlankLinesConfig::No => 0,
                    BlankLinesConfig::Collapse => usize::from(source_blank_lines > 0),
                    BlankLinesConfig::Preserve => source_blank_lines,
                }
            }
        } else {
            return false;
        };
        Self::set_trailing_blank_lines(output, target);
        true
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

        // Find matching semicolon
        let mut semicolon_idx = colon_idx + 1;
        while semicolon_idx < tokens.len() {
            if matches!(tokens[semicolon_idx], Token::Semicolon(_)) {
                break;
            }
            semicolon_idx += 1;
        }

        if semicolon_idx < tokens.len() {
            // Extract text between : and ;
            let semi_data = tokens[semicolon_idx].get_data();
            let def_text = &source[colon_data.start..semi_data.end];

            let mut consecutive_empty_lines = 0;
            for (line_idx, line) in def_text.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    if line_idx == 0 {
                        continue;
                    }
                    consecutive_empty_lines += 1;
                    match self.config.blank_lines {
                        BlankLinesConfig::No => {
                            // "The 'preserve_definition_newlines' option when true should not be applied to empty lines."
                            // Skip empty lines
                        }
                        BlankLinesConfig::Collapse => {
                            if consecutive_empty_lines == 1 {
                                output.push('\n');
                            }
                        }
                        BlankLinesConfig::Preserve => {
                            output.push('\n');
                        }
                    }
                } else {
                    consecutive_empty_lines = 0;
                    if line_idx > 0 {
                        output.push('\n');
                        output.push_str(&indent_str);
                    }
                    output.push_str(line.trim_start());
                }
            }

            semicolon_idx + 1
        } else {
            // No matching semicolon
            output.push(':');
            colon_idx + 1
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
        last_was_skip: &mut bool,
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
                *last_was_skip = false;
            }
            Token::Word(data) | Token::Number(data) => {
                // Check if this is a defining word
                let is_def_word = if let Token::Word(w) = token {
                    Self::is_defining_word(w.value)
                } else {
                    false
                };
                let is_skip = if let Token::Word(w) = token {
                    self.is_skip_word(w.value)
                } else {
                    false
                };

                if !output.is_empty() && !output.ends_with('\n') {
                    output.push(' ');
                }
                output.push_str(data.value);

                // If last token was a defining word or skip word, this is the name/argument - add newline after it
                if *last_was_defining || *last_was_skip {
                    output.push('\n');
                    *last_was_defining = false;
                    *last_was_skip = false;
                } else if is_def_word {
                    // Mark that next token will be the name
                    *last_was_defining = true;
                } else if is_skip {
                    // Mark that next token will be the argument
                    *last_was_skip = true;
                }
            }
            Token::Semicolon(_) => {
                output.push_str(" ;");
                *last_was_defining = false;
                *last_was_skip = false;
            }
            Token::Illegal(_) | Token::Eof(_) => {
                // Skip
                *last_was_defining = false;
                *last_was_skip = false;
            }
            Token::Colon(_) => {
                // Should not be called for colon tokens
                *last_was_defining = false;
                *last_was_skip = false;
            }
        }
    }

    /// Format tokens while preserving newlines within definitions
    fn format_tokens_preserve_newlines(&self, tokens: &[Token], source: &str) -> String {
        let mut output = String::new();
        let mut i = 0;
        let mut last_was_defining = false;
        let mut seen_first_definition = false;
        let mut last_was_skip = false;

        while i < tokens.len() {
            match &tokens[i] {
                Token::Eof(_) => break,
                Token::Colon(_) => {
                    self.apply_gap_blank_lines(
                        tokens,
                        source,
                        i,
                        seen_first_definition,
                        &mut output,
                    );

                    seen_first_definition = true;
                    i = self.format_preserved_definition(tokens, i, source, &mut output);
                    last_was_defining = false;
                    last_was_skip = false;
                }
                _ => {
                    self.apply_gap_blank_lines(
                        tokens,
                        source,
                        i,
                        seen_first_definition,
                        &mut output,
                    );

                    self.format_non_definition_token(
                        &tokens[i],
                        &mut output,
                        &mut last_was_defining,
                        &mut last_was_skip,
                    );
                    i += 1;
                }
            }
        }

        output
    }

    /// Format a list of tokens according to the configuration
    fn format_tokens(&self, tokens: &[Token], source: &str) -> String {
        let mut output = String::new();
        let mut indent_level = 0;
        let mut in_definition = false;
        let mut line_start = true;
        let mut prev_was_colon = false;
        let mut is_first_word_after_colon = false;
        let mut just_printed_stack_comment = false;
        let mut awaiting_potential_stack_comment = false;
        let mut seen_first_definition = false;
        let mut just_printed_endcase = false;
        let mut last_was_defining = false;
        let mut last_was_skip = false;

        let indent_str = if self.config.use_spaces {
            " ".repeat(self.config.indent_width)
        } else {
            "\t".to_string()
        };

        for (i, token) in tokens.iter().enumerate() {
            if !in_definition
                && self.apply_gap_blank_lines(tokens, source, i, seen_first_definition, &mut output)
            {
                line_start = true;
            }

            match token {
                Token::Eof(_) => break,

                Token::Colon(_) => {
                    seen_first_definition = true;

                    if !line_start {
                        output.push('\n');
                    }
                    output.push_str(&indent_str.repeat(indent_level));
                    output.push(':');
                    in_definition = true;
                    prev_was_colon = true;
                    is_first_word_after_colon = true;
                    line_start = false;
                    last_was_defining = false;
                    last_was_skip = false;

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
                    just_printed_endcase = false;
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
                        "THEN"
                            | "LOOP"
                            | "+LOOP"
                            | "UNTIL"
                            | "REPEAT"
                            | "ENDCASE"
                            | "ENDOF"
                            | "AGAIN"
                    );

                    // If we just printed endcase and got a word/number instead of semicolon, add newline first
                    if just_printed_endcase && self.config.indent_control_structures {
                        output.push('\n');
                        line_start = true;
                        just_printed_endcase = false;
                    }

                    // Decrease indent for mid/end control structures and add newline before control structures
                    if self.config.indent_control_structures
                        && in_definition
                        && !is_first_word_after_colon
                    {
                        // Decrease indent for mid/end control structures
                        if is_control_mid || is_control_end {
                            indent_level = indent_level.saturating_sub(1);
                        }

                        // Add newline BEFORE control structures if indentation is enabled (except OF which stays on same line as value)
                        if !line_start
                            && word_upper != "OF"
                            && (is_control_start || is_control_mid || is_control_end)
                        {
                            output.push('\n');
                            line_start = true;
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

                    // Handle defining words and skip words outside definitions
                    if !in_definition {
                        if last_was_defining || last_was_skip {
                            output.push('\n');
                            line_start = true;
                            last_was_defining = false;
                            last_was_skip = false;
                        } else if let Token::Word(w) = token {
                            if Self::is_defining_word(w.value) {
                                last_was_defining = true;
                            } else if self.is_skip_word(w.value) {
                                last_was_skip = true;
                            }
                        }
                    }

                    // After first word following colon (definition name)
                    if is_first_word_after_colon {
                        // Mark that we're waiting to see if a stack comment follows
                        is_first_word_after_colon = false;
                        awaiting_potential_stack_comment = true;
                    } else {
                        // Increase indent after control start/mid structures, or add newline after ENDOF / track ENDCASE
                        if self.config.indent_control_structures && in_definition {
                            if is_control_start || is_control_mid {
                                indent_level += 1;
                                output.push('\n');
                                line_start = true;
                            } else if word_upper == "ENDOF" {
                                output.push('\n');
                                line_start = true;
                            } else if word_upper == "ENDCASE" {
                                just_printed_endcase = true;
                            }
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

pub struct NullFormatter;

impl NullFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for NullFormatter {
    fn format_document(&self, rope: &Rope) -> Result<Vec<TextEdit>> {
        let source = rope.to_string();
        Ok(vec![TextEdit {
            range: Range::new(
                Position::new(0, 0),
                Position::new(rope.len_lines() as u32, 0),
            ),
            new_text: source,
        }])
    }

    fn format_source(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": abs dup 0 < if negate then ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": abs\n  dup 0 <\n  if\n    negate\n  then ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_multiple_definitions() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test 0 10 do i 2 mod 0 = if i . then loop ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected =
            ": test\n  0 10\n  do\n    i 2 mod 0 =\n    if\n      i .\n    then\n  loop ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_document_returns_text_edit() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ":   square   dup   *   ;";
        let rope = Rope::from_str(source);
        let edits = formatter.format_document(&rope).unwrap();

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, ": square\n  dup * ;\n");
    }

    #[test]
    fn test_stack_comment_on_declaration_line_default() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = r": test \ inline comment
dup ;";
        let formatted = formatter.format_source(source).unwrap();
        // Regular comments should not be moved
        assert!(formatted.contains(r"\ inline comment"));
    }

    #[test]
    fn test_blank_line_between_definitions_default() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": square dup * ; : cube dup square * ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should NOT have blank line between definitions
        assert_eq!(formatted, ": square\n  dup * ;\n: cube\n  dup square * ;\n");
    }

    #[test]
    fn test_blank_line_three_definitions() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test\n  1 2 +\n  3 4 *\n  + ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should preserve the newlines within the definition
        assert_eq!(formatted, ": test\n  1 2 +\n  3 4 *\n  + ;\n");
    }

    #[test]
    fn test_preserve_newlines_multiple_definitions() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": a\n  1\n  2 + ;\n: b\n  dup * ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should preserve newlines and add blank line between
        assert_eq!(formatted, ": a\n  1\n  2 + ;\n\n: b\n  dup * ;\n");
    }

    #[test]
    fn test_preserve_newlines_with_comments() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test\n  \\ comment\n  1 2 + ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": test\n  \\ comment\n  1 2 + ;\n");
    }

    #[test]
    fn test_preserve_non_definition_content() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

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
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test 1 2 ( inline paren ) + \\ inline line\n 3 ;";
        let formatted = formatter.format_source(source).unwrap();
        // Should preserve inline comments even in preserve mode
        assert!(formatted.contains("( inline paren )"));
        assert!(formatted.contains("\\ inline line"));
    }

    #[test]
    fn test_case_statement_formatting() {
        let config = FormatConfig {
            indent_control_structures: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": foo case 1 of exit endof 2 of exit endof endcase drop ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": foo\n  case\n    1 of\n      exit\n    endof\n    2 of\n      exit\n    endof\n  endcase\n  drop ;\n";
        assert_eq!(formatted, expected);

        // now same but with ; immediately after endcase
        let source = ": foo case 1 of exit endof 2 of exit endof endcase ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": foo\n  case\n    1 of\n      exit\n    endof\n    2 of\n      exit\n    endof\n  endcase ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_doc_comment_block_before_definition() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source =
            ": word1 1 ;\n\\ here comes\n\\ some comment block\n\\ documenting word\n: word2 2 ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": word1\n  1 ;\n\n\\ here comes\n\\ some comment block\n\\ documenting word\n: word2\n  2 ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_first_definition_doc_comment_no_blank_line() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "\\ here comes\n\\ doc for first word\n: word1 1 ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = "\\ here comes\n\\ doc for first word\n: word1\n  1 ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_first_definition_doc_comment_blank_line_when_not_first_line() {
        let config = FormatConfig::default();
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "10 CONSTANT X\n\\ here comes\n\\ doc for first word\n: word1 1 ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = "10 CONSTANT X\n\n\\ here comes\n\\ doc for first word\n: word1\n  1 ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_first_definition_doc_comment_blank_line_when_not_first_line_blank_lines_no() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::No,
            blank_line_between_definitions: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "10 CONSTANT X\n\\ here comes\n\\ doc for first word\n: word1 1 ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = "10 CONSTANT X\n\n\\ here comes\n\\ doc for first word\n: word1\n  1 ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_blank_lines_no() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::No,
            blank_line_between_definitions: false,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "\\ Section 1\n\n\n\\ Section 2\n\n10 CONSTANT X\n\n: a 1 ;\n\n: b 2 ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = "\\ Section 1\n\\ Section 2\n10 CONSTANT X\n: a\n  1 ;\n: b\n  2 ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_blank_lines_no_with_blank_line_between_definitions() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::No,
            blank_line_between_definitions: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source =
            "\\ Section 1\n\n\n\\ Section 2\n\n: a 1 ;\n\n\\ doc for b\n: b 2 ;\n\n: c 3 ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected =
            "\\ Section 1\n\\ Section 2\n: a\n  1 ;\n\n\\ doc for b\n: b\n  2 ;\n\n: c\n  3 ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_blank_lines_collapse_default() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::Collapse,
            blank_line_between_definitions: false,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "\\ Section 1\n\n\n\n\\ Section 2\n\n10 CONSTANT X";
        let formatted = formatter.format_source(source).unwrap();
        let expected = "\\ Section 1\n\n\\ Section 2\n\n10 CONSTANT X\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_blank_lines_preserve() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::Preserve,
            blank_line_between_definitions: false,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "\\ Section 1\n\n\n\n\\ Section 2\n\n10 CONSTANT X";
        let formatted = formatter.format_source(source).unwrap();
        let expected = "\\ Section 1\n\n\n\n\\ Section 2\n\n10 CONSTANT X\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_preserve_definition_newlines_empty_lines_removed_when_no() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            blank_lines: BlankLinesConfig::No,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test\n  1 2 +\n\n\n  3 4 *\n  + ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": test\n  1 2 +\n  3 4 *\n  + ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_preserve_definition_newlines_empty_lines_collapsed_when_collapse() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            blank_lines: BlankLinesConfig::Collapse,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test\n  1 2 +\n\n\n  3 4 *\n  + ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": test\n  1 2 +\n\n  3 4 *\n  + ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_preserve_definition_newlines_empty_lines_kept_when_preserve() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            blank_lines: BlankLinesConfig::Preserve,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": test\n  1 2 +\n\n\n  3 4 *\n  + ;";
        let formatted = formatter.format_source(source).unwrap();
        let expected = ": test\n  1 2 +\n\n\n  3 4 *\n  + ;\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_trailing_blank_lines_removed_at_eof() {
        // Test with all three BlankLinesConfig variants
        for variant in [
            BlankLinesConfig::No,
            BlankLinesConfig::Collapse,
            BlankLinesConfig::Preserve,
        ] {
            let config = FormatConfig {
                blank_lines: variant,
                ..Default::default()
            };
            let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

            let source = ": test 1 2 + ;\n\n\n\n\n";
            let formatted = formatter.format_source(source).unwrap();
            assert_eq!(
                formatted, ": test\n  1 2 + ;\n",
                "Failed for blank_lines variant {:?}",
                variant
            );
        }
    }

    #[test]
    fn test_preserve_multiple_blank_lines_before_word_definition() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::Preserve,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": first 1 ;\n\n\n\n: second 2 ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": first\n  1 ;\n\n\n\n: second\n  2 ;\n");
    }

    #[test]
    fn test_preserve_multiple_blank_lines_before_doc_comment_block() {
        let config = FormatConfig {
            blank_lines: BlankLinesConfig::Preserve,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": first 1 ;\n\n\n\\ doc comment\n: second 2 ;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(
            formatted,
            ": first\n  1 ;\n\n\n\\ doc comment\n: second\n  2 ;\n"
        );
    }

    #[test]
    fn test_preserve_multiple_blank_lines_between_definitions_preserve_newlines_mode() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            blank_lines: BlankLinesConfig::Preserve,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = ": first\n  1\n;\n\n\n: second\n  2\n;";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(formatted, ": first\n  1\n  ;\n\n\n: second\n  2\n  ;\n");
    }

    #[test]
    fn test_format_skip_words_on_separate_lines() {
        let config = FormatConfig {
            indent_control_structures: false,
            preserve_definition_newlines: false,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "require foo.4th require bar.4th\n3 constant qux";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(
            formatted,
            "require foo.4th\nrequire bar.4th\n3 constant qux\n"
        );
    }

    #[test]
    fn test_format_require_preserved_newlines_mode() {
        let config = FormatConfig {
            preserve_definition_newlines: true,
            ..Default::default()
        };
        let formatter = DefaultFormatter::new(config, crate::config::default_skip_words());

        let source = "require foo.4th\nrequire bar.4th\n3 constant qux";
        let formatted = formatter.format_source(source).unwrap();
        assert_eq!(
            formatted,
            "require foo.4th\nrequire bar.4th\n3 constant qux\n"
        );
    }
}
