use crate::parser::viz_comments::{load_viz_document, parse_viz_comments, VizDocument};
use std::collections::HashSet;
use std::path::Path;

/// List of allowed operation names. This is a minimal set used by tests and
/// can be extended in the future as new visual blocks are added.
const ALLOWED_OPS: &[&str] = &["inc", "dec", "Add", "ternary"];

/// Lint raw source content containing `@viz` comments.
///
/// Returns a list of human readable error messages. An empty list
/// indicates that no issues were found.
pub fn lint_str(content: &str) -> Vec<String> {
    let doc = parse_viz_comments(content);
    lint_document(&doc)
}

/// Lint a file on disk using either a sibling `*.viz.json` document or
/// inline `@viz` comments.
///
/// The function returns a list of detected problems or an I/O error if
/// the file could not be read.
pub fn lint_file(path: &Path) -> std::io::Result<Vec<String>> {
    let doc = load_viz_document(path)?;
    Ok(lint_document(&doc))
}

/// Perform linting on a [`VizDocument`].
fn lint_document(doc: &VizDocument) -> Vec<String> {
    let mut errors = Vec::new();
    let mut node_ids = HashSet::new();
    let known_ids: HashSet<&str> = doc
        .nodes
        .iter()
        .filter_map(|n| n.id.as_deref())
        .collect();

    for entry in &doc.nodes {
        let ident = entry.id.as_deref().unwrap_or("<unknown>");

        match entry.op.as_deref() {
            Some(op) if ALLOWED_OPS.contains(&op) => {}
            Some(op) => errors.push(format!("node {ident}: unknown op `{op}`")),
            None => errors.push(format!("node {ident}: missing op")),
        }

        match entry.node.as_deref() {
            Some(n) => {
                if n.parse::<u32>().is_err() {
                    errors.push(format!("node {ident}: invalid node `{n}`"));
                }
                if !node_ids.insert(n) {
                    errors.push(format!("duplicate node identifier `{n}`"));
                }
            }
            None => errors.push(format!("node {ident}: missing node")),
        }

        for inp in &entry.inputs {
            if !known_ids.contains(inp.as_str()) {
                errors.push(format!("node {ident}: unknown input `{inp}`"));
            }
        }
        for out in &entry.outputs {
            if !known_ids.contains(out.as_str()) {
                errors.push(format!("node {ident}: unknown output `{out}`"));
            }
        }
    }

    errors
}
