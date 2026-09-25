# CLI Commands

This reference captures the available CLI commands provided by the `domainforge` binary (feature `cli` in `domainforge-core`). Commands and flags mirror `domainforge --help` output and the clap definitions in `domainforge-core/src/cli/*.rs`, verified against the 0.17.0 source. Examples show expected behavior.

## Conventions

- Replace `model.sea` with your own file paths.
- Commands exit non-zero on validation or parsing errors; examples note expected exit codes when relevant.
- Use `--format json` when integrating with tooling.

## Global options

```
domainforge --help
domainforge --version
```

- `--version` prints `domainforge-core <version>` (matches the built crate version).
- `--help` lists subcommands.

## parse

Parse a SEA file into an internal graph and print a summary.

```
domainforge parse path/to/model.sea
```

Options:

- `--format <human|json>`: default `human` shows counts; `json` output depends on `--ast`.
  - Without `--ast`: outputs semantic Graph JSON (entities, resources, flows)
  - With `--ast`: outputs Abstract Syntax Tree JSON (declarations, source locations)
- `--ast`: output AST structure instead of semantic graph.
- `--out <path>`: write output to a file instead of stdout.

Example:

```
domainforge parse domainforge-core/examples/basic.sea --format human

# Output AST JSON
domainforge parse domainforge-core/examples/basic.sea --ast --format json

# Output Graph JSON
domainforge parse domainforge-core/examples/basic.sea --format json
```

Expected output includes entity/resource/flow counts and the active namespace. AST mode outputs tagged AST nodes (e.g., `{"type": "Entity", ...}`).

The human summary distinguishes `Resource instances:` (of a resource type) from
`Entity instances:` (of an entity type) and prints separate `Policies:` and `Patterns:`
lines. Use `parse --format json` for the complete graph, including declared units,
dimensions, policies, metrics, mappings, projections, patterns, concept changes, and
entity instances.

## validate

Validate syntax and semantics of a SEA file or a directory of SEA files.

```
domainforge validate path/to/model.sea
domainforge validate path/to/models/
```

Options:

- `--format <human|json|lsp>`: JSON returns a structured report (`{"error_count": N, "violations": [...]}`); `lsp` emits editor-oriented diagnostics.
- `--no-color`: disable colored output.
- `--show-source`: include source context with diagnostics.
- `--application`: also resolve the Application Contract (records, enums, operations) and report APP001–APP015 diagnostics. File targets only.
- `--allow-unknown`: treat missing references as `Unknown` instead of failing (matches three-valued logic).
- `--registry <.sea-registry.toml>`: provide namespace registry for cross-file resolution.

Exit codes:

- `0` when valid
- `1` when parse or semantic errors occur (parse failures print `Error: Parse failed ...` regardless of `--format`)

## project

Export a model to other formats.

```
domainforge project --format <format> [options] input.sea output
```

Run `domainforge project --help` for the full format list (29 formats at 0.17.0). Single-file
outputs: `calm`, `kg` (legacy Turtle/RDF-XML; prefer `rdf`), `protobuf`, `proto` (alias).
Directory outputs: `ai-llm`, `ai-graph-ml`, `cep-eval`, `ai-learning`, `lean`, `rdf`, `bpmn`,
`cmmn`, `archimate`, `otel-semconv`, `baml`, `dspy`, `zenml`, `cloudevents`, `asyncapi`,
`devbox`, `cell`, `dagger`, `cedar`, `gauge`, `alloy`, `tla`, `domain-python`,
`domain-typescript`, `domain-rust`.

Common options: `--namespace <ns>` (project only that namespace), `--created-at <RFC3339>`
(byte-identical output), `--base-iri <IRI>` (rdf only). Protobuf adds `--package`,
`--include-services`, `--multi-file`, `--compatibility additive|backward|breaking`, and buf
flags; AI families add `--recipe`, `--seed`, `--families`, `--authority-config`; `cell`
adds `--overrides` and `--only`.

Formats:

