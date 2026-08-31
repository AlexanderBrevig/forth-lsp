use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::words::Word;

/// Configuration for the Forth LSP server
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub format: FormatConfig,

    #[serde(default)]
    pub builtin: BuiltinConfig,

    #[serde(default)]
    pub workspace: WorkspaceConfig,
}

/// Workspace scanning configuration.
///
/// Controls which files are discovered and indexed when the server starts up.
/// The defaults reproduce the historical behaviour (the same Forth extensions
/// were always recognised) while additionally skipping common noise
/// directories such as `target` and `node_modules`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    /// File extensions (without the leading dot) treated as Forth source and
    /// indexed. Override this to drop extensions that clash with other
    /// languages, e.g. removing `"fs"` when it means F# or GLSL fragment
    /// shaders in your project.
    #[serde(default = "default_forth_extensions")]
    pub extensions: Vec<String>,

    /// Directory or file names to skip while scanning the workspace. Each
    /// pattern is matched against a single path component (a folder or file
    /// name, not a full path) and supports `*` and `?` wildcards, e.g.
    /// `"target"`, `"node_modules"`, or `"*.gen.fs"`.
    #[serde(default = "default_workspace_exclude")]
    pub exclude: Vec<String>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            extensions: default_forth_extensions(),
            exclude: default_workspace_exclude(),
        }
    }
}

impl WorkspaceConfig {
    /// Returns true if `path`'s extension is configured as Forth source.
    /// Comparison is case-insensitive, matching the historical behaviour.
    pub fn is_forth_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(OsStr::to_str)
            .map(|ext| {
                self.extensions
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(ext))
            })
            .unwrap_or(false)
    }

    /// Returns true if the final component (folder or file name) of `path`
    /// matches one of the exclude patterns. Called per directory entry as the
    /// scanner descends, so excluding a folder prunes its entire subtree.
    pub fn is_excluded(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(OsStr::to_str)
            .map(|name| self.exclude.iter().any(|pattern| glob_match(pattern, name)))
            .unwrap_or(false)
    }
}

/// Minimal glob matcher supporting `*` (any run of characters, including empty)
/// and `?` (exactly one character). Matches the whole `text` against `pattern`
/// and is applied to a single path component. Kept dependency-free on purpose.
fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    // Backtrack point for the most recent `*` and the text position it began at.
    let (mut star, mut resume) = (None, 0usize);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            resume = ti;
            pi += 1;
        } else if let Some(s) = star {
            // Last `*` absorbs one more character of `text`, then retry.
            pi = s + 1;
            resume += 1;
            ti = resume;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Formatter configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormatConfig {
    /// Control if the formatter should run or not
    #[serde(default = "default_true")]
    pub enabled: bool,

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
            enabled: default_true(),
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
    pub fn to_static_words(&self, workspace_root: Option<&str>) -> Vec<&'static Word<'static>> {
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

/// The Forth extensions recognised historically, before scanning was
/// configurable. Kept as the default so existing setups are unaffected.
fn default_forth_extensions() -> Vec<String> {
    ["f", "fth", "fs", "4th", "forth", "frt"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Directories that are almost never Forth sources and only slow scanning
/// down. Skipping them by default is strictly better than the old behaviour,
/// which descended into everything.
fn default_workspace_exclude() -> Vec<String> {
    [".git", "target", "node_modules"]
        .iter()
        .map(|s| s.to_string())
        .collect()
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
    fn test_workspace_defaults_match_legacy_extensions() {
        // No config must behave exactly like the old hardcoded list.
        let ws = WorkspaceConfig::default();
        for ext in ["f", "fth", "fs", "4th", "forth", "frt"] {
            assert!(
                ws.is_forth_file(Path::new(&format!("x.{ext}"))),
                "{ext} should be recognised as forth"
            );
        }
        // Case-insensitive, like the old behaviour.
        assert!(ws.is_forth_file(Path::new("PROG.FTH")));
        // Non-forth extensions are ignored.
        assert!(!ws.is_forth_file(Path::new("main.rs")));
        assert!(!ws.is_forth_file(Path::new("noext")));
    }

    #[test]
    fn test_workspace_default_exclude_skips_noise_dirs() {
        let ws = WorkspaceConfig::default();
        assert!(ws.is_excluded(Path::new("/proj/target")));
        assert!(ws.is_excluded(Path::new("/proj/node_modules")));
        assert!(ws.is_excluded(Path::new("/proj/.git")));
        assert!(!ws.is_excluded(Path::new("/proj/src")));
        // Only the final component is matched, so a project living under a
        // path that happens to contain "target" is not wrongly excluded.
        assert!(!ws.is_excluded(Path::new("/home/me/target-practice/src")));
    }

    #[test]
    fn test_workspace_extensions_override_drops_fs() {
        // The .fs clash fix: user redefines the extension set without "fs".
        let toml_content = r#"
            [workspace]
            extensions = ["f", "fth", "4th", "forth", "frt"]
        "#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(!config.workspace.is_forth_file(Path::new("shader.fs")));
        assert!(config.workspace.is_forth_file(Path::new("prog.fth")));
        // exclude falls back to its default when only extensions is set.
        assert!(config.workspace.is_excluded(Path::new("/p/target")));
    }

    #[test]
    fn test_workspace_exclude_override_and_globs() {
        let toml_content = r#"
            [workspace]
            exclude = ["shaders", "*.gen.fs", "vendor?"]
        "#;
        let config: Config = toml::from_str(toml_content).unwrap();
        let ws = &config.workspace;
        assert!(ws.is_excluded(Path::new("/p/shaders")));
        assert!(ws.is_excluded(Path::new("/p/foo.gen.fs")));
        assert!(ws.is_excluded(Path::new("/p/vendor1")));
        assert!(!ws.is_excluded(Path::new("/p/vendor"))); // ? needs exactly one char
        assert!(!ws.is_excluded(Path::new("/p/target"))); // overridden away
        // extensions fall back to default when only exclude is set.
        assert!(ws.is_forth_file(Path::new("x.fs")));
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("target", "target"));
        assert!(!glob_match("target", "targets"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*.fs", "shader.fs"));
        assert!(glob_match("*.gen.fs", "a.b.gen.fs"));
        assert!(!glob_match("*.fs", "shader.fsx"));
        assert!(glob_match("v?", "v1"));
        assert!(!glob_match("v?", "v"));
        assert!(!glob_match("v?", "v12"));
        assert!(glob_match("a*c", "abbbc"));
        assert!(glob_match("a*c", "ac"));
        assert!(!glob_match("a*c", "ab"));
    }

    #[test]
    fn test_to_static_words_combines_inline_and_files() {
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

        let static_words = config.to_static_words(Some(dir.path().to_str().unwrap()));
        assert_eq!(static_words.len(), 3);
        assert_eq!(static_words[0].token, "INLINE1");
        assert_eq!(static_words[0].stack, "( -- )");
        assert_eq!(static_words[0].help, "An inline word");
        assert_eq!(static_words[1].token, "FILEW1");
        assert_eq!(static_words[2].token, "FILEW2");
    }
}
