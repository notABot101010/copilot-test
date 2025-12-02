use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use duplex::{KeccakAead, TurboShake256, TurboShakeAead};

fn encrypt_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt");

    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let ad = b"associated data";

    for size in [16, 64, 256, 1024, 4096, 16384].iter() {
        let plaintext = vec![42u8; *size];

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut cipher = TurboShakeAead::new(&key, &nonce).unwrap();
                black_box(cipher.encrypt(&plaintext, ad))
            });
        });
    }

    group.finish();
}

fn decrypt_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt");

    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let ad = b"associated data";

    for size in [16, 64, 256, 1024, 4096, 16384].iter() {
        let plaintext = vec![42u8; *size];

        // Pre-encrypt the data
        let mut cipher = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher.encrypt(&plaintext, ad);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut cipher = KeccakAead::new(&key, &nonce).unwrap();
                black_box(cipher.decrypt(&ciphertext, ad).unwrap())
            });
        });
    }

    group.finish();
}

fn turboshake256_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("turboshake256");

    for size in [16, 64, 256, 1024, 4096, 16384].iter() {
        let input = vec![42u8; *size];
        let mut output = vec![0u8; 64]; // 64-byte output

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                TurboShake256::hash(black_box(&input), black_box(&mut output))
            });
        });
    }

    group.finish();
}

fn turboshake256_incremental_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("turboshake256_incremental");

    for size in [256, 1024, 4096].iter() {
        let input = vec![42u8; *size];
        let mut output = vec![0u8; 64];

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut hasher = TurboShake256::new();
                hasher.update(black_box(&input));
                hasher.finalize(black_box(&mut output))
            });
        });
    }

    group.finish();
}

fn turboshake256_variable_output_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("turboshake256_variable_output");
    let input = vec![42u8; 1024];

    for out_size in [32, 64, 128, 256].iter() {
        let mut output = vec![0u8; *out_size];

        group.throughput(Throughput::Bytes(*out_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(out_size), out_size, |b, _| {
            b.iter(|| {
                TurboShake256::hash(black_box(&input), black_box(&mut output))
            });
        });
    }

    group.finish();
}

criterion_group!(
    aead_benches,
    encrypt_benchmark,
    decrypt_benchmark,
    encrypt_no_ad_benchmark,
    roundtrip_benchmark
);

criterion_group!(
    turboshake_benches,
    turboshake256_benchmark,
    turboshake256_incremental_benchmark,
    turboshake256_variable_output_benchmark
);

criterion_main!(aead_benches, turboshake_benches);
