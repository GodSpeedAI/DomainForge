# cli — CLI Walkthrough Shell Scripts

## Overview
Two `set -e` bash walkthroughs that build the `domainforge` binary and exercise it
end to end. Both compute the binary path relative to the script directory
(`$SCRIPT_DIR/../../../target/debug/domainforge`) and fail fast if it is missing.

## Interface
- `import_export_workflow.sh` — builds `cargo build --bin domainforge --features cli`, writes a three-line `factory.sea`, runs `domainforge project --format calm` and `--format kg`, then `domainforge import --format kg` on the produced Turtle, and cleans up its temp files.
- `validate_example.sh` — builds the same binary, writes a one-line `sample.sea` (`Entity "Warehouse" in logistics`), runs `domainforge validate sample.sea` and `domainforge validate --format json sample.sea`, and removes the temp file.

## Invariants
- `import_export_workflow.sh` / `validate_example.sh` — both use `set -e`, so any failing step (including a non-zero CLI exit) aborts the script.
- `import_export_workflow.sh` / `validate_example.sh` — both check `[ -x "$SEA" ]` before use and exit 1 with a build hint, so the walkthrough never runs against a stale/missing binary.
- `validate_example.sh` — exercises both human and JSON output of the `validate` subcommand, so the JSON path is covered by the walkthrough as well as the human path.

## Side Effects
- `import_export_workflow.sh` — writes `factory.sea`, `factory.json`, and `factory.ttl` into the current working directory, then removes all three.
- `validate_example.sh` — writes `sample.sea` into the current working directory, then removes it.
- Both scripts — run `cargo build`, which writes to `target/`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
