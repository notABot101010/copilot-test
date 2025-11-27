//! Benchmarks comparing our base64 implementation with the external base64 crate.

use base64::{decode_with, decode_with_avx2, encode_with, encode_with_avx2, ALPHABET_STANDARD};
use base64_external::{engine::general_purpose::STANDARD, Engine};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

/// Sample data sizes for benchmarking
const SIZES: &[usize] = &[16, 64, 256, 1024, 4096, 16384];

fn generate_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for &size in SIZES {
        let data = generate_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("our_impl", size), &data, |b, data| {
            b.iter(|| encode_with(black_box(data), ALPHABET_STANDARD, true))
        });

        group.bench_with_input(BenchmarkId::new("our_impl_avx2", size), &data, |b, data| {
            b.iter(|| encode_with_avx2(black_box(data), ALPHABET_STANDARD, true))
        });

        group.bench_with_input(BenchmarkId::new("base64_crate", size), &data, |b, data| {
            b.iter(|| STANDARD.encode(black_box(data)))
        });
    }

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for &size in SIZES {
        let data = generate_data(size);
        let encoded_ours = encode_with(&data, ALPHABET_STANDARD, true);
        let encoded_external = STANDARD.encode(&data);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("our_impl", size),
            &encoded_ours,
            |b, encoded| b.iter(|| decode_with(black_box(encoded), ALPHABET_STANDARD)),
        );

        group.bench_with_input(
            BenchmarkId::new("our_impl_avx2", size),
            &encoded_ours,
            |b, encoded| b.iter(|| decode_with_avx2(black_box(encoded), ALPHABET_STANDARD)),
        );

        group.bench_with_input(
            BenchmarkId::new("base64_crate", size),
            &encoded_external,
            |b, encoded| b.iter(|| STANDARD.decode(black_box(encoded))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
