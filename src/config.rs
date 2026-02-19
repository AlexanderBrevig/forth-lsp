use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::words::Word;

/// Configuration for the Forth LSP server
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub format: FormatConfig,

    #[serde(default)]
    pub builtin: BuiltinConfig,
}

/// Formatter configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormatConfig {
    /// Number of spaces for indentation
    #[serde(default = "default_indent_width")]
    pub indent_width: usize,

    /// Use spaces instead of tabs
    #[serde(default = "default_use_spaces")]
    pub use_spaces: bool,

    /// Add space after colon in definitions (`: word` vs `:word`)
    #[serde(default = "default_true")]
    pub space_after_colon: bool,

    /// Add space before semicolon in definitions (`word ;` vs `word;`)
    #[serde(default)]
    pub space_before_semicolon: bool,

    /// Spaces between words (1 or more)
    #[serde(default = "default_word_spacing")]
    pub word_spacing: usize,

    /// Indent control structures (IF/THEN, DO/LOOP, etc.)
    #[serde(default = "default_true")]
    pub indent_control_structures: bool,

    /// Keep stack comments on same line as colon declaration
    /// When true: `: word ( a b -- c )`
    /// When false: `: word\n  ( a b -- c )`
    #[serde(default = "default_true")]
    pub stack_comment_on_declaration_line: bool,

    /// Preserve newlines within colon definitions from original source
    /// When true, keeps manual line breaks inside `: ... ;` blocks
    #[serde(default)]
    pub preserve_definition_newlines: bool,

    /// Add blank line before each colon definition (except first)
    /// Helps visually separate definitions
    #[serde(default = "default_true")]
    pub blank_line_between_definitions: bool,

    /// Add newline before parenthetical comments `( comment )`
    /// When false (default): preserves original whitespace
    /// When true: forces newline before paren comments
    #[serde(default)]
    pub newline_before_paren_comments: bool,

    /// Add newline before line comments `\ comment`
    /// When false (default): preserves original whitespace
    /// When true: forces newline before line comments
    #[serde(default)]
    pub newline_before_line_comments: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_width: default_indent_width(),
            use_spaces: default_use_spaces(),
            space_after_colon: default_true(),
            space_before_semicolon: false,
            word_spacing: default_word_spacing(),
            indent_control_structures: default_true(),
            stack_comment_on_declaration_line: default_true(),
            preserve_definition_newlines: false,
            blank_line_between_definitions: default_true(),
            newline_before_paren_comments: false,
            newline_before_line_comments: false,
        }
    }
}

/// A custom word definition
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct CustomWord {
    /// The word name (e.g., "DUP", "LOADFROM")
    pub word: String,

    /// Stack effect notation (e.g., "( x -- x x )")
    #[serde(default)]
    pub stack: Option<String>,

    /// Description of what the word does
    #[serde(default)]
    pub description: Option<String>,
}

/// Custom builtin words configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BuiltinConfig {
    /// Additional builtin words specific to the user's Forth implementation
    /// Supports both simple string format and detailed metadata format
    #[serde(default)]
    pub words: Vec<CustomWord>,

    /// Paths to files containing whitespace-separated word lists
    /// (e.g. output of `gforth -e 'words bye'`)
    /// Paths are relative to workspace root or absolute
    #[serde(default)]
    pub word_files: Vec<String>,
}

impl BuiltinConfig {
    /// Read word files and return parsed custom words.
    /// Paths are resolved relative to `workspace_root` unless absolute.
    pub fn load_words_from_files(&self, workspace_root: &str) -> Vec<CustomWord> {
        let root = PathBuf::from(workspace_root);
        let mut words = Vec::new();
        for file_path in &self.word_files {
            let path = if Path::new(file_path).is_absolute() {
                PathBuf::from(file_path)
            } else {
                root.join(file_path)
            };
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    for token in content.split_whitespace() {
                        words.push(CustomWord {
                            word: token.to_string(),
                            stack: None,
                            description: None,
                        });
                    }
                    eprintln!("Loaded word file {:?}", path);
                }
                Err(e) => {
                    eprintln!("Failed to read word file {:?}: {}", path, e);
                }
            }
        }
        words
    }

    /// Convert all custom words (inline + from files) into leaked `Word<'static>` references
    /// suitable for pushing into `Words.words`. Since the LSP runs for the process lifetime,
    /// leaking is appropriate.
    pub fn into_static_words(&self, workspace_root: Option<&str>) -> Vec<&'static Word<'static>> {
        let mut all_custom = self.words.clone();
        if let Some(root) = workspace_root {
            all_custom.extend(self.load_words_from_files(root));
        }
        all_custom
            .into_iter()
            .map(|cw| {
                let token: &'static str = Box::leak(cw.word.into_boxed_str());
                let stack: &'static str = match cw.stack {
                    Some(s) => Box::leak(s.into_boxed_str()),
                    None => "",
                };
                let help: &'static str = match cw.description {
                    Some(s) => Box::leak(s.into_boxed_str()),
                    None => "",
                };
                let word = Box::new(Word {
                    doc: "",
                    token,
                    stack,
                    help,
                });
                &*Box::leak(word)
            })
            .collect()
    }
}