- `calm`: FINOS CALM JSON
- `kg`: single-file RDF (Turtle or RDF/XML by output extension)
- `rdf`: RDF/OWL dataset — Turtle + JSON-LD + OWL ontology (directory output;
  see [RDF/OWL Projection](docs/rdf-projections.md))
- `bpmn`: BPMN 2.0 process XML (non-executable subset, directory output; see
  [BPMN Projection](docs/bpmn-projections.md))
- `cmmn`: CMMN 1.1 case XML — case, roles, human tasks, milestones, and
  policy-derived sentries (directory output; see
  [CMMN Projection](docs/cmmn-projections.md))
- `sbvr`: SBVR fact types
- `dsl`: reformat the DSL (pretty-print)
- `protobuf`: Protocol Buffer `.proto` files
- `lean`: Lean 4 formal verification package (directory output; see
  [Lean 4 Projection](docs/lean-projections.md))
- `ai-llm` / `ai-graph-ml` / `cep-eval` / `ai-learning`: AI learning datasets
  (directory output; see [AI Learning Projections](docs/ai-learning-projections.md))

### Lean-specific behavior

```bash
domainforge project --format lean [--created-at <RFC3339>] input.sea output_dir/
```

Output must be a directory. `--created-at` fixes the generation timestamp for
byte-identical output. The generated package is proof-checked with `lake build`
(sorry-free `DomainForge` library; deferred policies land in `Obligations/`).

### RDF-specific behavior

```bash
domainforge project --format rdf [--created-at <RFC3339>] [--base-iri <IRI>] input.sea output_dir/
```

Output must be a directory; it receives `model.ttl` (Turtle instances),
`model.jsonld` (JSON-LD over the same triples), and `ontology.owl.ttl` (OWL
class/property axioms and named individuals). `--created-at` fixes the
generation timestamp and `--base-iri` overrides the `sea:` prefix expansion in
the JSON-LD and OWL files (default: the canonical SEA vocabulary namespace).
Output is byte-deterministic; validate the Turtle with
`domainforge validate-kg output_dir/model.ttl` (requires the `shacl` feature).
See [RDF/OWL Projection](docs/rdf-projections.md).

### BPMN-specific behavior

```bash
domainforge project --format bpmn [--created-at <RFC3339>] input.sea output_dir/
```

Output must be a directory; it receives a single `model.bpmn` — a non-executable
BPMN 2.0 process. Entities that participate in flows become tasks, flow
fan-out/fan-in becomes parallel gateways, flows become sequence flows, roles
become swimlanes, and resources become data objects. `--created-at` fixes the
generation timestamp for byte-identical output. Validate with
`xmllint --schema schemas/bpmn/BPMN20.xsd output_dir/model.bpmn --noout` or open
directly in bpmn.io (which auto-lays-out the diagram). See
[BPMN Projection](docs/bpmn-projections.md).

### CMMN-specific behavior

```bash
domainforge project --format cmmn [--created-at <RFC3339>] input.sea output_dir/
```

Output must be a directory; it receives a single `model.cmmn` — a CMMN 1.1
case. Resources become case-file items, roles become case roles, entities
become human tasks, and authority policies become milestones gated by
sentries derived from the policy's condition. `--created-at` fixes the
generation timestamp for byte-identical output. Validate with
`xmllint --schema schemas/cmmn/CMMN11.xsd output_dir/model.cmmn --noout`. See
[CMMN Projection](docs/cmmn-projections.md).

### Protobuf-specific options

```bash
domainforge project --format protobuf [OPTIONS] input.sea output.proto
```

| Option                   | Description                                               |
| ------------------------ | --------------------------------------------------------- |
| `--package <name>`       | Override the proto package name                           |
| `--include-services`     | Generate gRPC service definitions from Flows              |
| `--multi-file`           | Output separate `.proto` files per namespace              |
| `--output-dir <path>`    | Directory for multi-file output                           |
| `--compatibility <mode>` | Compatibility mode: `additive`, `backward`, or `breaking` |
| `--against <path>`       | Previous schema for compatibility checking                |
| `--buf-lint`             | Run buf lint on generated output (requires buf CLI)       |
| `--buf-breaking`         | Run buf breaking change detection (requires buf CLI)      |
| `--option <key>=<value>` | Set proto options (e.g., `java_package=com.example`)      |

