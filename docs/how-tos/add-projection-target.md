# How-To: Add a New Projection Target (ADR-011)

This guide walks through the exact procedure for adding a new operator family projection target to DomainForge, following the architecture specified in [ADR-011](../specs/ADR-011-operator-backed-projection-families.md).

---

## Goal
Add a new projection format (e.g. `--format mytarget`) that reads an in-memory `Graph` and writes deterministic artifacts via `ArtifactSink`.

---

## Prerequisites
- Working Rust development environment with `cargo` and `just`.
- Repository dependencies installed (`just setup`).

---

## Procedure

### Step 1: Create the Subsystem Module
Create a directory `domainforge-core/src/projection/<family>/`:
- `domainforge-core/src/projection/<family>/mod.rs`
- `domainforge-core/src/projection/<family>/ir.rs` (if target requires semantic translation)

Implement the pure rendering function:
```rust
use crate::graph::Graph;
use crate::projection::ids::element_id;
use crate::projection::sink::ArtifactSink;

pub fn emit(graph: &Graph, _config: &(), sink: &mut ArtifactSink) -> Result<(), String> {
    for (id, entity) in graph.entities() {
        let node_id = element_id("<family>", &[entity.namespace(), entity.name()]);
        let rendered_content = format!("entity {} [id: {}];\n", entity.name(), node_id);
        sink.write(&format!("{}.txt", entity.name()), &rendered_content)?;
    }
    Ok(())
}
```

### Step 2: Register in `projection/mod.rs`
Open `domainforge-core/src/projection/mod.rs` and expose the module:
```rust
pub mod <family>;
```

### Step 3: Add the CLI Format Flag
Open `domainforge-core/src/cli/project.rs`:
1. Add `<family>` to the `ProjectFormat` enum:
   ```rust
   #[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
   pub enum ProjectFormat {
       // ...
       MyTarget,
   }
   ```
2. In the `run(args)` match block, wire the dispatcher:
   ```rust
   ProjectFormat::MyTarget => {
       crate::projection::<family>::emit(&graph, &(), &mut sink)?;
   }
   ```

### Step 4: Create a Verification Gate Script
Create `scripts/verify/projection-targets/<family>.sh`:
```bash
#!/usr/bin/env bash
set -euo pipefail

OUT_DIR=$(mktemp -d)
trap 'rm -rf "$OUT_DIR"' EXIT

cargo run -p domainforge-core --features cli -- project \
  --format <family> \
  fixtures/projection_cell/basic/model.sea \
  "$OUT_DIR"

# Run target ecosystem native validator or structural check
test -f "$OUT_DIR/Buyer.txt"
echo "<family> projection verified successfully!"
```
Make the script executable: `chmod +x scripts/verify/projection-targets/<family>.sh`.

### Step 5: Add an Integration Test
Add a test in `domainforge-core/tests/<family>_projection_tests.rs` asserting byte-identical output across two runs with a fixed `--created-at`.

---

## Validation
Run your gate script:
```bash
bash scripts/verify/projection-targets/<family>.sh
```
Run all projection gates:
```bash
bash scripts/verify/projection-targets/all.sh
```

---

## Common Failures
- **Path Traversal Error**: If your renderer writes `../file.txt`, `ArtifactSink` will reject it. Always use relative paths inside the target directory.
- **Non-Deterministic Diff**: If `diff -r` fails across two runs, check if you iterated a `HashMap` instead of `IndexMap`, or forgot to use `projection::ids::element_id()`.
