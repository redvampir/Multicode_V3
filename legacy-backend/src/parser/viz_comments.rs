use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Representation of a single `@viz` comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VizEntry {
    /// Operation type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Associated node identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// Optional unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Incoming connections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,
    /// Outgoing connections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<String>,
}

/// Collection of parsed viz entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VizDocument {
    #[serde(default)]
    pub nodes: Vec<VizEntry>,
}

static VIZ_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@viz\s+(?P<params>.*)").unwrap());

/// Parse all `@viz` comments contained in `content`.
///
/// Each comment must follow the pattern `// @viz op=... node=... id=... in=a,b out=c,d`.
/// Parameters are optional and may appear in any order. The `in` and `out`
/// parameters accept comma separated values.
pub fn parse_viz_comments(content: &str) -> VizDocument {
    let mut doc = VizDocument::default();
    for line in content.lines() {
        if let Some(caps) = VIZ_RE.captures(line) {
            let params = caps.name("params").map(|m| m.as_str()).unwrap_or("");
            doc.nodes.push(parse_entry(params));
        }
    }
    doc
}

fn parse_entry(params: &str) -> VizEntry {
    let mut entry = VizEntry::default();
    for part in params.split_whitespace() {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let value = kv.next().unwrap_or("");
        match key {
            "op" => entry.op = Some(value.to_string()),
            "node" => entry.node = Some(value.to_string()),
            "id" => entry.id = Some(value.to_string()),
            "in" => entry.inputs = parse_list(value),
            "out" => entry.outputs = parse_list(value),
            _ => {}
        }
    }
    entry
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Load a [`VizDocument`] associated with the provided source file.
///
/// The function first looks for a sibling `*.viz.json` file. If present, it is
/// deserialized and returned. Otherwise the source file itself is scanned for
/// `@viz` comments and a document is constructed from them.
pub fn load_viz_document(path: &Path) -> std::io::Result<VizDocument> {
    let viz_path = path.with_extension("viz.json");
    if viz_path.exists() {
        let json = fs::read_to_string(viz_path)?;
        serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    } else {
        let content = fs::read_to_string(path)?;
        Ok(parse_viz_comments(&content))
    }
}

/// Save a [`VizDocument`] as a sibling `*.viz.json` file next to `source`.
pub fn save_viz_document(path: &Path, doc: &VizDocument) -> std::io::Result<()> {
    let viz_path = path.with_extension("viz.json");
    let json = serde_json::to_string(doc)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(viz_path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_comment() {
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
}