Example:

```bash
# Basic export
domainforge project --format protobuf model.sea output.proto

# With gRPC services and custom package
domainforge project --format protobuf --include-services --package "com.example.api" model.sea api.proto

# Multi-file with buf validation
domainforge project --format protobuf --multi-file --buf-lint --output-dir ./proto model.sea
```

Use cases:

- `domainforge project --format calm model.sea calm.json` to feed downstream systems.
- `domainforge project --format kg model.sea graph.ttl` to load a single-file graph into triple stores (`rdf` gives the full directory dataset).
- `domainforge project --format protobuf model.sea schema.proto` for gRPC/binary serialization.
- `domainforge project --format lean model.sea lean_out/` to machine-check policy invariants in CI.

## import

Import SBVR XMI or Knowledge Graph (RDF) into a SEA graph. These are the only two import
formats; there is no CALM import and no `--out` flag — the command prints import graph
statistics to stdout.

```
domainforge import --format sbvr vocabulary.xmi
domainforge import --format kg graph.ttl
domainforge import --format kg graph.rdf
```

### Import from SBVR

- Converts noun concepts to entities.
- Converts verb concepts to relations.
- Converts business rules to policies.
- See `docs/how-tos/import-from-sbvr.md` (inside the DomainForge repo) for details.

### Import from Knowledge Graph (RDF)

- Auto-detects Turtle vs RDF/XML based on file extension (`.ttl`, `.turtle`, `.n3` vs `.xml`, `.rdf`) and content markers.
- RDF/XML import requires building with `--features cli,shacl`.

## validate-kg

Validate RDF/Turtle or RDF/XML files against SHACL shapes.

```
domainforge validate-kg graph.ttl
domainforge validate-kg graph.rdf
```

- Requires building with `--features cli,shacl`.
- Validates structure conforms to SEA SHACL shapes.
- Reports SHACL violations with severity and message.

## normalize

Normalize and compare policy expressions.

```
domainforge normalize [OPTIONS] <EXPRESSION>
```

Options:

- `--check-equiv <EXPR>`: Compare the input expression with another for semantic equivalence.
- `--json`: Output result as JSON object (always includes normalized string and hash; includes equivalence result only if `--check-equiv` is used).

Examples:

```bash
# Normalize an expression
domainforge normalize "b AND a"
# Output: (a AND b)

# Check equivalence
domainforge normalize "a AND b" --check-equiv "b AND a"
# Output:
# (a AND b)
# Equivalent (hash: 0x...)

# JSON output
domainforge normalize "true AND x" --json
# Output: { "normalized": "x", "hash": "0x..." }

# JSON output with equivalence
domainforge normalize "true AND x" --check-equiv "x" --json
# Output: { "normalized": "x", "hash": "0x...", "equivalent": true }
```

## Removed commands (use these equivalents)

There is no `graph`, `units`, `explain`, or `eval` subcommand. Do not invent them.

- Graph inspection: `domainforge parse --format human model.sea` for counts,
  `domainforge parse --format json model.sea` for the full graph with IDs. Useful when
  mapping DSL names to UUIDs.
- Units listing: `domainforge parse --format json model.sea | jq '.declared_units'`
  (declared units and dimensions live on the graph).
- Error explanation: `domainforge validate --format json --show-source model.sea`.
- Policy evaluation: no CLI subcommand evaluates `.sea` policies directly. Evaluate
  expressions with `domainforge normalize`, authority decisions with
  `domainforge authority`, or use the language bindings (`evaluate_policy` in the
  TypeScript/Python bindings; see `docs/reference/typescript-api.md`,
  `docs/reference/python-api.md`).

## test

```
domainforge test <pattern>
```

