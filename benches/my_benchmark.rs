use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};

use neon_test::*;

fn compare_algos(c: &mut Criterion) {
    let mut group = c.benchmark_group("double_array");
    let x = 10 * 1024 * 1024;
    let array = generate_array(x);
    // times three, because the input is x bytes, output is 2x bytes, so total processed is 3x bytes
    group.throughput(Throughput::Bytes((x * 3) as u64));
    // group.bench_function("sisd", |b| b.iter(|| double_array_sisd(black_box(&array))));
    // group.bench_function("sisd opt", |b| {
    //     b.iter(|| double_array_sisd_opt(black_box(&array)))
    // });
    // group.bench_function("sisd 64 opt", |b| {
    //     b.iter(|| double_array_sisd_opt_64(black_box(&array)))
    // });
    group.bench_function("sisd opt iter", |b| {
        b.iter(|| double_array_sisd_opt_iter(black_box(&array)))
    });
    group.bench_function("lut u4", |b| {
        b.iter(|| double_array_lookup_u4(black_box(&array)))
    });
    group.bench_function("lut u8", |b| {
        b.iter(|| double_array_lookup_u8(black_box(&array)))
    });
    group.bench_function("lut u16", |b| {
        b.iter(|| double_array_lookup_u16(black_box(&array)))
    });
    group.bench_function("laura", |b| {
        b.iter(|| double_array_sisd_laura(black_box(&array)))
    });
    group.bench_function("laura u32", |b| {
        b.iter(|| double_array_sisd_laura_u32(black_box(&array)))
    });
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx512f"
    ))]
    group.bench_function("lut u4 simd avx", |b| {
        {
            b.iter(|| double_array_lookup_avx_u4(black_box(&array)))
        }
    });
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx512f"
    ))]
    group.bench_function("simd u4 simd avx512", |b| {
        {
            b.iter(|| double_array_lookup_avx512_u4(black_box(&array)))
        }
    });
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx512f"
    ))]
    group.bench_function("simd laura avx", |b| {
        {
            b.iter(|| double_array_simd_laura(black_box(&array)))
        }
    });
    group.bench_function("throughput test", |b| {
        {
            b.iter(|| throughput_test(black_box(&array)))
        }
    });

    // // group.bench_function("sisd opt iter rayon", |b| {
    // //     b.iter(|| double_array_sisd_opt_rayon(black_box(&array)))
    // // });
    // group.bench_function("lut simd", |b| {
    //     b.iter(|| double_array_lookup_neon_u4(black_box(&array)))
    // });
    // group.bench_function("lut simd multi", |b| {
    //     let thread_pool = rayon::ThreadPoolBuilder::new()
    //         .num_threads(8)
    //         .build()
    //         .unwrap();
    //     b.iter(|| double_array_lookup_neon_u4_multithread(black_box(&array), &thread_pool))
    // });
    group.bench_function("ben", |b| b.iter(|| double_array_ben(black_box(&array))));
    group.bench_function("benk", |b| b.iter(|| double_array_benk(black_box(&array))));
    group.finish();
}

// fn compare_size_rayon(c: &mut Criterion) {
//     static KB: usize = 1024;
//     let mut group = c.benchmark_group("rayon");
//     for size in [KB, 16 * KB, 128 * KB, KB * KB, 16 * KB * KB, 128 * KB * KB].iter() {
//         let array = generate_array(*size);
//         group.throughput(Throughput::Bytes(*size as u64));
//         group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
//             b.iter(|| double_array_sisd_opt_rayon(black_box(&array)))
//         });
//     }
//     let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
//     group.plot_config(plot_config);
//     group.finish();
// }

// fn compare_size_opt(c: &mut Criterion) {
//     static KB: usize = 1024;
//     let mut group = c.benchmark_group("opt");
//     for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
//         let array = generate_array(*size);
//         group.throughput(Throughput::Bytes(*size as u64));
//         group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
//             b.iter(|| double_array_sisd_opt_iter(black_box(&array)))
//         });
//     }
//     group.finish();
// }

// criterion_group!(benches, compare_size_rayon, compare_size_opt);
criterion_group!(benches, compare_algos);
criterion_main!(benches);
