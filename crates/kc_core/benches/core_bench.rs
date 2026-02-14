use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kc_core::hashing::blake3_hex_prefixed;

fn hash_bench(c: &mut Criterion) {
    c.bench_function("core_hash_1kb", |b| {
        let input = vec![42u8; 1024];
        b.iter(|| blake3_hex_prefixed(black_box(&input)));
    });
}

criterion_group!(benches, hash_bench);
criterion_main!(benches);
