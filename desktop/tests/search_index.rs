use desktop::search::index::SearchIndex;
use std::time::Instant;

fn naive(items: &[(usize, String)], q: &str) -> Vec<usize> {
    items
        .iter()
        .filter_map(|(id, kw)| if kw == q { Some(*id) } else { None })
        .collect()
}

#[test]
fn compare_search_speed() {
    let mut idx = SearchIndex::new();
    let mut items = Vec::new();
    for i in 0..1000 {
        let kw = format!("cmd{}", i);
        idx.insert(&kw, i);
        items.push((i, kw));
    }
    let query = "cmd900";
    let start = Instant::now();
    for _ in 0..1000 {
        let r = naive(&items, query);
        assert_eq!(r, vec![900]);
    }
    let naive_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..1000 {
        let r = idx.search(query);
        assert_eq!(r, vec![900]);
    }
    let indexed_time = start.elapsed();
    println!("naive: {:?}, indexed: {:?}", naive_time, indexed_time);
}
