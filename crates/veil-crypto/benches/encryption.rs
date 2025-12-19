//! Benchmarks for encryption and hashing operations.
//!
//! Run with: `cargo bench -p veil-crypto`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use veil_crypto::{
    encrypt, hash, tokenize, EncryptionConfig, HashConfig, InMemoryVault, TokenConfig, TokenVault,
};

fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn create_test_key() -> Vec<u8> {
    vec![0u8; 32] // 256-bit key
}

fn bench_encryption(c: &mut Criterion) {
    let key = create_test_key();
    let config = EncryptionConfig::new(key);

    let mut group = c.benchmark_group("encryption");

    for size in [64, 1024, 64 * 1024, 1024 * 1024].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("aes_gcm", size), &data, |b, data| {
            b.iter(|| encrypt(black_box(data), black_box(&config)))
        });
    }

    group.finish();
}

fn bench_hashing(c: &mut Criterion) {
    let config_sha256 = HashConfig::sha256();
    let config_sha512 = HashConfig::sha512();

    let mut group = c.benchmark_group("hashing");

    for size in [64, 1024, 64 * 1024, 1024 * 1024].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("sha256", size), &data, |b, data| {
            b.iter(|| hash(black_box(data), black_box(&config_sha256)))
        });

        group.bench_with_input(BenchmarkId::new("sha512", size), &data, |b, data| {
            b.iter(|| hash(black_box(data), black_box(&config_sha512)))
        });
    }

    group.finish();
}

fn bench_tokenization(c: &mut Criterion) {
    let vault: Arc<dyn TokenVault> = Arc::new(InMemoryVault::new());
    let config = TokenConfig::new();

    c.bench_function("tokenize_short_string", |b| {
        b.iter(|| {
            tokenize(
                black_box("test@example.com"),
                black_box(&config),
                black_box(Arc::clone(&vault)),
            )
        })
    });

    c.bench_function("tokenize_credit_card", |b| {
        b.iter(|| {
            tokenize(
                black_box("4111111111111111"),
                black_box(&config),
                black_box(Arc::clone(&vault)),
            )
        })
    });
}

fn bench_hashing_with_salt(c: &mut Criterion) {
    let salt = b"random_salt_value".to_vec();
    let config = HashConfig::sha256().with_salt(salt);
    let data = b"test@example.com";

    c.bench_function("hash_with_salt", |b| {
        b.iter(|| hash(black_box(data), black_box(&config)))
    });
}

criterion_group!(
    benches,
    bench_encryption,
    bench_hashing,
    bench_tokenization,
    bench_hashing_with_salt
);

criterion_main!(benches);
