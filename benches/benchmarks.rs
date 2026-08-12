//! Real benchmarks for the clustering and RFM hot paths, at a couple of
//! realistic dataset sizes. Replaces the placeholder `dummy_benchmark` that
//! previously shipped here (it measured nothing about this crate).

use clusteraudiencekit::engine::clustering::kmeans;
use clusteraudiencekit::engine::rfm::{calculate_rfm, RFMConfig, Transaction};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::Array2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Generate `n` points in `dim` dimensions, drawn from `k` well-separated
/// Gaussian-ish blobs — representative of real RFM feature data (a handful
/// of numeric dimensions, points naturally clustering into a small number
/// of customer segments) rather than uniform noise.
fn synthetic_blobs(n: usize, dim: usize, k: usize, seed: u64) -> Array2<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut data = Vec::with_capacity(n * dim);
    for i in 0..n {
        let cluster = i % k;
        let center = (cluster as f64) * 50.0;
        for _ in 0..dim {
            data.push(center + rng.gen_range(-5.0..5.0));
        }
    }
    Array2::from_shape_vec((n, dim), data).unwrap()
}

fn synthetic_transactions(
    n_customers: usize,
    txs_per_customer: usize,
    seed: u64,
) -> Vec<Transaction> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut transactions = Vec::with_capacity(n_customers * txs_per_customer);
    for cust in 0..n_customers {
        for t in 0..txs_per_customer {
            let days_ago = rng.gen_range(0..365);
            let date = chrono::Utc::now() - chrono::Duration::days(days_ago);
            transactions.push(Transaction {
                customer_id: format!("cust_{cust}"),
                date: date.to_rfc3339(),
                amount: rng.gen_range(10.0..500.0) + t as f64,
            });
        }
    }
    transactions
}

fn bench_kmeans(c: &mut Criterion) {
    let mut group = c.benchmark_group("kmeans");

    let data_10k = synthetic_blobs(10_000, 3, 5, 42);
    group.bench_function("10k_rows_3d_5clusters", |b| {
        b.iter(|| kmeans(black_box(&data_10k), 5, 50, 42).unwrap())
    });

    let data_100k = synthetic_blobs(100_000, 3, 5, 42);
    group.sample_size(10);
    group.bench_function("100k_rows_3d_5clusters", |b| {
        b.iter(|| kmeans(black_box(&data_100k), 5, 50, 42).unwrap())
    });

    group.finish();
}

fn bench_rfm(c: &mut Criterion) {
    let mut group = c.benchmark_group("rfm");

    // 10k customers x 5 transactions each = 50k transaction rows.
    let tx_10k = synthetic_transactions(10_000, 5, 7);
    let config = RFMConfig::default();
    group.bench_function("10k_customers", |b| {
        b.iter(|| calculate_rfm(black_box(tx_10k.clone()), black_box(&config)).unwrap())
    });

    // 100k customers x 5 transactions each = 500k transaction rows.
    let tx_100k = synthetic_transactions(100_000, 5, 7);
    group.sample_size(10);
    group.bench_function("100k_customers", |b| {
        b.iter(|| calculate_rfm(black_box(tx_100k.clone()), black_box(&config)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_kmeans, bench_rfm);
criterion_main!(benches);
