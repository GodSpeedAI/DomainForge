# Reference — DomainForge

Purpose: Canonical descriptions of APIs, data models, grammar, CLI commands, configuration and error codes.

MECE checklist:

- Stable API surface docs and usage examples
- Grammar and AST specification
- Error codes with examples and recommended handling
- CLI reference command list with flags and examples
- Configuration schema and environment variables

## How to use this section

- Start with `grammar-spec.md` to understand supported syntax.
- Consult `primitives-api.md` to see the in-memory structures created by parsing.
- Use `cli-commands.md` when working from the terminal or CI.
- Binding-specific references (`python-api.md`, `typescript-api.md`, `wasm-api.md`) describe constructors, methods, and platform notes.
- `error-codes.md` provides remediation steps for validation output.
- `calm-mapping.md` documents integration with FINOS CALM.
- `protobuf-api.md` documents the Protobuf projection engine API.
- `configuration.md` centralizes environment variables and registry options.
- `registry.md` documents workspace namespace mappings (`.sea-registry.toml`).

## File map and highlights

### grammar-spec.md

- Pest grammar rules mirrored verbatim from `sea.pest`.
- Examples for every declaration type (namespace, dimension, unit, entity, resource, flow, instance, role, relation, policy).
- Edge cases (escaping, numeric formats, regex validation) documented for parser debugging.

### primitives-api.md

- Fields and constructors for Entity, Resource, Flow, ResourceInstance, Role, Relation, Policy.
- Cross-language method parity tables.
- JSON serialization examples and CALM mapping notes.

### cli-commands.md

- Subcommand-by-subcommand reference with flags and exit codes.
- Scenario-driven examples (batch validation, registry usage, JSON output piping).
- Pointers to CLI source files for maintainers.

### python-api.md

- Installation instructions (pip, maturin, platform wheels).
- Usage examples for Graph parsing, export/import, policy evaluation.
- Exception types and how validation errors surface in Python.

### typescript-api.md

- npm installation guidance (Node 18+, napi prebuilds).
- Graph API coverage with examples using Vitest.
- Notes on CommonJS vs ESM and bundler integration.

### wasm-api.md

- Browser and bundler setup (wasm-pack, npm scripts, MIME types).
- API surface for parsing/exporting in WASM contexts.
- Performance and memory considerations for large graphs.

### error-codes.md

- Exhaustive catalog of validation and parsing errors grouped by category.
- Examples demonstrating each code and suggested fixes.
- Binding notes on how errors are propagated.

### calm-mapping.md

- Bidirectional mapping tables between SEA primitives and CALM JSON fields.
- Implementation pointers to export/import modules and round-trip tests.
- Known limitations and schema alignment notes.

### configuration.md

- CLI/environment defaults, registry schema, binding configuration flags.
- Platform-specific guidance and migration notes across versions.

## Navigational checklist by task

- **Model authoring**: read `grammar-spec.md` and `primitives-api.md` first.
- **Running tools**: rely on `cli-commands.md` and `configuration.md`.
- **Debugging**: use `error-codes.md` plus `cli-commands.md` (`explain`), optionally cross-check `calm-mapping.md` for projection issues.
- **Integrations**: choose the binding reference that matches your runtime (Python/TS/WASM) and pair with `calm-mapping.md` or RDF guides in how-tos.

## Maintenance guidance

- When updating parser rules, edit `grammar-spec.md` and add examples mirroring new syntax.
- Any new primitive fields must be reflected in `primitives-api.md` and binding references.
- New CLI flags require updates to `cli-commands.md` and `configuration.md`.
- Adding or changing validation errors demands edits to `error-codes.md` and backfill of examples.
- Projection changes (CALM/RDF/SBVR) should be mirrored in `calm-mapping.md` with notes on compatibility.

## Cross-references

- Tutorials and how-tos in `docs/new_docs/tutorials/` and `docs/new_docs/how-tos/` provide guided flows; use this reference section to double-check APIs used there.
- Explanations in `docs/new_docs/explanations/` describe the rationale behind design choices documented here.

## Glossary

- **Namespace**: logical partition for IDs; default is `"default"`.
- **Dimension/Unit**: measurement system for quantities; see `create-custom-units` how-to for authoring guidance.
- **Policy**: logical statement evaluated over graph elements; see `policy-evaluation-logic.md` explanation.
- **CALM**: FINOS Architecture-as-Code JSON format for interoperability.

