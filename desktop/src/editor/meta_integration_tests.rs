use super::meta_integration::{changed_meta_ids, find_meta_comments};

#[test]
fn detects_new_meta_id() {
    let old = "";
    let new = "# @VISUAL_META {\"id\":\"new\",\"x\":0.0,\"y\":0.0}";
    assert_eq!(changed_meta_ids(old, new), vec!["new".to_string()]);
}

#[test]
fn detects_removed_meta_id() {
    let old = "# @VISUAL_META {\"id\":\"old\",\"x\":0.0,\"y\":0.0}";
    let new = "";
    assert_eq!(changed_meta_ids(old, new), vec!["old".to_string()]);
}

#[test]
fn detects_modified_meta_id() {
    let old = "# @VISUAL_META {\"id\":\"same\",\"x\":0.0,\"y\":0.0}";
    let new = "# @VISUAL_META {\"id\":\"same\",\"x\":1.0,\"y\":1.0}";
    assert_eq!(changed_meta_ids(old, new), vec!["same".to_string()]);
}

#[test]
fn does_not_duplicate_ids() {
    let old = "";
    let new = "# @VISUAL_META {\"id\":\"dup\",\"x\":0.0,\"y\":0.0}\n# @VISUAL_META {\"id\":\"dup\",\"x\":1.0,\"y\":1.0}";
    assert_eq!(changed_meta_ids(old, new), vec!["dup".to_string()]);
}

#[test]
fn finds_multiple_comments_per_line() {
    let content = "# @VISUAL_META {\"id\":\"one\"} @VISUAL_META {\"id\":\"two\"}";
    let comments = find_meta_comments(content);
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].2, "{\"id\":\"one\"}");
    assert_eq!(comments[1].2, "{\"id\":\"two\"}");
}

#[test]
fn ignores_text_after_closing_brace() {
    let content = "# @VISUAL_META {\"id\":\"one\"} trailing";
    let comments = find_meta_comments(content);
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].2, "{\"id\":\"one\"}");
}
