use backend::meta::read_all;

#[test]
fn detect_css_comment() {
    let src = "/* @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0,\"updated_at\":\"2024-01-01T00:00:00Z\"} */\n.selector { color: red; }";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "1");
}
