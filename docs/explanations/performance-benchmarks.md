# Performance Benchmarks for Three-Valued Logic

This note summarizes the tri-state (three-valued) performance benchmarks and how to run them.

## Why it matters

Policy evaluation defaults to three-valued logic. Benchmarks show the runtime overhead is modest (≈10% worst case) and often neutral or faster when data contains `Unknown` values.

## How to run

- **Criterion suite (recommended)**: statistical results and HTML reports.

  ```bash
  cd sea-core
  cargo bench --bench null_overhead
  # view reports under target/criterion/
  ```

- **Quick micro-benchmark (sanity)**:

  ```bash
  cargo test --release -p sea-core --lib -- --ignored bench_microbench --nocapture
  ```

## Results snapshot

- Criterion output shows lower/median/upper bounds of the 95% confidence interval. Use the median for comparisons.
- Sample findings from recent runs (1k items):

| Benchmark | Median (µs) | vs strict | Notes |
| --- | --- | --- | --- |
| Strict sum (baseline) | 8.23 | baseline | No null handling |
| Nullable, 0% nulls | 9.09 | +10.4% | Worst case |
| Nullable, 10% nulls | 7.61 | -7.5% | Typical |
| Nullable, 50% nulls | 4.05 | -51% | Sparse |
| Non-null, 10% nulls | 7.33 | -11% | Filters nulls |

- Micro-benchmark sanity check: ~4.9% overhead at 10% nulls.

## When to re-run

- After changing policy evaluation, `ThreeValuedBool`, or Decimal handling.
- Before releases to confirm no regressions in tri-state toggling or parsing hot paths.

## Appendix: raw outputs and files

- Benchmark code: `sea-core/benches/null_overhead.rs`
- Micro-bench: `sea-core/src/policy/three_valued_microbench.rs`
- Sample raw Criterion output:

  ```
  sum_baseline/sum_strict_1k
                          time:   [7.9087 µs 8.2293 µs 8.6586 µs]

  three_valued_sum/sum_nullable/0pct_null
                          time:   [8.5873 µs 9.0864 µs 9.6445 µs]
  ```

## See also

- [Three-Valued Logic](three-valued-logic.md)
- [Policy Evaluation Logic](policy-evaluation-logic.md)
- [CLI Commands](../reference/cli-commands.md) (`bench` targets are Rust-only; CI can run `cargo bench`)
