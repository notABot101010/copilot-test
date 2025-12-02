//! Benchmarks for TurboSHAKE, KangarooTwelve, and AEAD

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use turboshake::{KT128, KT256, TurboShake128, TurboShake256, TurboShakeAead};

/// Sample data sizes for benchmarking
const SIZES: &[usize] = &[16, 64, 256, 1024, 4096, 16384];

fn generate_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_turboshake256(c: &mut Criterion) {
    let mut group = c.benchmark_group("turboshake256");

    for &size in SIZES {
        let input = generate_data(size);
        let mut output = vec![0u8; 64];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                TurboShake256::hash(black_box(input), black_box(&mut output));
            });
        });
    }

    group.finish();
}

fn bench_kt256(c: &mut Criterion) {
    let mut group = c.benchmark_group("kt256");

    for &size in SIZES {
        let input = generate_data(size);
        let mut output = vec![0u8; 64];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                KT256::hash(black_box(input), &[], black_box(&mut output));
            });
        });
    }

    group.finish();
}

fn bench_aead_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("aead_encrypt");

    let key = [0x42u8; 32];
    let nonce = [0x13u8; 16];
    let ad = b"associated data";

    for &size in SIZES {
        let plaintext = generate_data(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &plaintext, |b, pt| {
            b.iter(|| {
                let mut aead = TurboShakeAead::new(&key, &nonce).unwrap();
                black_box(aead.encrypt(black_box(pt), ad))
            });
        });
    }

    group.finish();
}

fn bench_aead_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("aead_decrypt");

    let key = [0x42u8; 32];
    let nonce = [0x13u8; 16];
    let ad = b"associated data";

    for &size in SIZES {
        let plaintext = generate_data(size);
        let mut aead = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = aead.encrypt(&plaintext, ad);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &ciphertext, |b, ct| {
            b.iter(|| {
                let mut aead = TurboShakeAead::new(&key, &nonce).unwrap();
                black_box(aead.decrypt(black_box(ct), ad).unwrap())
            });
        });
    }

    group.finish();
}

fn bench_aead_encrypt_in_place(c: &mut Criterion) {
    let mut group = c.benchmark_group("aead_encrypt_in_place");

    let key = [0x42u8; 32];
    let nonce = [0x13u8; 16];
    let ad = b"associated data";

    for &size in SIZES {
        let plaintext = generate_data(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &plaintext, |b, pt| {
            b.iter(|| {
                let mut buffer = pt.clone();
                let mut aead = TurboShakeAead::new(&key, &nonce).unwrap();
                aead.encrypt_in_place(black_box(&mut buffer), ad);
                black_box(buffer)
            });
        });
    }

    group.finish();
}

fn bench_aead_decrypt_in_place(c: &mut Criterion) {
    let mut group = c.benchmark_group("aead_decrypt_in_place");

    let key = [0x42u8; 32];
    let nonce = [0x13u8; 16];
    let ad = b"associated data";

    for &size in SIZES {
        let plaintext = generate_data(size);
        let mut aead = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = aead.encrypt(&plaintext, ad);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &ciphertext, |b, ct| {
            b.iter(|| {
                let mut buffer = ct.clone();
                let mut aead = TurboShakeAead::new(&key, &nonce).unwrap();
                aead.decrypt_in_place(black_box(&mut buffer), ad).unwrap();
                black_box(buffer)
            });
        });
    }

    group.finish();
}

fn bench_turboshake256_variable_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("turboshake256_output_size");
    let input = generate_data(1024);

    for &out_size in &[32, 64, 128, 256] {
        let mut output = vec![0u8; out_size];

        group.throughput(Throughput::Bytes(out_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(out_size),
            &input,
            |b, input| {
                b.iter(|| {
                    TurboShake256::hash(black_box(input), black_box(&mut output));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    hash_benches,
    bench_turboshake256,
    bench_turboshake256_variable_output,
);

criterion_group!(kangaroo_benches, bench_kt256);

criterion_group!(
    aead_benches,
    bench_aead_encrypt,
    bench_aead_decrypt,
    bench_aead_encrypt_in_place,
    bench_aead_decrypt_in_place,
);

criterion_main!(hash_benches, kangaroo_benches, aead_benches);
