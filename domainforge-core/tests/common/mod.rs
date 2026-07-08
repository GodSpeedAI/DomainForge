//! Shared code for projection integration tests.
//!
//! Files under `tests/common/` are NOT compiled as their own test binaries;
//! each top-level test file pulls this in with `mod common;` and reaches the
//! helpers via `common::projection_harness::*`.

pub mod projection_harness;
