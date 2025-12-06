# Export to CALM

Goal: Export a SEA DSL model to FINOS CALM and verify the payload is valid.

## Prerequisites

- Rust toolchain installed and the CLI built (`cargo install --path sea-core --features cli`).
- A validated `.sea` model file.
- Optional: Python/TypeScript bindings if you want to export programmatically.

## Quick Steps (CLI)

1. Validate the model first: `sea validate path/to/model.sea` (fails fast on syntax/semantics).
2. Export to CALM JSON: `sea project --format calm path/to/model.sea calm.json`.
3. Inspect the output: `jq '.models[0].roles' calm.json` to confirm roles/relations are present.
4. Re-import to double-check: `sea import --format calm calm.json` (should parse without errors).

## Programmatic Export

- **Python**:

  ```python
  from sea_dsl import Graph
  graph = Graph.parse(open("model.sea").read())
  calm_json = graph.export_calm()
  open("calm.json", "w").write(calm_json)
  ```

- **TypeScript**:

  ```ts
  import { Graph } from "@domainforge/sea";
  const graph = Graph.parse(require("fs").readFileSync("model.sea", "utf8"));
  const calm = graph.exportCalm();
  require("fs").writeFileSync("calm.json", calm);
  ```

## Mapping Notes

- Roles and relations emit as typed facts; verify `subjectRole`, `predicate`, `objectRole` fields.
- Instances and resource quantities keep their units; ensure custom units are defined before export.
- Namespaces: unset namespaces default to `"default"` and may be omitted in CALM output.

## Validation Tips

- Use `jq` or `ajv` against the CALM schema if downstream requires strict validation.
- If a relation references a flow ID, ensure the flow exists and has a valid UUID; missing IDs cause export failures.
- Keep DSL and CALM files in the same namespace registry if you use `.sea-registry.toml`.
