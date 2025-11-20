use criterion::{criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;

fn bench_sum_nullable(c: &mut Criterion) {
    use crate::policy::three_valued::aggregators;

    let mut group = c.benchmark_group("three_valued_sum");

    let data_with_null: Vec<Option<Decimal>> = (0..1000)
        .map(|i| if i % 10 == 0 { None } else { Some(Decimal::new(i, 0)) })
        .collect();

    group.bench_function("sum_nullable_1k", |b| {
        b.iter(|| {
            let _ = aggregators::sum_nullable(&data_with_null);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sum_nullable);
criterion_main!(benches);
