use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_wasm_interop(c: &mut Criterion) {
    c.bench_function("wasm_js_bridge_latency", |b| {
        b.iter(|| {
            // Mock interop latency
            black_box("rust_to_js")
        })
    });
}

criterion_group!(benches, benchmark_wasm_interop);
criterion_main!(benches);
