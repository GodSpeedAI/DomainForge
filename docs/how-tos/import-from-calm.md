# Import from CALM

Goal: Import a FINOS CALM representation into DomainForge, verify integrity, and convert it back to SEA DSL constructs.

## Prerequisites

- SEA CLI built from source (`cargo install --path sea-core --features cli`) for validating the imported graph.
- Access to a CALM JSON file that conforms to the FINOS Architecture-as-Code schema (version `2.0` recommended).
- Optional bindings:
  - Python package (`sea_dsl`) built via `maturin develop` for programmatic import.
  - TypeScript package (`@domainforge/sea`) built via `npm run build`.

## Steps (be concise)

1. **Inspect the CALM file**

   ```bash
   jq '.version, .metadata["sea:version"], (.models[0].roles | length)' calm.json
   ```

   - Confirm the CALM version is `2.0` and that `sea:version` matches the expected SEA baseline (e.g., `0.1.0`).

2. **Import programmatically in Python**

   ```python
   from sea_dsl import Graph
   import json

   calm_json = open("calm.json").read()
   graph = Graph.import_calm(calm_json)

   print("Entities", graph.entity_count())
   print("Roles", graph.role_count())
   ```

   - Use `graph.all_relations()` to confirm relation predicates survived the import.
   - Call `graph.export_calm()` immediately after import to verify round-trip stability (the output should still be valid CALM JSON).

3. **Import programmatically in TypeScript**

   ```ts
   import { Graph } from "@domainforge/sea";
   import { readFileSync } from "fs";

   const calm = readFileSync("calm.json", "utf8");
   const graph = Graph.importCalm(calm);

   console.log("Flows", graph.flowCount());
   console.log("Relations", graph.relationCount());
   ```

   - Re-export with `graph.exportCalm()` to prove the importer produced a valid graph.

4. **Validate the imported graph with the CLI**

   ```bash
   # Export the imported graph back to CALM then re-parse
   python - <<'PY'
   from sea_dsl import Graph
   with open('calm.json') as f:
       data = f.read()
   graph = Graph.import_calm(data)
   with open('/tmp/roundtrip.calm.json', 'w') as out:
       out.write(graph.export_calm())
   PY

   sea import --format kg /tmp/roundtrip.calm.json
   ```

   - The second command exercises the CLI import pipeline (Turtle/RDF) to ensure the graph can be converted downstream.

5. **Convert CALM to SEA DSL text (Rust)**

   ```rust
   use sea_core::calm::{import, export};

   let calm_json = std::fs::read_to_string("calm.json")?;
   let graph = import(serde_json::from_str(&calm_json)?)?;
   // You can now traverse primitives or export to KG/SBVR
   let exported = export(&graph)?;
   assert_eq!(exported["metadata"]["sea:version"], sea_core::VERSION);
   ```

   - Although there is no direct CLI flag for CALM → SEA text, the Rust API lets you transform CALM into a `Graph` and then emit other projections (`project --format kg` for Turtle/RDF).

6. **Round-trip verification**

   - Import CALM → Export CALM → Import again. Check that counts for entities/resources/flows/roles/relations remain stable.
   - Validate units: ensure every `quantity` in CALM includes `unit`; missing units will fail the SEA validator when exporting to DSL.

## Mapping and Normalisation Notes

- Namespaces default to `"default"` when not present; CALM models often omit them. SEA accepts missing namespaces but can enrich them during export.
- Relations map to `subjectRole`, `predicate`, and `objectRole`; confirm that role IDs in CALM exist in the `roles` array before import.
- Flows require UUIDs; if a CALM file omits them, the importer will generate deterministic IDs but may warn in the bindings logs.
- Metadata fields `sea:exported`, `sea:version`, and `sea:timestamp` are preserved; adjust them only if you intentionally re-sign the payload.

## Error Handling and Diagnostics

- **Schema mismatch**: If parsing fails due to a missing field, validate the CALM file against the upstream schema (`ajv validate -s calm.schema.json -d calm.json`).
- **Unknown role IDs**: Importers raise a `ValueError`/`Error` when a relation references a missing role. Add the role or correct the relation IDs.
- **Unit conversion failures**: Ensure that every resource or flow quantity uses a defined unit; otherwise, normalization to SEA quantities will fail.
- **Encoding issues**: CALM must be UTF-8. Re-save files with `iconv -f <encoding> -t utf-8` if you see decoding errors.

## Common Pitfalls / Troubleshooting

- Importing CALM via the CLI is not yet available; use Python/TypeScript/Rust APIs instead, then rely on `sea project --format kg` for downstream exports.
- When working with large models, load the JSON with streaming (`ijson` in Python) if memory is constrained; construct the `Graph` once you validate the structure.
- Keep the SEA version consistent across files; mixing CALM payloads exported from older SEA versions can surface validation differences.

## Links

- Tutorials: [Getting Started](../tutorials/getting-started.md)
- Reference: [CALM Mapping](../reference/calm-mapping.md), [CLI Commands](../reference/cli-commands.md), [Python API](../reference/python-api.md), [TypeScript API](../reference/typescript-api.md)