## Sample workflow tying references together

1. Review `grammar-spec.md` to model a payments domain with roles and relations.
2. Use `cli-commands.md` to parse and validate the DSL: `sea validate payments.sea --format human`.
3. If errors appear, look up codes in `error-codes.md` and adjust accordingly.
4. Export to CALM using `cli-commands.md` guidance, then cross-check fields using `calm-mapping.md`.
5. Embed the graph into your runtime using `python-api.md` or `typescript-api.md` examples.
6. Configure CI with settings from `configuration.md` to keep validation automated.

## Example table of common references

| Topic               | Primary file      | Secondary references                                   |
| ------------------- | ----------------- | ------------------------------------------------------ |
| Syntax              | grammar-spec.md   | explanations/architecture-overview.md                  |
| API structs         | primitives-api.md | python-api.md, typescript-api.md                       |
| CLI usage           | cli-commands.md   | how-tos/install-cli.md                                 |
| Errors              | error-codes.md    | how-tos/debugging-parser-failures.md                   |
| CALM integration    | calm-mapping.md   | how-tos/export-to-calm.md, how-tos/import-from-calm.md |
| Protobuf projection | protobuf-api.md   | how-tos/export-to-protobuf.md                          |
| Configuration       | configuration.md  | how-tos/run-cross-language-tests.md                    |

## See also

- `docs/new_docs/how-tos/` for task-based guidance.
- `docs/new_docs/tutorials/` for guided, end-to-end walkthroughs.
- `docs/new_docs/playbooks/` for operational procedures (releases, migrations).

## Quality bar for reference updates

- **Accuracy**: validate examples against current code. Run `cargo test -p sea-core --features "cli,python,typescript"` when modifying APIs.
- **Completeness**: ensure new primitives or flags are reflected across all relevant reference files.
- **Clarity**: prefer short code snippets with expected outputs; avoid pseudocode.
- **Traceability**: link to source files and tests that define behavior.

## Review checklist

1. Are grammar changes mirrored in `grammar-spec.md` with examples?
2. Are new fields or methods documented in `primitives-api.md` and binding references?
3. Do CLI flag additions include exit code expectations?
4. Are new error codes documented with example inputs and fixes?
5. Do CALM/RDF projection changes mention compatibility considerations?
6. Are configuration defaults and environment variables updated?
7. Are cross-references updated so tutorials/how-tos remain accurate?

## Updating this directory

When adding or editing reference files:

- Follow the heading hierarchy (H1 title, H2 sections, H3 subsections).
- Keep line-wrapped bullet lists for readability.
- Include “See also” links at the bottom of each file to guide readers to related content.
- Use Markdown tables where comparisons help (e.g., CLI flags, binding parity).

## Frequently asked questions

- **Where do the binding APIs come from?**
  - Python: `sea-core/src/python/` via PyO3 classes.
  - TypeScript: `sea-core/src/typescript/` via napi-rs.
  - WASM: `sea-core/src/wasm/` via wasm-bindgen.
- **How stable is the CLI?**
  - Semantic versioning applies; minor releases add flags, major releases break.
- **Can I generate docs automatically?**
  - Use `sea --help` and bindings’ docstrings as sources; this reference is hand-curated to stay concise.

## Example contribution workflow

```bash
# Update grammar-spec after adding a new declaration
vim docs/new_docs/reference/grammar-spec.md
just fmt
just all-tests
```

- Submit PR with links to relevant tests verifying the change.
- Keep commit messages descriptive (e.g., "Document new module import syntax").

## Contact and ownership

- Primary maintainers: see `CODEOWNERS` and `docs/plans/` for domain leads.
- File issues with documentation bugs referencing the file path and version.

## Release alignment

- Align documentation updates with version bumps recorded in `CHANGELOG.md`.
- When cutting releases, verify reference links against tagged source to avoid drift.
- Archive previous versions in branch tags if API changes are significant.

## Metrics for completeness

- Each reference file should include: summary, API or schema details, examples with expected outputs, troubleshooting notes, and cross-references.
- Run link checkers (e.g., `markdown-link-check`) before merging large doc updates.

(End of reference overview)
