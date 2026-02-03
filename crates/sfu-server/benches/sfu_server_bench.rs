use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn benchmark_simulcast_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulcast_selection");

    // Available layers: 0 (low), 1 (mid), 2 (high)
    let layers = vec![0, 1, 2];

    group.bench_function("select_high_bandwidth", |b| {
        b.iter(|| {
            // simulating 2Mbps bandwidth
            video_chat_sfu_server::simulcast::SimulcastManager::select_layer(
                black_box(&layers),
                black_box(2_000_000),
            )
        })
    });

    group.bench_function("select_low_bandwidth", |b| {
        b.iter(|| {
            // simulating 100kbps bandwidth
            video_chat_sfu_server::simulcast::SimulcastManager::select_layer(
                black_box(&layers),
                black_box(100_000),
            )
        })
    });

    group.finish();
}

fn benchmark_packet_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtp_packet_processing");
    group.throughput(Throughput::Bytes(1500));

    // Simulate a standard RTP packet size
    let packet_size = 1500;
    let packet_data = vec![0u8; packet_size];

    group.bench_function("process_rtp_packet", |b| {
        b.iter(|| {
            // Basic packet validation/inspection simulation
            let data = black_box(&packet_data);
            if data.len() >= 12 && data[0] & 0x80 != 0 {
                // simulated header check
                black_box(true)
            } else {
                black_box(false)
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simulcast_selection,
    benchmark_packet_processing
);
criterion_main!(benches);
