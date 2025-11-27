//! Benchmarks comparing our base32 implementation with the external base32 crate.

use base32::{decode, decode_avx2, encode, encode_avx2, ALPHABET_STANDARD};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

/// Sample data sizes for benchmarking
const SIZES: &[usize] = &[16, 64, 256, 1024, 4096, 16384];

fn generate_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base32_encode");

    for &size in SIZES {
        let data = generate_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("our_impl", size), &data, |b, data| {
            b.iter(|| encode(black_box(data), ALPHABET_STANDARD, true))
        });

        group.bench_with_input(BenchmarkId::new("our_impl_avx2", size), &data, |b, data| {
            b.iter(|| encode_avx2(black_box(data), ALPHABET_STANDARD, true))
        });

        group.bench_with_input(BenchmarkId::new("base32_crate", size), &data, |b, data| {
            b.iter(|| {
                base32_external::encode(
                    base32_external::Alphabet::Rfc4648 { padding: true },
                    black_box(data),
                )
            })
        });
    }

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base32_decode");

    for &size in SIZES {
        let data = generate_data(size);
        let encoded_ours = encode(&data, ALPHABET_STANDARD, true);
        let encoded_external =
            base32_external::encode(base32_external::Alphabet::Rfc4648 { padding: true }, &data);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("our_impl", size),
            &encoded_ours,
            |b, encoded| b.iter(|| decode(black_box(encoded), ALPHABET_STANDARD)),
        );

        group.bench_with_input(
            BenchmarkId::new("our_impl_avx2", size),
            &encoded_ours,
            |b, encoded| b.iter(|| decode_avx2(black_box(encoded), ALPHABET_STANDARD)),
        );

        group.bench_with_input(
            BenchmarkId::new("base32_crate", size),
            &encoded_external,
            |b, encoded| {
                b.iter(|| {
                    base32_external::decode(
                        base32_external::Alphabet::Rfc4648 { padding: true },
                        black_box(encoded),
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