- Stub: prints `Test runner not yet implemented. Pattern: <pattern>` and exits 0. Run language
  suites with `just all-tests` instead.

## contract

Resolve and print one file's Application Contract document
(`domainforge-application-contract/v1`).

```
domainforge contract path/to/model.sea
```

- Takes a single file target. Other subcommands resolve the graph only, never the
  Application Contract, so operation-level errors are visible here first.

## envelope

Resolve and print the Canonical Semantic Envelope document for an entry file's resolved
closure.

```
domainforge envelope [ENTRY] [--emit representation|cep|both|verification-contract]
  [--pack <pack_id>=<content_hash>]... [--registry <file>]
  [--default-namespace <ns>] [--scope <json>] [--inline-threshold-bytes <n>]
  [-o|--out <file>] [--capabilities]
```

- Default `--emit representation` is canonical D (byte-identical across runs).
- `--capabilities` prints adapter capabilities as JSON and exits.

## registry

```
domainforge registry resolve [--fail-on-ambiguity] <file>
```

- Resolves the namespace binding for a file using nearby `.sea-registry.toml` discovery.

## authority

Evaluate authority decisions from JSON inputs (not `.sea` policies).

```
domainforge authority <config.json> <request.json> [--facts <facts.json>] [--json]
```

## fmt

Format SEA-DSL source files with consistent styling. The canonical subcommand name is
`format`; `fmt` is an alias.

```
domainforge fmt model.sea                           # Output to stdout
domainforge fmt model.sea --out formatted.sea       # Write to file
domainforge fmt --check model.sea                   # CI mode: exit 1 if changes needed
```

### Options

| Option               | Default | Description                                |
| -------------------- | ------- | ------------------------------------------ |
| `--out <path>`       | stdout  | Write formatted output to file             |
| `--check`            | false   | Check if file is formatted (exit 1 if not) |
| `--indent-width <n>` | 4       | Number of spaces per indent level          |
| `--use-tabs`         | false   | Use tabs instead of spaces                 |
| `--sort-imports`     | true    | Sort imports alphabetically                |

### Examples

```bash
# Format and display
domainforge fmt model.sea

# Format in-place (then re-validate: see round-trip warning below)
domainforge fmt model.sea --out model.sea
domainforge validate --format human model.sea

# Check in CI (fails if formatting needed)
domainforge fmt --check model.sea

# Use tabs with 2-space equivalent
domainforge fmt model.sea --use-tabs

# Custom indent width
domainforge fmt model.sea --indent-width 2
```

### Features

- **14 declaration types supported**: Entity, Resource, Flow, Pattern, Role, Relation, Instance, Policy, Metric, Dimension, Unit, ConceptChange, Mapping, Projection
- **Comment preservation**: File header comments are preserved
- **Idempotent**: Formatting twice produces the same result as formatting once
- **Import sorting**: Organizes imports alphabetically by module path (`--sort-imports`, default true)
- **Round-trip safety**: `fmt` output re-parses, including mapping/projection contracts whose keys stay unquoted identifiers. Still re-validate after formatting as a habit.

## Environment variables

- `SEA_LOG`: set to `debug` to enable verbose logging (`SEA_LOG=debug domainforge validate model.sea`).
- `SEA_REGISTRY`: default path for `.sea-registry.toml` if not provided via `--registry`.

## Exit codes summary

- `0`: success (parse/validate/eval/export succeeded)
- `1`: parser or validation errors
- `2`: IO or configuration errors (file not found, permission issues)

## Example workflow

```bash
domainforge validate payment.sea \
  && domainforge project --format calm payment.sea calm.json \
  && domainforge parse payment.sea --format json --out payment-graph.json
```

- First command fails fast on syntax/semantic issues.
- Second exports CALM JSON for integration.
- Third captures the full semantic graph for tooling (note the `declared_units`
  caveat under `parse`).

## Troubleshooting

