use backend::meta::read_all;

#[test]
fn detect_css_comment() {
    let src = "/* @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0} */\n.selector { color: red; }";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "1");
}
