use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_decimal::Decimal;
use sea_core::policy::three_valued_aggregators as aggregators;

/// Baseline: strict sum that assumes no nulls (for comparison)
fn sum_strict(items: &[Decimal]) -> Decimal {
    items.iter().copied().sum()
}

fn bench_sum_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_baseline");

    // Baseline: no Option wrapper at all
    let data_no_option: Vec<Decimal> = (0..1000)
        .map(|i| Decimal::new(i, 0))
        .collect();

    group.bench_function("sum_strict_1k", |b| {
        b.iter(|| {
            let _ = sum_strict(&data_no_option);
        })
    });

    group.finish();
}

fn bench_sum_nullable(c: &mut Criterion) {
    let mut group = c.benchmark_group("three_valued_sum");

    // Test with 10% nulls
    let data_10pct_null: Vec<Option<Decimal>> = (0..1000)
        .map(|i| if i % 10 == 0 { None } else { Some(Decimal::new(i, 0)) })
        .collect();

    // Test with no nulls (but still Option-wrapped)
    let data_no_null: Vec<Option<Decimal>> = (0..1000)
        .map(|i| Some(Decimal::new(i, 0)))
        .collect();

    // Test with 50% nulls
    let data_50pct_null: Vec<Option<Decimal>> = (0..1000)
        .map(|i| if i % 2 == 0 { None } else { Some(Decimal::new(i, 0)) })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sum_nullable", "0pct_null"),
        &data_no_null,
        |b, data| {
            b.iter(|| {
                let _ = aggregators::sum_nullable(data);
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("sum_nullable", "10pct_null"),
        &data_10pct_null,
        |b, data| {
            b.iter(|| {
                let _ = aggregators::sum_nullable(data);
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("sum_nullable", "50pct_null"),
        &data_50pct_null,
        |b, data| {
            b.iter(|| {
                let _ = aggregators::sum_nullable(data);
            })
        },
    );

    group.finish();
}

fn bench_sum_nonnull(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_nonnull");

    let data_10pct_null: Vec<Option<Decimal>> = (0..1000)
        .map(|i| if i % 10 == 0 { None } else { Some(Decimal::new(i, 0)) })
        .collect();

    group.bench_function("sum_nonnull_10pct_null_1k", |b| {
        b.iter(|| {
            let _ = aggregators::sum_nonnull(&data_10pct_null);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sum_baseline, bench_sum_nullable, bench_sum_nonnull);
criterion_main!(benches);

