use backend::export::remove_meta_lines;

#[test]
fn remove_python_meta() {
    let src = "# @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0}\nprint(\"hi\")";
    let cleaned = remove_meta_lines(src);
    assert!(!cleaned.contains("@VISUAL_META"));
    assert!(cleaned.contains("print"));
}

#[test]
fn remove_js_meta() {
    let src = "// @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0}\nconsole.log(\"hi\");";
    let cleaned = remove_meta_lines(src);
    assert!(!cleaned.contains("@VISUAL_META"));
    assert!(cleaned.contains("console.log"));
}

#[test]
fn remove_css_meta() {
    let src = "/* @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0} */\n.selector { color: red; }";
    let cleaned = remove_meta_lines(src);
    assert!(!cleaned.contains("@VISUAL_META"));
    assert!(cleaned.contains(".selector"));
}

#[test]
fn remove_html_meta() {
    let src = "<!-- @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0} -->\n<div></div>";
    let cleaned = remove_meta_lines(src);
    assert!(!cleaned.contains("@VISUAL_META"));
    assert!(cleaned.contains("<div>"));
}
