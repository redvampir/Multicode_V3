use backend::export::{serialize_viz_document, deserialize_viz_document, VizDocument};

#[test]
fn roundtrip_viz_document() {
    let src = "// @VISUAL_META {\"id\":\"1\",\"x\":0.0,\"y\":0.0}\nfn main() {}";
    let json = serialize_viz_document(src).expect("should serialize");
    let doc = deserialize_viz_document(&json).expect("valid json");
    assert_eq!(doc.nodes.len(), 1);
    assert_eq!(doc.nodes[0].id, "1");
}