fn default_indent_width() -> usize {
    2
}

fn default_use_spaces() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_word_spacing() -> usize {
    1
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Try to load config from default locations:
    /// 1. .forth-lsp.toml in workspace root
    /// 2. Default config if not found
    pub fn load_from_workspace(workspace_root: Option<&str>) -> Self {
        if let Some(root) = workspace_root {
            let config_path = PathBuf::from(root).join(".forth-lsp.toml");
            if config_path.exists() {
                match Self::from_file(&config_path) {
                    Ok(config) => {
                        eprintln!("Loaded config from {:?}", config_path);
                        return config;
                    }
                    Err(e) => {
                        eprintln!("Failed to load config from {:?}: {}", config_path, e);
                    }
                }
            }
        }
        eprintln!("Using default configuration");
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.format.indent_width, 2);
        assert!(config.format.use_spaces);
        assert!(config.format.space_after_colon);
        assert!(!config.format.space_before_semicolon);
        assert_eq!(config.format.word_spacing, 1);
        assert!(config.format.indent_control_structures);
        assert!(config.builtin.words.is_empty());
    }

    #[test]
    fn test_parse_format_config() {
        let toml_content = r#"
            [format]
            indent_width = 4
            use_spaces = false
            space_after_colon = false
            space_before_semicolon = true
            word_spacing = 2
            indent_control_structures = false
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.format.indent_width, 4);
        assert!(!config.format.use_spaces);
        assert!(!config.format.space_after_colon);
        assert!(config.format.space_before_semicolon);
        assert_eq!(config.format.word_spacing, 2);
        assert!(!config.format.indent_control_structures);
    }

    #[test]
    fn test_parse_builtin_config_simple() {
        let toml_content = r#"
            [[builtin.words]]
            word = "LOADFROM"

            [[builtin.words]]
            word = "CUSTOMWORD"

            [[builtin.words]]
            word = "MYSTACK"
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.builtin.words.len(), 3);
        assert_eq!(config.builtin.words[0].word, "LOADFROM");
        assert_eq!(config.builtin.words[1].word, "CUSTOMWORD");
        assert_eq!(config.builtin.words[2].word, "MYSTACK");
    }

    #[test]
    fn test_parse_builtin_config_with_metadata() {
        let toml_content = r#"
            [[builtin.words]]
            word = "DUP"
            stack = "( x -- x x )"
            description = "Duplicates top of stack"

            [[builtin.words]]
            word = "LOADFROM"
            stack = "( addr -- )"
            description = "Custom load operation"
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.builtin.words.len(), 2);

        let dup = &config.builtin.words[0];
        assert_eq!(dup.word, "DUP");
        assert_eq!(dup.stack, Some("( x -- x x )".to_string()));
        assert_eq!(dup.description, Some("Duplicates top of stack".to_string()));

        let loadfrom = &config.builtin.words[1];
        assert_eq!(loadfrom.word, "LOADFROM");
        assert_eq!(loadfrom.stack, Some("( addr -- )".to_string()));
        assert_eq!(
            loadfrom.description,
            Some("Custom load operation".to_string())
        );
    }

    #[test]
    fn test_parse_builtin_config_mixed() {
        let toml_content = r#"
            [[builtin.words]]
            word = "DUP"
            stack = "( x -- x x )"
            description = "Duplicates top of stack"

            [[builtin.words]]
            word = "SIMPLEWORD"
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.builtin.words.len(), 2);
        assert_eq!(config.builtin.words[0].word, "DUP");
        assert_eq!(config.builtin.words[1].word, "SIMPLEWORD");
        assert_eq!(config.builtin.words[1].stack, None);
        assert_eq!(config.builtin.words[1].description, None);
    }

    #[test]
    fn test_parse_full_config() {
        let toml_content = r#"
            [format]
            indent_width = 4
            use_spaces = true

            [[builtin.words]]
            word = "LOADFROM"
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.format.indent_width, 4);
        assert!(config.format.use_spaces);
        assert_eq!(config.builtin.words.len(), 1);
        assert_eq!(config.builtin.words[0].word, "LOADFROM");
    }

    #[test]
    fn test_load_from_file() {
        let toml_content = r#"
            [format]
            indent_width = 3

            [[builtin.words]]
            word = "TEST"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.format.indent_width, 3);
        assert_eq!(config.builtin.words.len(), 1);
        assert_eq!(config.builtin.words[0].word, "TEST");
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let toml_content = r#"
            [format]
            indent_width = 8
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.format.indent_width, 8);
        // Should use defaults for missing fields
        assert!(config.format.use_spaces);
        assert_eq!(config.format.word_spacing, 1);
    }

    #[test]
    fn test_parse_word_files_config() {
        let toml_content = r#"
            [builtin]
            word_files = ["gforth.words", "/absolute/path.words"]
        "#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.builtin.word_files.len(), 2);
        assert_eq!(config.builtin.word_files[0], "gforth.words");
        assert_eq!(config.builtin.word_files[1], "/absolute/path.words");
    }

    #[test]
    fn test_word_files_default_empty() {
        let config = Config::default();
        assert!(config.builtin.word_files.is_empty());
    }

    #[test]
    fn test_load_words_from_files() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let words_path = dir.path().join("test.words");
        std::fs::write(&words_path, "DUP SWAP OVER\n  ROT  DROP\n").unwrap();

        let config = BuiltinConfig {
            words: vec![],
            word_files: vec!["test.words".to_string()],
        };

        let words = config.load_words_from_files(dir.path().to_str().unwrap());
        assert_eq!(words.len(), 5);
        assert_eq!(words[0].word, "DUP");
        assert_eq!(words[1].word, "SWAP");
        assert_eq!(words[2].word, "OVER");
        assert_eq!(words[3].word, "ROT");
        assert_eq!(words[4].word, "DROP");
        // All loaded from file should have no stack/description
        assert_eq!(words[0].stack, None);
        assert_eq!(words[0].description, None);
    }

    #[test]
    fn test_load_words_from_absolute_path() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let words_path = dir.path().join("abs.words");
        std::fs::write(&words_path, "EMIT CR").unwrap();

        let config = BuiltinConfig {
            words: vec![],
            word_files: vec![words_path.to_str().unwrap().to_string()],
        };

        // workspace_root doesn't matter for absolute paths
        let words = config.load_words_from_files("/nonexistent");
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].word, "EMIT");
        assert_eq!(words[1].word, "CR");
    }

    #[test]
    fn test_load_words_missing_file_skipped() {
        let config = BuiltinConfig {
            words: vec![],
            word_files: vec!["nonexistent.words".to_string()],
        };

        let words = config.load_words_from_files("/tmp");
        assert!(words.is_empty());
    }

    #[test]
    fn test_into_static_words_combines_inline_and_files() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let words_path = dir.path().join("extra.words");
        std::fs::write(&words_path, "FILEW1 FILEW2").unwrap();

        let config = BuiltinConfig {
            words: vec![CustomWord {
                word: "INLINE1".to_string(),
                stack: Some("( -- )".to_string()),
                description: Some("An inline word".to_string()),
            }],
            word_files: vec!["extra.words".to_string()],
        };

        let static_words = config.into_static_words(Some(dir.path().to_str().unwrap()));
        assert_eq!(static_words.len(), 3);
        assert_eq!(static_words[0].token, "INLINE1");
        assert_eq!(static_words[0].stack, "( -- )");
        assert_eq!(static_words[0].help, "An inline word");
        assert_eq!(static_words[1].token, "FILEW1");
        assert_eq!(static_words[2].token, "FILEW2");
    }
}
