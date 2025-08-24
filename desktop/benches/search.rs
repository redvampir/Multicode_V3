use criterion::{criterion_group, criterion_main, Criterion};
use desktop::search::index::SearchIndex;

fn naive(items: &[(usize, String)], q: &str) -> Vec<usize> {
    items
        .iter()
        .filter_map(|(id, kw)| if kw == q { Some(*id) } else { None })
        .collect()
}

fn bench_search(c: &mut Criterion) {
    let mut idx = SearchIndex::new();
    let mut items = Vec::new();
    for i in 0..1000 {
        let kw = format!("cmd{}", i);
        idx.insert(&kw, i);
        items.push((i, kw));
    }
    let query = "cmd900";

    c.bench_function("naive search", |b| {
        b.iter(|| {
            let r = naive(&items, query);
            assert_eq!(r, vec![900]);
        })
    });

    c.bench_function("indexed search", |b| {
        b.iter(|| {
            let r = idx.search(query);
            assert_eq!(r, vec![900]);
        })
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
