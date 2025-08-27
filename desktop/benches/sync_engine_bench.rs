use criterion::{black_box, criterion_group, criterion_main, Criterion};
use desktop::sync::{SyncEngine, SyncMessage, SyncSettings};
use multicode_core::parser::Lang;
use memory_stats::memory_stats;

fn generate_code_with_metas(n: usize) -> String {
    let mut code = String::with_capacity(n * 80);
    for i in 0..n {
        code.push_str(&format!(
            "// @VISUAL_META {{\"id\":\"{i}\",\"x\":0.0,\"y\":0.0}}\nfn f{i}() {{}}\n"
        ));
    }
    code
}

fn bench_text_changed(c: &mut Criterion) {
    let code = generate_code_with_metas(10_000);
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    c.bench_function("SyncMessage::TextChanged", |b| {
        b.iter(|| {
            let code_clone = code.clone();
            let before = memory_stats().unwrap().physical_mem;
            let _ = engine.handle(SyncMessage::TextChanged(code_clone, Lang::Rust));
            let after = memory_stats().unwrap().physical_mem;
            black_box(after - before);
        })
    });
}

criterion_group!(benches, bench_text_changed);
criterion_main!(benches);
