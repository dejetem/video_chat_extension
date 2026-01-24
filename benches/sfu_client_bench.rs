use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_sfu_client(c: &mut Criterion) {
    c.bench_function("sfu_client_connection_prep", |b| {
        b.iter(|| {
            // Mock preparation for a connection
            black_box("client_ready")
        })
    });
}

criterion_group!(benches, benchmark_sfu_client);
criterion_main!(benches);
