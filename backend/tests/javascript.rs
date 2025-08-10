use backend::meta::read_all;

#[test]
fn detect_js_comment() {
    let src = "// @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0}\nconsole.log(\"hi\");";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "1");
}
