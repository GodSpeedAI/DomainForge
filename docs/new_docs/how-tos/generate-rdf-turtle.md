# Generate RDF/Turtle

Goal: Output SEA models as RDF/Turtle (or RDF/XML) for knowledge graph pipelines and validate them before loading into a triple store.

## Prerequisites

- SEA CLI installed with `cli` feature.
- Rust toolchain and optional SHACL feature if you need RDF/XML import validation (`cargo test -p sea-core --features "cli shacl"`).
- Optional bindings: Python/TypeScript packages built locally for programmatic export.
- A `.sea` file (e.g., `sea-core/examples/basic.sea`).

## Steps (be concise)

1. **Export to Turtle using the CLI**

   ```bash
   sea project --format kg sea-core/examples/basic.sea /tmp/basic.ttl
   ```

   - The `project` command parses the DSL, builds a graph, and writes Turtle by default.
   - The CLI prints `Projected to KG: /tmp/basic.ttl` on success.

2. **Export to RDF/XML instead of Turtle**

   ```bash
   sea project --format kg sea-core/examples/basic.sea /tmp/basic.rdf
   ```

   - The exporter inspects the output extension; `.rdf` or `.xml` triggers RDF/XML formatting.

3. **Export programmatically in Rust**

   ```rust
   use sea_core::{KnowledgeGraph, parser::parse_to_graph};

   let source = std::fs::read_to_string("sea-core/examples/basic.sea")?;
   let graph = parse_to_graph(&source)?;
   let kg = KnowledgeGraph::from_graph(&graph)?;
   let turtle = kg.to_turtle();
   std::fs::write("/tmp/basic.ttl", turtle)?;
   ```

   - Use `kg.to_rdf_xml()` if you need XML output for SHACL validators.

4. **Export programmatically in Python**

   ```python
   from sea_dsl import Graph
   import json

   graph = Graph.parse(open("sea-core/examples/basic.sea").read())
   calm = graph.export_calm()
   print("CALM length", len(calm))
   # Convert to Turtle via the CLI or call Rust from Python if you need KG directly
   ```

   - Python bindings expose CALM export; convert CALM to Turtle using the CLI (`sea import --format kg /tmp/basic.calm.json`) if you need a one-liner.

5. **Export programmatically in TypeScript**

   ```ts
   import { Graph } from "@domainforge/sea";
   import { writeFileSync } from "fs";

   const graph = Graph.parse(require("fs").readFileSync("sea-core/examples/basic.sea", "utf8"));
   const calm = graph.exportCalm();
   writeFileSync("/tmp/basic.calm.json", calm);
   ```

   - Feed the CALM output to the CLI or a Rust helper to generate Turtle. Direct KG export is currently provided by the Rust core.

6. **Validate the Turtle with SHACL (optional)**

   ```bash
   sea validate-kg --file /tmp/basic.ttl
   ```

   - Requires building with `--features shacl`. Validation fails if the KG violates the SEA SHACL shapes.

7. **Load into a triple store**

   - Use `riot --validate /tmp/basic.ttl` (Apache Jena) to check syntax.
   - Load into GraphDB or Blazegraph with the namespace you expect; Turtle output uses CURIE prefixes derived from namespaces in the source DSL.

## Metadata and Namespaces

- Default namespace resolves from `.sea-registry.toml` if present. If you do not set one, resources/entities use the `default` namespace prefix.
- Custom prefixes appear in the Turtle header. Use consistent namespace configuration to avoid collisions when merging multiple models.
- CALM metadata fields (`sea:version`, `sea:timestamp`) are not embedded directly in Turtle; store them alongside the graph in your catalog.

## Post-processing and Validation Tips

- **Minify or canonicalize**: Run `riot --out=N-TRIPLE /tmp/basic.ttl` to flatten prefixes for diff-friendly output.
- **SHACL diagnostics**: When `sea validate-kg` fails, inspect the error for the offending triple and fix the originating DSL element (often a missing role or flow ID).
- **Round-trip**: Convert DSL → Turtle → Import (`sea import --format kg`) → Export CALM to ensure no information loss before loading to production stores.

## Common Pitfalls / Troubleshooting

- Missing namespaces lead to verbose IRIs; define them in the registry to keep triples stable.
- Ensure flows have UUIDs; the KG exporter relies on deterministic IDs for edges.
- If you see `Failed to convert to Knowledge Graph`, re-run `sea validate` first—semantic errors in the DSL often surface during KG projection.

## Links

- Tutorials: [Getting Started](../tutorials/getting-started.md)
- Reference: [CALM Mapping](../reference/calm-mapping.md), [CLI Commands](../reference/cli-commands.md), [Grammar Spec](../reference/grammar-spec.md)
