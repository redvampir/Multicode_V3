use backend::viz_lint::lint_str;
use std::process::Command;
use tempfile::NamedTempFile;

/// Path to compiled backend binary provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_backend");

#[test]
fn valid_graph_passes() {
    let src = "// @viz op=inc node=1 id=a out=b\n// @viz op=dec node=2 id=b in=a";
    let errors = lint_str(src);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn detects_unknown_op() {
    let src = "// @viz op=foo node=1 id=a";
    let errors = lint_str(src);
    assert!(errors.iter().any(|e| e.contains("unknown op")), "{errors:?}");
}

#[test]
fn cli_reports_errors() {
    let file = NamedTempFile::new().expect("temp file");
    std::fs::write(file.path(), "// @viz op=foo node=1 id=a").expect("write temp file");
    let output = Command::new(BIN)
        .args(["viz", "lint", file.path().to_str().unwrap()])
        .output()
        .expect("run lint");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown op"), "stderr: {stderr}");
}
