use core::viz_lint::lint_str;

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
    assert!(errors.iter().any(|e| e.contains("неизвестная операция")), "{errors:?}");
}
