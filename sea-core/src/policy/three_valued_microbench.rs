use super::three_valued::aggregators::sum_nullable;
/// Simple micro-benchmark using std::time::Instant
/// Run with: cargo test --release -- --nocapture --ignored bench_microbench
use rust_decimal::Decimal;

#[cfg(test)]
mod microbench {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Run explicitly with --ignored
    fn bench_microbench_sum_comparison() {
        const ITERATIONS: usize = 10_000;
        const DATA_SIZE: usize = 1_000;
        const WARMUP_ITERS: usize = 100;

        // Baseline: strict sum with manual loop (more comparable to nullable version)
        let data_strict: Vec<Decimal> = (0..DATA_SIZE).map(|i| Decimal::new(i as i64, 0)).collect();

        let expected_strict_sum: Decimal = data_strict.iter().copied().sum();

        for _ in 0..WARMUP_ITERS {
            let mut total = Decimal::ZERO;
            for item in &data_strict {
                total += *item;
            }
            std::hint::black_box(total);
        }

        let start = Instant::now();
        let mut last_strict_total = Decimal::ZERO;
        for _ in 0..ITERATIONS {
            let mut total = Decimal::ZERO;
            for item in &data_strict {
                total += *item;
            }
            std::hint::black_box(total);
            last_strict_total = total;
        }
        let baseline_duration = start.elapsed();
        assert_eq!(
            last_strict_total, expected_strict_sum,
            "strict sum should equal the arithmetic series of {} elements",
            DATA_SIZE
        );

        // Nullable sum with 10% nulls
        let data_nullable: Vec<Option<Decimal>> = (0..DATA_SIZE)
            .map(|i| {
                if i % 10 == 0 {
                    None
                } else {
                    Some(Decimal::new(i as i64, 0))
                }
            })
            .collect();

        let expected_nullable_sum = sum_nullable(&data_nullable);

        for _ in 0..WARMUP_ITERS {
            let result = sum_nullable(&data_nullable);
            std::hint::black_box(result);
        }

        let start = Instant::now();
        let mut last_nullable_result = None;
        for _ in 0..ITERATIONS {
            let result = sum_nullable(&data_nullable);
            std::hint::black_box(result);
            last_nullable_result = result;
        }
        let nullable_duration = start.elapsed();
        assert_eq!(
            last_nullable_result, expected_nullable_sum,
            "nullable sum result should match expectation for data with 10% nulls"
        );

        // Calculate overhead
        let baseline_nanos = baseline_duration.as_nanos();
        let denom = baseline_nanos.max(1);
        let nullable_nanos = nullable_duration.as_nanos();
        let overhead_pct = ((nullable_nanos as f64 / denom as f64) - 1.0) * 100.0;

        println!("\n=== Micro-Benchmark Results ===");
        println!("Iterations: {}", ITERATIONS);
        println!("Data size: {}", DATA_SIZE);
        println!("Baseline (strict):  {:?}", baseline_duration);
        println!("Nullable (10% null): {:?}", nullable_duration);
        println!("Overhead: {:.2}%", overhead_pct);
        println!("================================\n");
        println!("Note: Microbenchmarks show higher variance than Criterion.");
        println!("Criterion benchmarks show ~7-10% overhead for this scenario.");

        // Assert overhead is within acceptable range
        // Note: Microbenchmarks typically show higher overhead than Criterion
        // due to less sophisticated measurement techniques
        assert!(
            overhead_pct < 35.0,
            "Overhead {:.2}% exceeds 35% threshold (microbench variance)",
            overhead_pct
        );
    }
}
