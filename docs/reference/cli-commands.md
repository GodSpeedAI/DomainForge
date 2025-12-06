# CLI Commands

This reference captures the available CLI commands provided by the `sea` binary (feature `cli` in `sea-core`). Commands and flags mirror `sea --help` output and include examples with expected behavior.

## Conventions

- Replace `model.sea` with your own file paths.
- Commands exit non-zero on validation or parsing errors; examples note expected exit codes when relevant.
- Use `--format json` when integrating with tooling.

## Global options

```
sea --help
sea --version
```

- `--version` prints `sea-core <version>` (matches the built crate version).
- `--help` lists subcommands.

## parse

Parse a SEA file into an internal graph and print a summary.

```
sea parse path/to/model.sea
```

Options:

- `--format <human|json>`: default `human` shows counts; `json` prints the parsed graph structure.
- `--out <path>`: write output to a file instead of stdout.

Example:

```
sea parse sea-core/examples/basic.sea --format human
```

Expected output includes entity/resource/flow counts and the active namespace.

## validate

Validate syntax and semantics of a SEA file.

```
sea validate path/to/model.sea
```

Options:

- `--format <human|json>`: JSON returns a structured report with errors and warnings.
- `--allow-unknown`: treat missing references as `Unknown` instead of failing (matches three-valued logic).
- `--registry <.sea-registry.toml>`: provide namespace registry for cross-file resolution.

Exit codes:

- `0` when valid
- `1` when parse or semantic errors occur

## project

Export a model to other formats.

```
sea project --format <calm|rdf|sbvr|dsl> input.sea output.json
```

Formats:

- `calm`: FINOS CALM JSON
- `rdf`: RDF/Turtle
- `sbvr`: SBVR fact types
- `dsl`: reformat the DSL (pretty-print)

Use cases:

- `sea project --format calm model.sea calm.json` to feed downstream systems.
- `sea project --format rdf model.sea graph.ttl` to load into triple stores.

## import

Import CALM JSON into SEA DSL.

```
sea import --format calm calm.json
```

- Outputs SEA DSL to stdout by default; redirect to a file to persist.
- Use `--out` to write directly: `sea import --format calm calm.json --out restored.sea`.

## graph

Display a normalized view of the graph for debugging.

```
sea graph model.sea
```

- Prints entities, resources, flows, instances, roles, and relations with IDs.
- Useful when mapping DSL names to UUIDs.

## eval (policy evaluation)

Evaluate policies against a model.

```
sea eval --policy policy.json model.sea
```

Options:

- `--format <human|json>`: JSON exposes violation details.
- `--allow-unknown`: enable three-valued logic for incomplete data.

Policy JSON schema matches `sea-core::policy::Policy`. See [`primitives-api`](./primitives-api.md) for the policy type.

## fmt

Pretty-print a SEA file.

```
sea fmt model.sea --out formatted.sea
```

- Standardizes indentation and ordering of sections.
- Does not change semantics.

## units

List known dimensions and units registered in a model.

```
sea units model.sea
```

- Shows base units and conversion factors.
- Errors if units are undefined or circular.

## explain

Show detailed error context for validation failures.

```
sea explain --format human model.sea
```

- Equivalent to `validate` but includes suggestions from the validator (e.g., did-you-mean for unknown IDs).

## Environment variables

- `SEA_LOG`: set to `debug` to enable verbose logging (`SEA_LOG=debug sea validate model.sea`).
- `SEA_REGISTRY`: default path for `.sea-registry.toml` if not provided via `--registry`.

## Exit codes summary

- `0`: success (parse/validate/eval/export succeeded)
- `1`: parser or validation errors
- `2`: IO or configuration errors (file not found, permission issues)

## Example workflow

```bash
sea validate payment.sea \
  && sea project --format calm payment.sea calm.json \
  && sea import --format calm calm.json --out roundtrip.sea
```

- First command fails fast on syntax/semantic issues.
- Second exports CALM JSON for integration.
- Third re-imports to verify round-trip fidelity.

## Troubleshooting

- If `sea` is not found, ensure `$HOME/.cargo/bin` is on `PATH` or reinstall with `cargo install --path sea-core --features cli --force`.
- For Windows, run inside Developer PowerShell so the MSVC toolchain is available.
- Use `--format json` when parsing complex files to pinpoint the failing declaration.

## See also

- [`../how-tos/install-cli.md`](../how-tos/install-cli.md) for installation and verification.
- [`../how-tos/parse-sea-files.md`](../how-tos/parse-sea-files.md) for parsing guidance.
- [`../how-tos/export-to-calm.md`](../how-tos/export-to-calm.md) and [`../how-tos/import-from-calm.md`](../how-tos/import-from-calm.md) for data exchange.
- [`error-codes.md`](./error-codes.md) for interpreting validation failures.

## Command reference by scenario

### Batch-validate a directory

```bash
find models -name "*.sea" -print0 | xargs -0 -n1 -I{} sea validate {}
```

- Fails fast on the first invalid file when combined with `set -e` in CI scripts.

### Use a namespace registry

```toml
# .sea-registry.toml
[registry]
paths = ["./models/shared", "./models/payments"]
```

```bash
sea validate --registry .sea-registry.toml models/payments/payment.sea
```

### Emit JSON for tooling

```bash
sea parse model.sea --format json | jq '.entities[] | {name, id}'
```

- Use when integrating with code generators or graph visualizers.

### Check CLI help for a subcommand

```
sea project --help
```

- Mirrors the subcommand definitions in `sea-core/src/bin/sea.rs`.

## Command locations in the repository

- Entry point: `sea-core/src/bin/sea.rs`
- Subcommand implementations: `sea-core/src/cli/commands/*.rs`
- Argument parsing: `sea-core/src/cli/mod.rs` (clap definitions)

Reading these files ensures documentation stays aligned with the shipped binary.

## CI integration

- The repo uses `just all-tests` to run Rust, Python, and TypeScript suites; incorporate `sea validate` on sample DSL fixtures to guard docs.
- When building release artifacts, the GitHub workflows call the same CLI with `--format json` to capture machine-readable outputs.

## Legacy flags

- Older snapshots referenced `--pretty`; this is superseded by `sea fmt`.
- The `--dump-ast` debug flag is only available when building with `debug` assertions; it prints the parser AST and is unsupported in release binaries.

## Security considerations

- CLI does not execute embedded code; however, avoid running against untrusted files in directories with malicious symlinks (use `--out` to control output paths).
- When exporting CALM or RDF, check file permissions to avoid leaking sensitive model details.

## Version compatibility

- New flags are added under minor version bumps; scripts should check `sea --version` to ensure expected options exist.
- The CLI respects semantic versioning with breaking changes gated to major bumps; see [`../explanations/versioning-strategy.md`](../explanations/versioning-strategy.md).
