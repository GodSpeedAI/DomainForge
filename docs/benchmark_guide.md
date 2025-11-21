# How to Run and Interpret Tri-State Logic Benchmarks

## Quick Start

### Run Criterion Benchmarks (Recommended)

```bash
cd sea-core
cargo bench --bench null_overhead
```

This will:

- Run comprehensive statistical analysis
- Generate HTML reports in `target/criterion/`
- Compare against previous runs (if any)
- Show confidence intervals and outlier detection

### Run Micro-Benchmark (Fast Validation)

```bash
cargo test --release -p sea-core --lib -- --ignored bench_microbench --nocapture
```

This will:

- Run a quick validation using `std::time::Instant`
- Print overhead percentage
- Pass if overhead < 35% (accounting for measurement variance)

## Understanding the Results

### Criterion Output

```
sum_baseline/sum_strict_1k
                        time:   [7.9087 µs 8.2293 µs 8.6586 µs]
```

- **First value (7.9087 µs)**: Lower bound of 95% confidence interval
- **Second value (8.2293 µs)**: **Median time** (use this for comparisons)
- **Third value (8.6586 µs)**: Upper bound of 95% confidence interval

### Interpreting Overhead

**Worst Case (0% nulls - all Some values)**:

- Baseline: 8.23 µs
- Nullable: 9.09 µs
- Overhead: **10.4%**
- **Why**: Every value must be unwrapped from `Option<T>`

**Typical Case (10% nulls)**:

- Baseline: 8.23 µs
- Nullable: 7.61 µs
- Overhead: **-7.5%** (faster!)
- **Why**: Fewer additions to perform, early null detection

**Best Case (50% nulls)**:

- Baseline: 8.23 µs
- Nullable: 4.05 µs
- Overhead: **-51%** (much faster!)
- **Why**: Half the work to do

### What This Means

1. **For dense data (few nulls)**: ~10% overhead
2. **For sparse data (many nulls)**: Actually faster
3. **For typical workloads**: ~5% overhead

## Comparing to Baseline

The baseline (`sum_strict_1k`) represents a pure `Decimal` sum with no null handling:

```rust
fn sum_strict(items: &[Decimal]) -> Decimal {
    items.iter().copied().sum()
}
```

This is the theoretical minimum time for summing 1,000 decimals.

## When to Re-run Benchmarks

Run benchmarks after:

- Modifying `three_valued.rs` aggregator functions
- Changing `ThreeValuedBool` logic
- Rust compiler upgrades
- Significant dependency updates

## Continuous Monitoring

### Save Baseline

```bash
cargo bench --bench null_overhead --save-baseline main
```

### Compare Against Baseline

```bash
cargo bench --bench null_overhead --baseline main
```

Criterion will show:

- Performance regressions (slower)
- Performance improvements (faster)
- Statistical significance

## Expected Performance Characteristics

### ✅ Good Performance

- Overhead < 15% for 0% null case
- Overhead < 10% for 10% null case
- Faster for 50% null case

### ⚠️ Investigate If

- Overhead > 20% for 0% null case
- Overhead > 15% for 10% null case
- Slower for 50% null case

### 🚨 Performance Regression

- Overhead > 30% for any case
- Significant slowdown vs previous baseline

## Troubleshooting

### Benchmarks Not Running

**Problem**: `running 0 tests`

**Solution**: Ensure `Cargo.toml` has:

```toml
[[bench]]
name = "null_overhead"
harness = false
```

### High Variance

**Problem**: Large confidence intervals or many outliers

**Solutions**:

1. Close other applications
2. Disable CPU frequency scaling
3. Run on dedicated benchmark machine
4. Increase sample size in benchmark code

### Inconsistent Results

**Problem**: Results vary significantly between runs

**Solutions**:

1. Use `--save-baseline` and `--baseline` for comparisons
2. Run multiple times and average
3. Check system load during benchmarks
4. Ensure release mode: `cargo bench` (not `cargo test`)

## Advanced Usage

### Custom Sample Size

```rust
group.sample_size(200); // Default is 100
```

### Custom Measurement Time

```rust
group.measurement_time(Duration::from_secs(10)); // Default is 5s
```

### Profiling

```bash
cargo bench --bench null_overhead --profile-time=5
```

Then use `perf` or other profilers on the generated binary.

## Files and Locations

- **Benchmark code**: `sea-core/benches/null_overhead.rs`
- **Micro-benchmark**: `sea-core/src/policy/three_valued_microbench.rs`
- **Results**: `sea-core/target/criterion/`
- **HTML reports**: `sea-core/target/criterion/*/report/index.html`
- **Documentation**: `docs/benchmark_tristate_performance.md`

## CI Integration

To run benchmarks in CI:

```yaml
- name: Run benchmarks
  run: cargo bench --bench null_overhead --no-fail-fast
```

**Note**: Benchmark results in CI may vary due to shared resources. Use dedicated benchmark runners for reliable comparisons.

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking Best Practices](https://easyperf.net/blog/2018/08/26/Basics-of-profiling)
