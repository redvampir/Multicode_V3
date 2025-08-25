use super::meta_integration::changed_meta_ids;

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
