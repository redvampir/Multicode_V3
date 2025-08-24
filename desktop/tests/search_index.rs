use desktop::search::index::SearchIndex;

#[test]
fn search_finds_existing_item() {
    let mut idx = SearchIndex::new();
    idx.insert("cmd900", 900);
    assert_eq!(idx.search("cmd900"), vec![900]);
}

#[test]
fn search_returns_empty_for_missing_item() {
    let mut idx = SearchIndex::new();
    idx.insert("cmd900", 900);
    assert!(idx.search("unknown").is_empty());
}
