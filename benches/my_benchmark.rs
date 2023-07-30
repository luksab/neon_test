use criterion::{black_box, criterion_group, criterion_main, Criterion};

use neon_test::*;

fn criterion_benchmark(c: &mut Criterion) {
    let x = 384;
    let y = 10000;
    let array = generate_array(x, y);
    c.bench_function("transpose sisd", |b| {
        b.iter(|| transpose_array_sisd(black_box(&array)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
