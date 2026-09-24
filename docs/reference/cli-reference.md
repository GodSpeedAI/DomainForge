# Reference: CLI Commands & Options

This document provides a comprehensive technical reference for the `domainforge` command-line binary.

---

## 1. Global Options

```bash
domainforge [OPTIONS] <COMMAND>
```

| Option | Values | Default | Description |
|---|---|---|---|
| `--verbose` | Flag | `false` | Enable verbose debug logging output. Conflicts with `--quiet`. |
| `--quiet` | Flag | `false` | Suppress all non-error output. Conflicts with `--verbose`. |
| `--color` | `auto`, `always`, `never` | `auto` | Control colorized terminal output. |
| `--help`, `-h` | Flag | N/A | Print help information for the binary or subcommand. |
| `--version`, `-V` | Flag | N/A | Print version information (`domainforge-core X.Y.Z`). |

---

## 2. Subcommands Reference

### `parse`
Parse SEA DSL source files and print summary or JSON AST/Graph.

```bash
domainforge parse [OPTIONS] <PATH>
```
- `<PATH>`: Path to entry `.sea` file.
- `--ast`: Emit Abstract Syntax Tree JSON instead of relational Graph.
- `--format <human|json>`: Output format. Default: `human`.
- `--out <PATH>`: Write output to file instead of stdout.

---

### `validate`
Validate syntax, schemas, units, and evaluate policy expressions across the transitive import closure.

```bash
domainforge validate [OPTIONS] <PATH>
```
- `<PATH>`: Path to entry `.sea` file.
- `--format <human|json>`: Output format. Default: `human`.
- `--allow-unknown`: Treat policy `Null` / `Unknown` outcomes as passing instead of errors.
- `--registry <PATH>`: Optional path to `.sea-registry.toml`.

---

### `project`
Project a semantic model into an external ecosystem format.

```bash
domainforge project [OPTIONS] --format <FORMAT> <INPUT> <OUTPUT>
```
- `<INPUT>`: Path to source `.sea` model.
- `<OUTPUT>`: Directory or file destination for emitted artifacts.
- `--format <FORMAT>`: Target operator family. Supported formats:
  - `rdf`, `kg`: Semantic Graph (Turtle, JSON-LD, OWL)
  - `calm`: FINOS Common Architecture Language Model JSON
  - `bpmn`: BPMN 2.0 Process XML
  - `cmmn`: CMMN 1.1 Case XML
  - `archimate`: ArchiMate 3.0 Model Exchange XML
  - `otel-semconv`: OpenTelemetry Semantic Conventions
  - `baml`: BAML AI prompt definitions
  - `dspy`: DSPy Python optimization program
  - `zenml`: ZenML Python pipeline
  - `lean`: Lean 4 formal Lake package
  - `tla`: TLA+ formal specification
  - `asyncapi`: AsyncAPI 3.0 YAML specification
  - `cloudevents`: CloudEvents 1.0 JSONL stream
  - `cedar`: Cedar authorization schema and policies
  - `devbox`: Devbox hermetic manifest
  - `dagger`: Dagger Python CI module
  - `cell`: Cell environment (`cell.lock`, Devbox, Mise)
  - `domain-python`: Python DDD/CQRS package
  - `domain-typescript`: TypeScript DDD/CQRS package
  - `domain-rust`: Rust DDD/CQRS zero-dep crate
  - `protobuf`: Protocol Buffers `.proto` schemas
- `--created-at <TIMESTAMP>`: Pin timestamp (ISO 8601 UTC) for byte-identical determinism.

---

### `pack`
Manage, validate, sign, and diff Semantic Packs.

```bash
domainforge pack <SUBCOMMAND>
```
- `pack build --source <GLOB> --org <ORG> --domain <DOMAIN> --version <VER> --meaning-version <MVER> --approval <candidate|approved> --out <FILE>`
- `pack validate --pack <FILE> [--strict]`
- `pack sign --in <FILE> --key <PEM_FILE> --out <FILE>`
- `pack diff --old <FILE> --new <FILE> [--format human|json] [--fail-on-breaking]`

---

### `contract`
Resolve and print the ADR-013 Application Contract JSON.

