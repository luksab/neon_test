use criterion::{black_box, criterion_group, criterion_main, Criterion};

use neon_test::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("double_array");
    let x = 10_000_000;
    let array = generate_array(x);
    group.bench_function("sisd", |b| {
        b.iter(|| double_array_sisd(black_box(&array)))
    });
    group.bench_function("sisd opt", |b| {
        b.iter(|| double_array_sisd_opt(black_box(&array)))
    });
    group.bench_function("sisd 64 opt", |b| {
        b.iter(|| double_array_sisd_opt_64(black_box(&array)))
    });
    group.bench_function("sisd opt iter", |b| {
        b.iter(|| double_array_sisd_opt_iter(black_box(&array)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
