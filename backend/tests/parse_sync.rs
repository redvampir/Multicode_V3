use backend::meta::read_all;

#[test]
fn block_disappears_after_comment_removal() {
    let with_comment = "// @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0,\"updated_at\":\"2024-01-01T00:00:00Z\"}\nfn main() {}";
    assert_eq!(read_all(with_comment).len(), 1);

    let without_comment = "fn main() {}";
    assert!(read_all(without_comment).is_empty());
}