```bash
domainforge contract [OPTIONS] <ENTRY>
```
- `<ENTRY>`: Path to entry `.sea` file.
- `--out <PATH>`: Write output to file.

---

### `envelope`
Resolve and print the Canonical Semantic Envelope document JSON or CEP-0008 envelope.

```bash
domainforge envelope [OPTIONS] <ENTRY>
domainforge envelope --capabilities
```
- `<ENTRY>`: Path to entry `.sea` file.
- `--emit <MODE>`: Emission mode (`representation` [default], `cep`, `both`, or `verification-contract`).
  - `representation`: Prints the raw canonical semantic document ($D$) byte-for-byte.
  - `cep`: Synthesizes a CEP-0008 conformant envelope with boundary, completeness, omission, and extension metadata.
  - `both`: Emits a JSON bundle containing both the CEP envelope and the canonical representation.
  - `verification-contract`: Emits a CEP `work_request` with declared verification obligations; requires a valid model.
- `--capabilities`: Print machine-readable JSON adapter capabilities (`producer`, `version`, `supported_emit_modes`, `contracts`) and exit 0.
- `--pack <PATH>`: Include referenced semantic pack JSON files (repeatable).
- `--registry <PATH>`: Optional path to a namespace registry file.
- `--default-namespace <NAME>`: Explicit default namespace override.
- `--scope <JSON>`: JSON object with caller scope context (e.g. `repo_id`, `run_id`).
- `--inline-threshold-bytes <BYTES>`: Max bytes for inlined representation before CAS carriage (default: 65,536).
- `-o, --out <PATH>`: Write output to file instead of stdout.

**Emission fields:**

- `representation` sets `schema_version` to `domainforge-semantic-envelope/v1`. Its `self_hash` hashes the canonical document without `self_hash`; `semantic_closure_hash` covers the resolved semantic closure. `inputs` records `source_set_hash`, `semantic_pack_set_hash`, `language_schema_version`, and `interpretation_version`. `envelope` carries the resolved declarations, references, and application contract.
- `cep` carries a `boundary_record` naming included sections and known omissions. When $D$ exists, `representations` carries it inline or by content reference. When $D$ cannot be constructed, `omissions` records `omission_type: "representation_unavailable"`, `extensions.domainforge` carries `model_validation_status: "invalid"`, `invalid_declared_checkpoint_hash`, and diagnostics, and `conformance_status` is `conformant` for the CEP envelope itself.
- `both` contains `representation` and `cep_envelope` members when $D$ exists.
- `verification-contract` carries questions, evidence constraints, and obligations keyed to canonical declaration references.

**Exit Codes:**
- `0`: Valid model resolved and emitted successfully.
- `1`: Model validation failed (syntax error, unresolvable import, or policy violation). In `--emit cep` or `--emit both` modes, a schema-valid CEP failure envelope is emitted with explicit omissions and diagnostics.
- `2`: CLI usage or file I/O error (unreadable entry, malformed arguments).

---

### `authority`
Evaluate authority decisions against context facts and emit an audit trace.

```bash
domainforge authority eval --pack <PACK_OR_MODEL> --facts <FACTS_JSON> [--out <TRACE_JSON>]
```

---

### `format` (alias: `fmt`)
Format `.sea` files canonically.

```bash
domainforge format [OPTIONS] <PATH>
```
- `--check`: Check if files are formatted without writing; exits with code 1 if unformatted.
- `--write`: Overwrite files in place with formatted text.

---

### `normalize`
Normalize a policy expression into canonical simplified form.

```bash
domainforge normalize "<EXPRESSION>"
```

---

## 3. Exit Codes

| Exit Code | Meaning |
|---|---|
| `0` | Success. Command executed, validated, or projected with zero errors. |
| `1` | Semantic failure. Parse errors, policy violations, drift errors, or invalid arguments. |
| `2` | I/O or filesystem error (file not found, permission denied, path traversal blocked). |

---

## Source Trail
- `domainforge-core/src/cli/mod.rs` — CLI argument definitions via `clap`
- `domainforge-core/src/bin/domainforge.rs` — Main entry point
