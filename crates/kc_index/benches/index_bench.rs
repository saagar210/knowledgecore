use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn trivial_index_bench(c: &mut Criterion) {
    c.bench_function("index_sort_1k", |b| {
        b.iter(|| {
            let mut data: Vec<u32> = (0..1000).rev().collect();
            data.sort();
            black_box(data);
        })
    });
}

criterion_group!(benches, trivial_index_bench);
criterion_main!(benches);
