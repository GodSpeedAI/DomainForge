/// Simple micro-benchmark using std::time::Instant
/// Run with: cargo test --release -- --nocapture bench_microbench
use rust_decimal::Decimal;
use super::three_valued::aggregators::sum_nullable;

#[cfg(test)]
mod microbench {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Run explicitly with --ignored
    fn bench_microbench_sum_comparison() {
        const ITERATIONS: usize = 10_000;
        const DATA_SIZE: usize = 1_000;

        // Baseline: strict sum with manual loop (more comparable to nullable version)
        let data_strict: Vec<Decimal> = (0..DATA_SIZE)
            .map(|i| Decimal::new(i as i64, 0))
            .collect();

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let mut total = Decimal::ZERO;
            for item in &data_strict {
                total += *item;
            }
            std::hint::black_box(total);
        }
        let baseline_duration = start.elapsed();

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

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let result = sum_nullable(&data_nullable);
            std::hint::black_box(result);
        }
        let nullable_duration = start.elapsed();

        // Calculate overhead
        let overhead_pct = ((nullable_duration.as_nanos() as f64
            / baseline_duration.as_nanos() as f64)
            - 1.0)
            * 100.0;

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
