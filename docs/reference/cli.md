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

- `--format <FORMAT>`: Output format (human, json, lsp)
- `--strict`: Enable strict validation
- `--no-source`: Do not show source code in output
- `--round-trip <FORMAT>`: Verify round-trip conversion

### `import`

Import from other formats.

```bash
sea import --format <FORMAT> <FILE>
```

Formats:

- `sbvr`: SBVR XMI
- `kg`: Knowledge Graph (Turtle, RDF/XML)

### `project`

Project/Export to other formats.

```bash
sea project --format <FORMAT> <INPUT> <OUTPUT>
```

Formats:

- `calm`: CALM JSON
- `kg`: Knowledge Graph (Turtle, RDF/XML)

### `format`

Format SEA files.

```bash
sea format [OPTIONS] <FILE>
```

Options:

- `--check`: Check if file is formatted

### `test`

Run tests.

```bash
sea test <PATTERN>
```

### `validate-kg`

Validate Knowledge Graph files against SHACL shapes.

```bash
sea validate-kg <FILE>
```
