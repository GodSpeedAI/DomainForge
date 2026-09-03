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
Resolve and print the Canonical Semantic Envelope document JSON.

```bash
domainforge envelope [OPTIONS] <ENTRY>
```
- `<ENTRY>`: Path to entry `.sea` file.
- `--pack <PATH>`: Include referenced semantic pack JSON files.

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
