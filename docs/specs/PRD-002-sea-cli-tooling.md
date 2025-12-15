# PRD-002: SEA CLI Tooling

**Product:** SEA Command-Line Interface  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Goals

- Provide a unified `sea` command for all DSL operations.
- Enable local development workflows: validate, format, project.
- Support CI/CD integration with structured output.
- Offer extensibility for future commands.

---

## 2. Non-Goals

- GUI or TUI interfaces (out of scope for v1).
- Package management or registry operations.
- Remote execution or cloud deployment.

---

## 3. User Stories

### 3.1 As a Domain Modeler

> I want to validate my `.sea` files before committing so that I catch errors early.

**Acceptance Criteria:**

- `sea validate path/to/model.sea` reports errors with line numbers
- Exit code is non-zero on validation failure

### 3.2 As a Developer

> I want to project my SEA models to JSON/DDD manifests for consumption by downstream systems.

**Acceptance Criteria:**

- `sea project --format json model.sea` outputs valid JSON
- Multiple format options available (json, ddd, yaml)

### 3.3 As a CI Pipeline

> I want machine-readable validation output for automated quality gates.

**Acceptance Criteria:**

- `sea validate --format json` outputs structured error array
- Consistent exit codes for scripting

---

## 4. Commands

### 4.1 `sea validate`

Validate SEA source files.

```bash
sea validate [OPTIONS] <FILES>...

Options:
  --format <FORMAT>    Output format [default: human] [possible: human, json, lsp]
  --strict             Treat warnings as errors
  --profile <PROFILE>  Enforce a specific DSL profile
```

### 4.2 `sea project`

Export semantic graph to target formats.

```bash
sea project [OPTIONS] <FILE>

Options:
  --format <FORMAT>    Target format [default: json] [possible: json, ddd, yaml]
  --output <PATH>      Output file path (default: stdout)
```

### 4.3 `sea format`

Format SEA source files.

```bash
sea format [OPTIONS] <FILES>...

Options:
  --check              Check formatting without modifying files
  --write              Write formatted output back to files
```

### 4.4 `sea import`

Import models from external formats.

```bash
sea import [OPTIONS] <FILE>

Options:
  --from <FORMAT>      Source format [possible: turtle, rdfxml, sbvr]
  --output <PATH>      Output .sea file path
```

### 4.5 `sea test`

Run SEA test assertions.

```bash
sea test [OPTIONS] <FILES>...

Options:
  --filter <PATTERN>   Run only tests matching pattern
```

### 4.6 `sea validate-kg`

Validate Knowledge Graph files (SHACL).

```bash
sea validate-kg [OPTIONS] <FILE>

Options:
  --shapes <PATH>      SHACL shapes file
```

---

## 5. Global Options

| Option             | Description                       |
| ------------------ | --------------------------------- |
| `--verbose`        | Increase output verbosity         |
| `--quiet`          | Suppress non-error output         |
| `--color <CHOICE>` | Color output: auto, always, never |

---

## 6. Exit Codes

| Code | Meaning                     |
| ---- | --------------------------- |
| 0    | Success                     |
| 1    | Validation/processing error |
| 2    | Invalid arguments           |

---

## 7. Implementation Status

| Command       | Status         |
| ------------- | -------------- |
| `validate`    | ✅ Implemented |
| `project`     | ✅ Implemented |
| `format`      | ✅ Implemented |
| `import`      | ✅ Implemented |
| `test`        | ✅ Implemented |
| `validate-kg` | ✅ Implemented |

---

## Related Documents

- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
- [ADR-006: Error Handling Strategy](./ADR-006-error-handling-strategy.md)
