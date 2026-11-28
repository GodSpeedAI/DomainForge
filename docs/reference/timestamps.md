# Timestamps and durations

DomainForge uses the [`time`](https://crates.io/crates/time) crate (0.3) for instants and durations. It replaced `chrono` in 0.23.0 because chrono is unmaintained (RUSTSEC-2026-0543). No new crate entered the dependency tree.

Public timestamp fields are `time::OffsetDateTime` (always UTC when DomainForge creates them) and durations are `time::Duration`.

## Why a `timefmt` module

Signed, digested and wire timestamps must keep their exact bytes, and `time`'s defaults differ from chrono's. Every format and parse therefore goes through `domainforge_core::timefmt`:

| Function | Output | Used by |
|---|---|---|
| `to_rfc3339` | `2024-01-02T03:04:05.5+00:00` (fraction of 0, 3, 6 or 9 digits) | fact signing payload, trace, pack and projection `created_at` defaults, CALM and protobuf `SOURCE_DATE_EPOCH` |
| `to_rfc3339_z` | `2024-01-02T03:04:05.500Z` (same fraction rule) | `FactEnvelope`, `AuthorityRequest`, typed timestamp values, canonical hashing |
| `to_rfc3339_micros_z` | `2024-01-02T03:04:05.500000Z` | `domainforge envelope` `created_at` |
| `parse_rfc3339` | strict RFC 3339 | policy `before`/`after`, validators, `--created-at` |
| `parse_relaxed` | the relaxed form chrono's serde accepted | serde adapters |

Years outside 0 to 9999 are written with a sign (`+10000`, `-0001`). Duration serde is the tuple `[secs, nanos]` with the seconds floored.

`domainforge-core/tests/timestamp_golden_tests.rs` and the corpora in `tests/fixtures/timestamp_golden/` were recorded from chrono and pin every byte and every accepted input.

## Behaviour change: leap seconds

chrono kept a leap second (`23:59:60`) and wrote `:60` back out. `time` cannot represent one.

- Comparison-only parsing (policy `before`/`after`, validators, `--created-at`) accepts `:60` and treats it as the next second.
- Anything that stores a value (serde, typed timestamps) rejects `:60` with `leap seconds (:60) are not representable`, so it can never emit bytes different from the input.

Timestamps DomainForge generates itself never contain a leap second.
