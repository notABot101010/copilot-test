//! Benchmarks for ChaCha stream cipher

use chacha::{ChaCha12, ChaCha20, ChaCha8};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

fn bench_chacha_encrypt(c: &mut Criterion) {
    let key = [0u8; 32];
    let nonce = [0u8; 8];

    let mut group = c.benchmark_group("chacha_encrypt");

    for size in [64, 256, 1024, 4096, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("ChaCha8", size), size, |b, &size| {
            let mut data = vec![0u8; size];
            b.iter(|| {
                let mut cipher = ChaCha8::new(black_box(&key), black_box(&nonce));
                cipher.xor_keystream(black_box(&mut data));
            });
        });

        group.bench_with_input(BenchmarkId::new("ChaCha12", size), size, |b, &size| {
            let mut data = vec![0u8; size];
            b.iter(|| {
                let mut cipher = ChaCha12::new(black_box(&key), black_box(&nonce));
                cipher.xor_keystream(black_box(&mut data));
            });
        });

        group.bench_with_input(BenchmarkId::new("ChaCha20", size), size, |b, &size| {
            let mut data = vec![0u8; size];
            b.iter(|| {
                let mut cipher = ChaCha20::new(black_box(&key), black_box(&nonce));
                cipher.xor_keystream(black_box(&mut data));
            });
        });
    }

    group.finish();
}

fn bench_chacha_vs_crate(c: &mut Criterion) {
    let key = [0u8; 32];
    // chacha20 crate uses 12-byte nonce (IETF variant)
    let nonce_ietf = [0u8; 12];
    // Our crate uses 8-byte nonce (DJB variant)
    let nonce_djb = [0u8; 8];

    let mut group = c.benchmark_group("chacha20_comparison");

    for size in [256, 1024, 4096, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        // Our implementation
        group.bench_with_input(BenchmarkId::new("ours_chacha20", size), size, |b, &size| {
            let mut data = vec![0u8; size];
            b.iter(|| {
                let mut cipher = ChaCha20::new(black_box(&key), black_box(&nonce_djb));
                cipher.xor_keystream(black_box(&mut data));
            });
        });

        // chacha20 crate
        group.bench_with_input(
            BenchmarkId::new("chacha20_crate", size),
            size,
            |b, &size| {
                let mut data = vec![0u8; size];
                b.iter(|| {
                    let mut cipher = chacha20::ChaCha20::new((&key).into(), (&nonce_ietf).into());
                    cipher.apply_keystream(black_box(&mut data));
                });
            },
        );
    }

    group.finish();
}

fn bench_chacha_partial_blocks(c: &mut Criterion) {
    let key = [0u8; 32];
    let nonce = [0u8; 8];

    let mut group = c.benchmark_group("chacha_partial_blocks");

    // Benchmark partial block handling
    for chunk_size in [1, 7, 15, 31, 63].iter() {
        group.bench_with_input(
            BenchmarkId::new("ChaCha20", chunk_size),
            chunk_size,
            |b, &chunk_size| {
                let mut data = vec![0u8; 1024];
                b.iter(|| {
                    let mut cipher = ChaCha20::new(black_box(&key), black_box(&nonce));
                    for chunk in data.chunks_mut(chunk_size) {
                        cipher.xor_keystream(black_box(chunk));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_chacha_block_generation(c: &mut Criterion) {
    let key = [0u8; 32];
    let nonce = [0u8; 8];

    let mut group = c.benchmark_group("chacha_block_generation");

    group.bench_function("ChaCha8_single_block", |b| {
        let mut data = [0u8; 64];
        b.iter(|| {
            let mut cipher = ChaCha8::new(black_box(&key), black_box(&nonce));
            cipher.xor_keystream(black_box(&mut data));
        });
    });

    group.bench_function("ChaCha12_single_block", |b| {
        let mut data = [0u8; 64];
        b.iter(|| {
            let mut cipher = ChaCha12::new(black_box(&key), black_box(&nonce));
            cipher.xor_keystream(black_box(&mut data));
        });
    });

    group.bench_function("ChaCha20_single_block", |b| {
        let mut data = [0u8; 64];
        b.iter(|| {
            let mut cipher = ChaCha20::new(black_box(&key), black_box(&nonce));
            cipher.xor_keystream(black_box(&mut data));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_chacha_encrypt,
    bench_chacha_vs_crate,
    bench_chacha_partial_blocks,
    bench_chacha_block_generation,
);

criterion_main!(benches);