- If `domainforge` is not found, ensure `$HOME/.cargo/bin` is on `PATH` or install with `cargo install --path domainforge-core --features cli --force`.
- For Windows, run inside Developer PowerShell so the MSVC toolchain is available.
- Use `--format json` when parsing complex files to pinpoint the failing declaration.

## See also

Repo-root-relative paths (they resolve inside the DomainForge repository):

- `docs/how-tos/install-cli.md` for installation and verification.
- `docs/how-tos/parse-sea-files.md` for parsing guidance.
- `docs/how-tos/export-to-calm.md` and `docs/how-tos/import-from-calm.md` for data exchange.
- `docs/reference/error-codes.md` for interpreting validation failures.

## Command reference by scenario

### Batch-validate a directory

```bash
find models -name "*.sea" -print0 | xargs -0 -n1 -I{} domainforge validate {}
```

- Fails fast on the first invalid file when combined with `set -e` in CI scripts.

### Use a namespace registry

```toml
# .sea-registry.toml
[registry]
paths = ["./models/shared", "./models/payments"]
```

```bash
domainforge validate --registry .sea-registry.toml models/payments/payment.sea
```

### Emit JSON for tooling

```bash
domainforge parse model.sea --format json | jq '.entities[] | {name, id}'
```

- Use when integrating with code generators or graph visualizers.

### Check CLI help for a subcommand

```
domainforge project --help
```

- Mirrors the subcommand definitions in `domainforge-core/src/bin/domainforge.rs`.

## Command locations in the repository

- Entry point: `domainforge-core/src/bin/domainforge.rs`
- Subcommand implementations: `domainforge-core/src/cli/*.rs`
- Argument parsing: `domainforge-core/src/cli/mod.rs` (clap definitions)

Reading these files ensures documentation stays aligned with the shipped binary.

## CI integration

- The repo uses `just all-tests` to run Rust, Python, and TypeScript suites; incorporate `domainforge validate` on sample DSL fixtures to guard docs.
- When building release artifacts, the GitHub workflows call the same CLI with `--format json` to capture machine-readable outputs.

## Legacy flags

- Older snapshots referenced `--pretty`; this is superseded by `domainforge fmt`.

## Security considerations

- CLI does not execute embedded code; however, avoid running against untrusted files in directories with malicious symlinks (use `--out` to control output paths).
- When exporting CALM or RDF, check file permissions to avoid leaking sensitive model details.

## Pack Commands

The `domainforge pack` subcommand provides tools for building, validating, inspecting, diffing, signing, and verifying semantic packs.

### domainforge pack build

Build a semantic pack from .sea source files.

```bash
domainforge pack build \
  --source <glob> \
  --org <id> \
  --domain <id> \
  --version <semver> \
  --meaning-version <semver> \
  --approval candidate|approved \
  --review <path> \
  --previous-pack <path> \
  --allow-first-approved-version \
  --out <path> \
  --format human|json|jsonl
```

Options:

| Option                          | Description                                                        |
|---------------------------------|--------------------------------------------------------------------|
| `--source` (required, repeatable) | Glob patterns or file paths for .sea source files.               |
| `--org` (required)              | Organization identifier.                                           |
| `--domain` (required)           | Domain identifier.                                                 |
| `--version` (required)          | Pack version (semver).                                             |
| `--meaning-version` (required)  | Meaning version (semver). Must increase when fingerprint changes.  |
| `--approval`                    | `candidate` (default) or `approved`.                               |
| `--review`                      | Path to JSONL review manifest (required for `approved`).           |
| `--previous-pack`               | Path to previous pack for version comparison.                      |
| `--allow-first-approved-version`| Allow first approved pack without a previous version.              |
| `--out`                         | Output file path. Prints to stdout if omitted.                     |
| `--format`                      | Output format: `human` (default), `json`, or `jsonl`.              |

Exit codes: `0` success, `1` semantic/build errors, `2` parse error.

Examples:

