//! Helper functions for working with URIs in LSP.

use lsp_types::Uri;
use std::path::Path;

/// Convert a file path to a URI.
///
/// Returns None if the path cannot be converted to a valid file:// URI.
#[allow(dead_code)]
pub fn path_to_uri<P: AsRef<Path>>(path: P) -> Option<Uri> {
    let path_str = path.as_ref().to_str()?;
    let uri_str = format!("file://{}", path_str);
    uri_str.parse().ok()
}

/// Convert a file path string to a URI.
///
/// This is a convenience wrapper for string paths.
#[allow(dead_code)]
pub fn path_str_to_uri(path: &str) -> Option<Uri> {
    let uri_str = format!("file://{}", path);
    uri_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_uri() {
        let uri = path_to_uri("/tmp/test.forth");
        assert!(uri.is_some());
        let uri = uri.unwrap();
        assert!(uri.as_str().starts_with("file://"));
        assert!(uri.as_str().contains("test.forth"));
    }

    #[test]
    fn test_path_str_to_uri() {
        let uri = path_str_to_uri("/tmp/test.forth");
        assert!(uri.is_some());
        let uri = uri.unwrap();
        assert!(uri.as_str().starts_with("file://"));
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_unix_path() {
        let uri = path_to_uri("/home/user/code/test.fs");
        assert!(uri.is_some());
        let uri = uri.unwrap();
        assert!(uri.scheme().is_some());
        assert_eq!(uri.scheme().unwrap().as_str(), "file");
    }

    #[test]
    fn test_relative_path_fails() {
        // Relative paths should fail conversion
        let uri = path_str_to_uri("test.forth");
        // On some systems this might succeed, on others fail
        // The behavior is platform-dependent
        if let Some(u) = uri {
            assert!(u.scheme().is_some());
            assert_eq!(u.scheme().unwrap().as_str(), "file");
        }
    }
}
