# Parse SEA Files

Goal: Parse `.sea` models using the CLI and programmatic bindings while catching syntax/semantic errors early.

## Prerequisites

- Rust toolchain 1.77+ and the SEA CLI installed.

  - For developers building from this repo: `cargo install --path sea-core --features cli` (produces the `sea` binary).
  - For users installing from crates.io or binaries: use the published `sea` or release artifact; confirm with `sea --version`.

- Optional: Python bindings (install locally via `maturin develop --features python` or via PyPI when published) and TypeScript package (`npm install @domainforge/sea` then `npm run build`).
- A `.sea` source file. Use `sea-core/examples/basic.sea` or any model under `examples/` to follow along.

## Steps (be concise)

1. **Validate and parse a single file (CLI)**

   ```bash
   sea validate --format human sea-core/examples/basic.sea
   ```

   - The command parses the file, resolves namespaces from `.sea-registry.toml` when present, and reports semantic errors (unknown entities/resources, unit mismatches).
   - Use `--format json` for machine-readable diagnostics that include error codes from `validation_error.rs`.

2. **Parse and project to an intermediate representation (CLI)**

   ```bash
   sea project --format calm sea-core/examples/basic.sea /tmp/basic.calm.json
   ```

   - `project` parses the DSL, builds a `Graph`, then exports to CALM JSON. Swap `--format calm` with `--format kg` to emit RDF/Turtle or RDF/XML (auto-detected by extension).
   - The command fails fast if parsing fails; the error message cites the offending line/column.

3. **Batch-validate multiple files (CLI)**

   ```bash
   for f in examples/*.sea; do
     echo "Checking $f"
     sea validate --format human "$f" || exit 1
   done
   ```

   - This pattern stops on the first failure. Add `|| true` to continue despite errors.

4. **Generate AST JSON (CLI)**

   ```bash
   sea parse --ast --format json sea-core/examples/basic.sea > basic.ast.json
   ```

   - Produces raw AST JSON preserving source structure and location info.
   - Useful for feeding tools like `tools/ast_to_ir.py` or writing custom linters.

5. **Parse programmatically in Rust**

   ```rust
   use sea_core::parser::parse_to_graph;
   ```

let source = std::fs::read_to_string("sea-core/examples/basic.sea")?;
let graph = parse_to_graph(&source)?;
assert!(graph.entity_count() >= 1);

````

- Prefer `parse_to_graph_with_options` if you need namespace resolution using `NamespaceRegistry::discover`.

6. **Parse programmatically in Python**

```python
from pathlib import Path
from sea_dsl import Graph

text = Path("sea-core/examples/basic.sea").read_text()

# Semantic graph parsing
graph = Graph.parse(text)
assert graph.resource_count() >= 1

# Raw AST parsing (e.g., for tooling)
ast_json = Graph.parse_to_ast_json(text)
````

- Call `graph.all_entities()` or `graph.find_entity_by_name("Customer")` to inspect the parsed model.
- Errors raise `ValueError` with the same message as the Rust parser.

7. **Parse programmatically in TypeScript**

   ```ts
   import { readFileSync } from "fs";
   import { Graph } from "@domainforge/sea";

   const source = readFileSync("sea-core/examples/basic.sea", "utf8");

   // Semantic graph parsing
   const graph = Graph.parse(source);
   console.log(graph.entityCount());

   // Raw AST parsing
   const astJson = Graph.parseToAstJson(source);
   ```

   - Use `graph.allResources()` or `graph.findRoleByName("Payer")` after parsing relation-enabled models.

8. **Handle parse warnings and errors**

   - **Unknown identifiers**: The parser emits `UnknownEntity` or `UnknownResource` errors; add missing declarations or fix typos.
   - **Unit mismatches**: If a flow references an undefined unit, define it in the `Dimension/Unit` section before the flow.
   - **Namespace resolution**: If you see `NamespaceMissing`, create `.sea-registry.toml` near the entry file or pass fully qualified names (e.g., `finance/Invoice`).

9. **Integrate parsing into CI**

   ```bash
   just rust-test  # runs `cargo test -p sea-core --features cli` and exercises parser paths
   just python-test  # rebuilds bindings (maturin develop) and parses DSL fixtures
   just ts-test  # runs Vitest against `Graph.parse`
   ```

   - Combine with `cargo fmt --all -- --check` and `cargo clippy -p sea-core -- -D warnings` for full validation.

## Common Pitfalls / Troubleshooting

- **Using stale bindings**: After editing grammar or parser code, rebuild Python (`maturin develop`) and TypeScript (`npm run build`) bindings so `Graph.parse` reflects the changes.
- **File encoding**: Ensure `.sea` files are UTF-8; unusual whitespace can trigger unexpected token errors around indentation-sensitive sections.
- **Entry path awareness**: When parsing embedded strings (not files), namespace discovery cannot infer the path; pass `ParseOptions` with `entry_path` if you rely on registry lookups.
- **Large batches**: Wrap CLI calls with `xargs -P` or `GNU parallel` to speed up validation across many files; keep an eye on memory if models are very large.

## Links

- Tutorials: [Getting Started](../tutorials/getting-started.md), [First SEA Model](../tutorials/first-sea-model.md)
- Reference: [Grammar Spec](../reference/grammar-spec.md), [CLI Commands](../reference/cli-commands.md), [Primitives API](../reference/primitives-api.md)
