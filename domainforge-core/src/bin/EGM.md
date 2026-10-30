# bin — `domainforge` Binary Entry Point

## Overview
The single-file CLI binary target. It parses the clap `Cli` struct and dispatches
each subcommand to the corresponding runner in `crate::cli`, returning
`anyhow::Result<()>`.

## Interface
- `[domainforge.rs:main]` — `fn main() -> anyhow::Result<()>`; calls `Cli::parse()` and matches `cli.command` into `parse::run`, `validate::run`, `import::run`, `project::run`, `format::run`, `test::run`, `validate_kg::run`, `normalize::run`, `registry::run`, `authority::run`, `pack::run`, `contract::run`, `envelope::run`.

## Invariants
- `[domainforge.rs:main]` — the match is exhaustive over `Commands`, so adding a subcommand without wiring a runner is a compile error.
- `[domainforge.rs:main]` — global `verbose`/`quiet` flags are parsed but not yet consumed; a `TODO` marks the planned logging/tracing setup.

## Side Effects
- None observed in source beyond what the dispatched runner does. The binary itself performs no I/O; all file reads, writes, and stdout/stderr output happen inside `crate::cli::*::run`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
