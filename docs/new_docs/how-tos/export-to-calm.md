# Export to CALM

Goal: Export a SEA DSL model to FINOS CALM and verify the payload is valid.

## Prerequisites

- SEA CLI installed (`cargo install --path sea-core --features cli`).
- A validated `.sea` model file with entities/resources/flows (and optional roles/relations).
- Optional: Python/TypeScript bindings for programmatic export.
- JSON tooling such as `jq` for inspection.

## Steps (be concise)

1. **Validate the model first (CLI)**

   ```bash
   sea validate --format human path/to/model.sea
   ```

   - Fails fast on syntax/semantic errors; fix these before exporting.

2. **Export to CALM JSON (CLI)**

   ```bash
   sea project --format calm path/to/model.sea calm.json
   ```

   - On success, the command prints `Projected to CALM: calm.json`.
   - Output includes `metadata.sea:version` set to the current SEA version (e.g., `0.1.0`).

3. **Inspect the output**

   ```bash
   jq '.metadata, (.models[0].roles | length), (.models[0].relations | length)' calm.json
   ```

   - Confirm roles/relations/flows are present and IDs look like UUIDs.

4. **Re-import to double-check**

   ```bash
   sea import --format kg calm.json
   ```

   - Uses the KG importer to ensure the CALM payload can be transformed back into a graph.

5. **Export programmatically in Python**

   ```python
   from sea_dsl import Graph

   graph = Graph.parse(open("path/to/model.sea").read())
   calm_json = graph.export_calm()
   open("calm.json", "w").write(calm_json)
   ```

6. **Export programmatically in TypeScript**

   ```ts
   import { Graph } from "@domainforge/sea";
   import { readFileSync, writeFileSync } from "fs";

   const graph = Graph.parse(readFileSync("path/to/model.sea", "utf8"));
   const calm = graph.exportCalm();
   writeFileSync("calm.json", calm);
   ```

7. **Validate against the CALM schema (optional)**

   ```bash
   ajv validate -s calm.schema.json -d calm.json
   ```

   - Use the FINOS Architecture-as-Code schema for strict validation if a downstream system requires it.

## Mapping Notes

- Roles and relations emit as typed facts; verify `subjectRole`, `predicate`, and `objectRole` fields reference valid role IDs.
- Instances and resource quantities retain their units; ensure custom units are defined before export to avoid validation failures.
- Namespaces: unset namespaces default to `"default"` and may be omitted in the CALM output.
- Metadata includes `sea:exported`, `sea:version`, and `sea:timestamp`; keep them for traceability.

## Validation Tips

- Use `jq` to spot-check key sections (roles, relations, flows) and ensure UUIDs are present.
- If a relation references a flow ID, confirm the flow exists and carries a valid UUID; missing IDs cause export failures.
- For large models, prefer `sea project --format calm input.sea >(gzip > calm.json.gz)` to stream-compress during export.

## Troubleshooting

- **Parse errors**: Run `sea validate` before exporting; export stops on parse failure.
- **Missing roles/relations**: Ensure they are declared in the DSL; empty arrays in CALM indicate nothing was declared.
- **Schema validation failures**: Align with the official CALM schema and update your DSL model to supply required fields (e.g., `id`, `name`).
- **Version drift**: If consumers require a specific SEA version, check `metadata["sea:version"]` and consider pinning the toolchain.

## Verification Checklist

- [ ] `sea validate` passes on the source `.sea` file.
- [ ] `sea project --format calm` succeeds and outputs metadata with `sea:version`.
- [ ] CALM payload re-imports through `sea import --format kg` without losing entities/resources/roles/relations.
- [ ] Optional schema validation (AJV) passes when required by downstream systems.

## Links

- Tutorials: [Getting Started](../tutorials/getting-started.md)
- Reference: [CALM Mapping](../reference/calm-mapping.md), [CLI Commands](../reference/cli-commands.md), [Primitives API](../reference/primitives-api.md)
