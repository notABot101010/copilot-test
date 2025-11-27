//! Benchmarks comparing our hex implementation with the external hex crate.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hex::{decode, decode_avx2, encode, encode_avx2, ALPHABET_LOWER};
use std::hint::black_box;

/// Sample data sizes for benchmarking
const SIZES: &[usize] = &[16, 64, 256, 1024, 4096, 16384];

fn generate_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex_encode");

    for &size in SIZES {
        let data = generate_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("our_impl", size), &data, |b, data| {
            b.iter(|| encode(black_box(data), ALPHABET_LOWER))
        });

        group.bench_with_input(BenchmarkId::new("our_impl_avx2", size), &data, |b, data| {
            b.iter(|| encode_avx2(black_box(data), ALPHABET_LOWER))
        });

        group.bench_with_input(BenchmarkId::new("hex_crate", size), &data, |b, data| {
            b.iter(|| hex_external::encode(black_box(data)))
        });
    }

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex_decode");

    for &size in SIZES {
        let data = generate_data(size);
        let encoded_ours = encode(&data, ALPHABET_LOWER);
        let encoded_external = hex_external::encode(&data);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("our_impl", size),
            &encoded_ours,
            |b, encoded| b.iter(|| decode(black_box(encoded), ALPHABET_LOWER)),
        );

        group.bench_with_input(
            BenchmarkId::new("our_impl_avx2", size),
            &encoded_ours,
            |b, encoded| b.iter(|| decode_avx2(black_box(encoded), ALPHABET_LOWER)),
        );

        group.bench_with_input(
            BenchmarkId::new("hex_crate", size),
            &encoded_external,
            |b, encoded| b.iter(|| hex_external::decode(black_box(encoded))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
