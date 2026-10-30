# cli — `domainforge` Command-Line Interface

## Overview
Clap-based subcommand tree and per-command runners for the `domainforge` binary.
Each subcommand module exposes an `<X>Args` clap struct and a `run(args)` entry
point; `src/bin/domainforge.rs` dispatches `Cli::parse()` into them. The module
is gated behind `#[cfg(feature = "cli")]` in `[../lib.rs]`.

## Interface
- `[mod.rs:Cli]` — top-level clap struct: `command`, global `verbose`, global `quiet` (mutually conflicting), global `color`.
- `[mod.rs:Commands]` — the subcommand enum: `Parse`, `Validate`, `Import`, `Project`, `Format` (alias `fmt`), `Test`, `ValidateKg` (`validate-kg`), `Normalize`, `Registry`, `Authority`, `Pack`, `Contract`, `Envelope`.
- `[mod.rs:ColorChoice]` — `Auto`/`Always`/`Never`.
- `[parse.rs:ParseArgs]`, `[parse.rs:ParseFormat]` (`Human`/`Json`), `[parse.rs:run]` — parse a `.sea` file into AST or graph output.
- `[validate.rs:ValidateArgs]`, `[validate.rs:OutputFormat]` (`Json`/`Human`/`Lsp`), `[validate.rs:run]` — validate a file or directory; `--application` requires a file target.
- `[validate_kg.rs:ValidateKgArgs]`, `[validate_kg.rs:run]` — validate knowledge-graph files.
- `[format.rs:FormatArgs]`, `[format.rs:run]` — format SEA source.
- `[normalize.rs:NormalizeArgs]`, `[normalize.rs:run]`, `[normalize.rs:run_with_writer]` — normalize a policy expression; the writer-generic variant exists so tests can capture output.
- `[registry.rs:RegistryArgs]`, `[registry.rs:RegistryCommands]`, `[registry.rs:ResolveArgs]`, `[registry.rs:run]` — namespace registry management.
- `[test.rs:TestArgs]`, `[test.rs:run]` — `domainforge test`; the runner is a stub that prints "Test runner not yet implemented" and returns `Ok(())`.
- `[authority.rs:AuthorityArgs]`, `[authority.rs:run]`, `[authority.rs:run_with_writer]` — evaluate authority decisions; the writer-generic variant exists so tests can capture output.
- `[pack.rs:PackArgs]`, `[pack.rs:PackCommands]` (`build`/`validate`/`inspect`/`diff`/`sign`/`verify`), `[pack.rs:run]` — semantic pack lifecycle.
- `[pack.rs:BuildArgs]`, `[pack.rs:PackValidateArgs]`, `[pack.rs:InspectArgs]`, `[pack.rs:DiffArgs]`, `[pack.rs:SignArgs]`, `[pack.rs:VerifyArgs]` — per-subcommand argument structs.
- `[pack.rs:ApprovalChoice]` (`Candidate`/`Approved`), `[pack.rs:ValidationModeChoice]` (`Off`/`Warn`/`Strict`), `[pack.rs:DeprecatedPolicyChoice]` (`Allow`/`Warn`/`ErrorInStrict`/`ErrorAlways`), `[pack.rs:PackOutputFormat]` (`Human`/`Json`/`Jsonl`) — pack CLI enums.
- `[contract.rs:ContractArgs]`, `[contract.rs:run]` — resolve and print the Application Contract document.
- `[envelope.rs:EnvelopeArgs]`, `[envelope.rs:EmitMode]` (`Representation`/`Cep`/`Both`/`VerificationContract`), `[envelope.rs:run]` — resolve and print the Canonical Semantic Envelope; `--capabilities` prints adapter capabilities as JSON instead of resolving.
- `[import.rs:ImportArgs]`, `[import.rs:ImportFormat]` (`Sbvr`/`Kg`), `[import.rs:run]` — import from SBVR-XMI or knowledge-graph RDF.
- `[project.rs:ProjectArgs]`, `[project.rs:ProjectFormat]`, `[project.rs:CliCompatibilityMode]`, `[project.rs:run]` — project a model into an output format (including directory-output formats).

## Invariants
- `[mod.rs:Cli]` — `--verbose` and `--quiet` are declared `conflicts_with` each other, so clap rejects passing both.
- `[pack.rs]` — pack exit codes are fixed constants: `EXIT_PASSED=0`, `EXIT_SEMANTIC_FAILED=1`, `EXIT_PARSE_ERROR=2`, `EXIT_PACK_UNAVAILABLE=3`, `EXIT_SIGNATURE_FAILURE=4`, `EXIT_CONFIG_ERROR=5`, `EXIT_INTERNAL_ERROR=6`.
- `[validate.rs:run]` — `--application` is rejected for directory targets (returns an `anyhow` error).
- `[envelope.rs:EnvelopeArgs]` — `entry` is `required_unless_present = "capabilities"`; `world_alias`/`world_label` require `world_name`.
- `[envelope.rs:EmitMode]` — `Representation` output must be byte-identical across runs (canonical D only).

## Side Effects
- `[parse.rs:run]` — reads the input file with `fs::read_to_string` and writes output to disk via `fs::write` when an output path is given.
- `[validate.rs:run]`, `[import.rs:run]`, `[validate_kg.rs:run]`, `[contract.rs:run]`, `[authority.rs:run]` — read input files with `fs::read_to_string`.
- `[pack.rs]` — reads pack/source files with `fs::read_to_string` and writes pack JSON with `fs::write` on build/sign.
- `[envelope.rs:run]` — reads the entry source with `fs::read_to_string` and writes with `fs::write` when `--out` is given; calls `exit(2)` from `std::process::exit` on read failure.
- `[project.rs:run]` — creates output directories with `std::fs::create_dir_all` for directory-output projection formats.
- `[pack.rs]`, `[project.rs]`, `[envelope.rs]`, `[validate.rs]`, `[validate_kg.rs]`, `[format.rs]`, `[parse.rs]`, `[contract.rs]`, `[test.rs]`, `[registry.rs]`, `[import.rs]` — write human/JSON reporting to stdout via `println!`; `[envelope.rs:run]` also writes errors to stderr via `eprintln!`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
