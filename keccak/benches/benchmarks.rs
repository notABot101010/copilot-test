//! Benchmarks comparing our AVX2 p1600 implementation with the keccak crate

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_p1600_24_rounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("p1600_24_rounds");

    // Our implementation
    group.bench_function("keccak_avx2", |b| {
        let mut state = [0u64; 25];
        for i in 0..25 {
            state[i] = i as u64 * 0x0123456789ABCDEF;
        }
        b.iter(|| {
            keccak_avx2::p1600::<24>(black_box(&mut state));
        });
    });

    // Reference keccak crate
    group.bench_function("keccak_crate", |b| {
        let mut state = [0u64; 25];
        for i in 0..25 {
            state[i] = i as u64 * 0x0123456789ABCDEF;
        }
        b.iter(|| {
            keccak::p1600(black_box(&mut state), 24);
        });
    });

    group.finish();
}

fn bench_p1600_12_rounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("p1600_12_rounds");

    // Our implementation
    group.bench_function("keccak_avx2", |b| {
        let mut state = [0u64; 25];
        for i in 0..25 {
            state[i] = i as u64 * 0x0123456789ABCDEF;
        }
        b.iter(|| {
            keccak_avx2::p1600::<12>(black_box(&mut state));
        });
    });

    // Reference keccak crate
    group.bench_function("keccak_crate", |b| {
        let mut state = [0u64; 25];
        for i in 0..25 {
            state[i] = i as u64 * 0x0123456789ABCDEF;
        }
        b.iter(|| {
            keccak::p1600(black_box(&mut state), 12);
        });
    });

    group.finish();
}

fn bench_p1600_multiple_invocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("p1600_100_invocations");

    // Our implementation - 100 invocations
    group.bench_function("keccak_avx2", |b| {
        let mut state = [0u64; 25];
        b.iter(|| {
            for _ in 0..100 {
                keccak_avx2::p1600::<24>(black_box(&mut state));
            }
        });
    });

    // Reference keccak crate - 100 invocations
    group.bench_function("keccak_crate", |b| {
        let mut state = [0u64; 25];
        b.iter(|| {
            for _ in 0..100 {
                keccak::p1600(black_box(&mut state), 24);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_p1600_24_rounds,
    bench_p1600_12_rounds,
    bench_p1600_multiple_invocations,
);

criterion_main!(benches);
