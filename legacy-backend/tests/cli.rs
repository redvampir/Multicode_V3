use std::process::Command;

use tempfile::NamedTempFile;

/// Path to compiled backend binary provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_backend");

#[test]
fn parse_reports_missing_file() {
    let output = Command::new(BIN)
        .args(["parse", "no_such_file.rs", "--lang", "rust"])
        .output()
        .expect("run backend");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn parse_reports_unknown_language() {
    let file = NamedTempFile::new().expect("temp file");
    std::fs::write(file.path(), "fn main() {}").expect("write temp file");

    let output = Command::new(BIN)
        .args([
            "parse",
            file.path().to_str().unwrap(),
            "--lang",
            "madeuplang",
        ])
        .output()
        .expect("run backend");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown language"),
        "unexpected stderr: {stderr}"
    );
}

