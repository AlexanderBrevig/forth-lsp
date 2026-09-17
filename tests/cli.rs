use std::process::Command;

#[test]
fn test_version_long() {
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .arg("--version")
        .output()
        .expect("failed to execute forth-lsp");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout, format!("{}\n", env!("CARGO_PKG_VERSION")));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr should be empty when --version is given, got: {stderr}"
    );
}

#[test]
fn test_version_short() {
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .arg("-V")
        .output()
        .expect("failed to execute forth-lsp");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout, format!("{}\n", env!("CARGO_PKG_VERSION")));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr should be empty when -V is given, got: {stderr}"
    );
}

#[test]
fn test_version_without_cargo_toml() {
    let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .current_dir(temp_dir.path())
        .arg("--version")
        .output()
        .expect("failed to execute forth-lsp");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout, format!("{}\n", env!("CARGO_PKG_VERSION")));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr should be empty when running in isolated dir, got: {stderr}"
    );
}

#[test]
fn test_help_shows_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .arg("--help")
        .output()
        .expect("failed to execute forth-lsp");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--version"));
    assert!(stdout.contains("-V"));
}

#[test]
fn test_invalid_flag_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .arg("--nonexistent-flag")
        .output()
        .expect("failed to execute forth-lsp");

    assert!(!output.status.success());
}
