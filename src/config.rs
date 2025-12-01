use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
}
