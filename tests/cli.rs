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
fn test_extraneous_client_args_are_accepted() {
    // LSP clients commonly pass arguments like --stdio; the server must not
    // reject them at startup.
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .arg("--stdio")
        .output()
        .expect("failed to execute forth-lsp");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "extraneous args should be accepted, got: {stderr}"
    );
    assert!(
        stderr.contains("starting generic LSP server"),
        "server should start when given extraneous args, got: {stderr}"
    );
}

#[test]
fn test_version_with_extraneous_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_forth-lsp"))
        .args(["--version", "--stdio"])
        .output()
        .expect("failed to execute forth-lsp");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, format!("{}\n", env!("CARGO_PKG_VERSION")));
}
