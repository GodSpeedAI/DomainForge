# SEA CLI Reference

The SEA CLI (`sea`) is the primary tool for interacting with the SEA DSL.

## Installation

```bash
cargo install --path sea-core --features cli
```

## Global Options

- `--verbose`: Enable verbose output
- `--quiet`: Suppress output
- `--color <COLOR>`: Control color output (auto, always, never)

## Commands

### `validate`

Validate SEA files.

```bash
sea validate [OPTIONS] <TARGET>
```

Options:

- `--format <FORMAT>`: Output format (human, json, lsp) [default: human]
- `--no-color`: Disable colored output

**Note:** The following options are planned but not yet implemented:

- `--json`: Output results in JSON format (alternative to `--format json`)
- `--exit-code`: Set exit code based on validation result
- `--fix`: Attempt to auto-fix issues
- `--config <FILE>`: Custom validation configuration
- `--strict`: Enable strict validation mode (fail on warnings)
- `--round-trip <FORMAT>`: Validate round-trip integrity

### `import`

Import from other formats.

```bash
sea import [OPTIONS] --format <FORMAT> <FILE> [OUTPUT]
```

Options:

- `--format <FORMAT>`: Source format (sbvr, kg) [required]
- `--validate`: Validate after importing (planned, not yet implemented)
- `--merge <FILE>`: Merge with existing SEA file (planned, not yet implemented)
- `--namespace <NS>`: Override default namespace for KG imports (planned, not yet implemented)

Formats:

- `sbvr`: SBVR XMI
- `kg`: Knowledge Graph (Turtle, RDF/XML - auto-detected)

### `project`

Project/Export to other formats.

```bash
sea project [OPTIONS] --format <FORMAT> <INPUT> <OUTPUT>
```

Options:

- `--format <FORMAT>`: Target format (calm, kg) [required]
- `--output-dir <DIR>`: Output directory for multi-file exports (planned, not yet implemented)
- `--pretty`: Enable pretty-printing for output (planned, not yet implemented)
- `--validate`: Validate before exporting (planned, not yet implemented)

Formats:

- `calm`: CALM JSON
- `kg`: Knowledge Graph (Turtle or RDF/XML based on output file extension)

### `format`

Format SEA files (not yet fully implemented).

```bash
sea format [OPTIONS] <FILES>...
```

Options:

- `--write`: Write formatted output back to files (planned, not yet implemented)
- `--config <FILE>`: Custom formatting configuration (planned, not yet implemented)
- `--line-length <N>`: Maximum line length (planned, not yet implemented)
- `--indent <N>`: Indentation size in spaces (planned, not yet implemented)
- `--check`: Only check if files are formatted (planned, not yet implemented)

**Note:** Currently returns "Formatting not yet implemented" error. Only syntax validation is supported.

### `test`

Run tests (not yet fully implemented).

```bash
sea test [OPTIONS] [PATTERN]
```

Options:

- `--pattern <GLOB>`: Glob pattern to select test files (planned, not yet implemented)
- `--filter <REGEX>`: Run only tests matching regex (planned, not yet implemented)
- `--verbose`: Enable verbose output (planned, not yet implemented)
- `--fail-fast`: Stop on first failure (planned, not yet implemented)
- `--json`: Emit machine-readable JSON output (planned, not yet implemented)

**Note:** The test runner is currently a placeholder that accepts a pattern argument.

### `validate-kg`

Validate Knowledge Graph files against SHACL shapes.

```bash
sea validate-kg [OPTIONS] <FILE>
```

Options:

- `--shapes <FILE>`: SHACL shapes file (planned, not yet implemented; default: embedded `sea-shapes.ttl`)
- `--format <FORMAT>`: Input format (turtle, rdf-xml, n-triples) (planned, not yet implemented; currently auto-detected)
- `--json`: Output validation report in JSON (planned, not yet implemented)
- `--severity <LEVEL>`: Minimum severity to report (info, warning, violation) (planned, not yet implemented)

**Note:** Format is currently auto-detected based on file extension and content. Additional options are planned but not yet implemented.