```bash
# Build a candidate pack
domainforge pack build --source "models/**/*.sea" --org acme --domain logistics \
  --version 1.0.0 --meaning-version 1.0.0 --out packs/candidate.json

# Build an approved pack with review
domainforge pack build --source "models/**/*.sea" --org acme --domain logistics \
  --version 1.1.0 --meaning-version 1.1.0 --approval approved \
  --review reviews/logistics.jsonl --previous-pack packs/candidate.json \
  --out packs/approved.json
```

### domainforge pack validate

Validate .sea source files against a semantic pack.

```bash
domainforge pack validate \
  --pack <path> \
  --mode off|warn|strict \
  --deprecated-policy allow|warn|error_in_strict|error_always \
  --require-signature \
  --expected-hash <hash> \
  --format human|json|jsonl \
  <input.sea>...
```

Options:

| Option                   | Description                                                        |
|--------------------------|--------------------------------------------------------------------|
| `--pack` (required)      | Path to the semantic pack JSON file.                               |
| `--mode`                 | `warn` (default), `off`, or `strict`.                              |
| `--deprecated-policy`    | `warn` (default), `allow`, `error-in-strict`, or `error-always`.   |
| `--require-signature`    | Fail if the pack is unsigned.                                      |
| `--expected-hash`        | Pin the expected pack content hash for tamper protection.          |
| `--format`               | Output format: `human` (default), `json`, or `jsonl`.              |
| `<input.sea>...`         | One or more .sea files to validate.                                |

Exit codes: `0` passed, `1` validation errors, `2` parse error, `3` unsigned pack, `4` signature failure.

Examples:

```bash
# Warn-mode validation
domainforge pack validate --pack packs/approved.json models/*.sea

# Strict CI validation
domainforge pack validate --pack packs/signed.json --mode strict \
  --require-signature --expected-hash sha256:abc123... models/**/*.sea
```

### domainforge pack inspect

Display metadata and contents of a semantic pack.

```bash
domainforge pack inspect [--format human|json|jsonl] <pack-path>
```

Options:

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `--format`  | `human` (default), `json`, or `jsonl`.                 |

Example:

```bash
domainforge pack inspect packs/acme-logistics-1.1.0.json
domainforge pack inspect --format json packs/acme-logistics-1.1.0.json
```

### domainforge pack diff

Compare two semantic packs and classify changes.

```bash
domainforge pack diff --old <path> --new <path> [--format human|json|jsonl]
```

Options:

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `--old`     | Path to the old (base) pack.                           |
| `--new`     | Path to the new pack.                                  |
| `--format`  | `human` (default), `json`, or `jsonl`.                 |

Exit codes: `0` no breaking changes, `1` breaking changes detected.

Diff classifications: `ADD` (additive), `DEF` (definitional change), `DEP` (deprecating), `BRK` (breaking), `GOV` (governance critical), `SIG` (signature only).

Example:

```bash
domainforge pack diff --old packs/v1.0.0.json --new packs/v1.1.0.json
```

### domainforge pack sign

Sign a semantic pack with an Ed25519 private key.

```bash
domainforge pack sign <pack-path> --key <private.pem> [--out <path>]
```

Options:

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `--key`     | Path to the Ed25519 private key (PEM format).          |
| `--out`     | Output path for the signed pack. Prints to stdout if omitted. |

Exit codes: `0` success, `4` signing failure.

Example:

```bash
domainforge pack sign packs/approved.json --key keys/private.pem \
  --out packs/signed.json
```

### domainforge pack verify

Verify the Ed25519 signature of a semantic pack.

```bash
domainforge pack verify <pack-path> --key <public.pem>
```

Options:

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `--key`     | Path to the Ed25519 public key (PEM format).           |

Exit codes: `0` valid signature, `4` invalid or missing signature.

Example:

```bash
domainforge pack verify packs/signed.json --key keys/public.pem
```

## Version compatibility

- New flags are added under minor version bumps; scripts should check `domainforge --version` to ensure expected options exist.
- The CLI respects semantic versioning with breaking changes gated to major bumps; see `docs/explanations/versioning-strategy.md`.
