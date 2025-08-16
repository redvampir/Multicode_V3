use backend::parser::viz_comments::{load_viz_document, parse_viz_comments, save_viz_document};
use std::fs;
use tempfile::tempdir;

#[test]
fn parse_comment_into_document() {
    let src = "// @viz op=Add node=1 id=n1 in=a,b out=c";
    let doc = parse_viz_comments(src);
    assert_eq!(doc.nodes.len(), 1);
    let n = &doc.nodes[0];
    assert_eq!(n.op.as_deref(), Some("Add"));
    assert_eq!(n.node.as_deref(), Some("1"));
    assert_eq!(n.id.as_deref(), Some("n1"));
    assert_eq!(n.inputs, vec!["a", "b"]);
    assert_eq!(n.outputs, vec!["c"]);
}

#[test]
fn roundtrip_import_export() -> std::io::Result<()> {
    let tmp = tempdir()?;
    let path = tmp.path().join("file.rs");
    fs::write(
        &path,
        "// @viz op=Add node=1 id=n1 in=a,b out=c\nfn main() {}\n",
    )?;
    // Import from comment (since .viz.json doesn't exist)
    let doc1 = load_viz_document(&path)?;
    assert_eq!(doc1.nodes.len(), 1);
    // Export to .viz.json
    save_viz_document(&path, &doc1)?;
    let viz_path = path.with_extension("viz.json");
    assert!(viz_path.exists());
    // Import again, now from .viz.json
    let doc2 = load_viz_document(&path)?;
    assert_eq!(doc1, doc2);
    Ok(())
}

#[test]
fn parse_inc_dec_ops() {
    let src = "// @viz op=inc node=1 id=i in=a out=b\n// @viz op=dec node=2 id=d in=b out=c";
    let doc = parse_viz_comments(src);
    assert_eq!(doc.nodes.len(), 2);
    assert_eq!(doc.nodes[0].op.as_deref(), Some("inc"));
    assert_eq!(doc.nodes[1].op.as_deref(), Some("dec"));
}
