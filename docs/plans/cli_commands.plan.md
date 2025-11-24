# CLI Commands for Projection/Import/Validation/Testing

This plan outlines the implementation of a comprehensive CLI for the SEA DSL, as specified in Task 7 of `docs/work/plans/polish-plan.md`. It involves refactoring the existing `sea` binary into a modular structure and adding new commands for exporting, formatting, and testing.

> [!IMPORTANT]
> This refactor will move existing CLI logic from `src/bin/sea.rs` to `src/cli/`.
> The `Validate` command arguments will be updated to match the plan (adding `--strict`, `--json`, `--exit-code`, `--round-trip`).
> New dependencies might be needed if not already present (e.g., `oxigraph` is already optional, but we need to ensure it's used correctly).

## Overview

The refactored CLI will support the following commands:

- **validate**: Validate SEA DSL files (enhanced with strict mode, JSON output, round-trip validation)
- **import**: Import from external formats (CALM, KG/RDF, SBVR) into SEA DSL
- **project**: Export/project SEA DSL to external formats (CALM, KG/RDF, SBVR)
- **format**: Format and pretty-print SEA DSL files
- **test**: Run policy validation tests against SEA files
- **validate-kg**: Validate knowledge graphs using SHACL shapes

## Proposed Changes

### Code Structure

#### [NEW] `src/cli/mod.rs`

Module organization and public API:

- Define root `Cli` struct with `clap` derive
- Define `Commands` enum with all subcommands
- Export submodule handlers: `project`, `import`, `validate`, `format`, `test`, `validate_kg`
- Provide `run()` function as main entry point for CLI execution
- Implement common utilities: error handling, output formatting, file I/O helpers

#### [NEW] `src/cli/project.rs`

Export/projection command implementation:

- Command: `sea project [OPTIONS] <INPUT> [OUTPUT]`
- Flags:
  - `--format <FORMAT>`: Target format (calm, kg, sbvr) [required]
  - `--output-dir <DIR>`: Output directory for multi-file exports
  - `--pretty`: Enable pretty-printing for output
  - `--validate`: Validate before exporting
- Implementation:
  - Parse SEA DSL input file
  - Transform to target format using appropriate projection logic
  - Write output to file or stdout
  - Handle format-specific options (e.g., namespace for KG, templates for SBVR)

#### [NEW] `src/cli/import.rs`

Import command implementation (refactored from existing logic):

- Command: `sea import [OPTIONS] <INPUT> [OUTPUT]`
- Flags:
  - `--format <FORMAT>`: Source format (calm, kg, sbvr) [required]
  - `--validate`: Validate after importing
  - `--merge <FILE>`: Merge with existing SEA file
  - `--namespace <NS>`: Override default namespace (for KG imports)
- Implementation:
  - Read input file in specified format
  - Parse and transform to SEA DSL AST
  - Optionally merge with existing SEA model
  - Write SEA output or validate
  - Preserve metadata and annotations where possible

#### [NEW] `src/cli/validate.rs`

Enhanced validation command (refactored from existing logic):

- Command: `sea validate [OPTIONS] <FILES>...`
- Flags:
  - `--strict`: Enable strict validation mode (fail on warnings)
  - `--json`: Output results in JSON format
  - `--exit-code`: Set exit code based on validation result
  - `--round-trip <FORMAT>`: Validate round-trip integrity (export then import)
  - `--fix`: Attempt to auto-fix issues (if possible)
  - `--config <FILE>`: Custom validation configuration
- Implementation:
  - Parse input files
  - Run semantic validation (type checking, constraint validation)
  - Check for undefined references, circular dependencies
  - If `--round-trip`: export to format, re-import, compare ASTs
  - Format output (human-readable markdown or JSON)
  - Set appropriate exit codes

#### [NEW] `src/cli/format.rs`

Format/pretty-print command:

- Command: `sea format [OPTIONS] <FILES>...`
- Flags:
  - `--check`: Only check if files are formatted (exit code only)
  - `--write`: Write formatted output back to files (in-place)
  - `--config <FILE>`: Custom formatting configuration
  - `--line-length <N>`: Maximum line length
  - `--indent <N>`: Indentation size (spaces)
- Implementation:
  - Parse SEA DSL files
  - Apply formatting rules (consistent indentation, spacing, ordering)
  - Either check, display diff, or write back
  - Use existing pretty-printer or implement canonical formatter
  - Support `.sea-format` config file

#### [NEW] `src/cli/test.rs`

Policy testing command:

- Command: `sea test [OPTIONS] [PATTERN]`
- Flags:
  - `--pattern <GLOB>`: File pattern for test discovery (default: `**/*.sea.test`)
  - `--filter <REGEX>`: Filter tests by name
  - `--verbose`: Show detailed test output
  - `--fail-fast`: Stop on first failure
  - `--json`: Output results in JSON format
- Implementation:
  - Discover test files using glob pattern
  - Parse test specifications (expected outcomes, constraints)
  - Run validation against test cases
  - Report results: passed/failed/skipped counts
  - Support test metadata: tags, descriptions, expectations

#### [NEW] `src/cli/validate_kg.rs`

SHACL knowledge graph validation:

- Command: `sea validate-kg [OPTIONS] <KG_FILE>`
- Flags:
  - `--shapes <FILE>`: SHACL shapes file (default: embedded `sea-shapes.ttl`)
  - `--format <FORMAT>`: Input format (turtle, rdf-xml, n-triples)
  - `--json`: Output validation report in JSON
  - `--severity <LEVEL>`: Minimum severity to report (info, warning, violation)
- Implementation:
  - Load knowledge graph using `oxigraph`
  - Load SHACL shapes (from file or embedded resource)
  - Run SHACL validation
  - Parse and format validation report
  - Map violations back to source locations if possible

#### [MODIFY] `src/bin/sea.rs`

Simplified main binary:

```rust
use clap::Parser;
use sea_core::cli::{Cli, run};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run(cli)
}
```

- Remove all command logic (delegated to `src/cli/`)
- Keep only CLI parsing and execution entry point
- Handle global flags: `--verbose`, `--quiet`, `--color`
- Set up logging/tracing based on verbosity

#### [MODIFY] `Cargo.toml`

Dependencies and features:

- Ensure `clap = { version = "4", features = ["derive", "cargo", "env"] }`
- Enable `oxirs-shacl` for SHACL validation: `oxirs-shacl = { version = "", optional = true }`
- Add feature flag: `shacl = ["oxirs-shacl"]`
- Consider adding:
  - `serde_json` for JSON output
  - `colored` or `termcolor` for colored terminal output
  - `glob` for test pattern matching
  - `similar` or `diff` for formatting diffs

### Documentation

#### [NEW] `docs/reference/cli.md`

Comprehensive CLI reference documentation:

- Overview and installation
- Global options and environment variables
- Detailed documentation for each command with examples
- Common workflows and recipes
- Troubleshooting and FAQ
- Exit codes reference

#### [MODIFY] `README.md`

Add CLI usage section:

- Quick start examples for common commands
- Link to full CLI reference documentation
- Installation instructions

#### [NEW] `examples/cli/`

Example files demonstrating CLI usage:

- `validate_example.sh`: Validation examples
- `import_export_workflow.sh`: Round-trip workflow
- `test_runner_example.sh`: Testing example
- Sample input files for each format

#### [MODIFY] `justfile`

Add recipes for CLI testing:

```makefile
# Test all CLI commands
cli-test:
    cargo test -p sea-core --test cli_tests --features cli

# Run CLI validation examples
cli-validate:
    cd sea-core/examples/cli && ./validate_example.sh

# Run CLI import/export workflow
cli-workflow:
    cd sea-core/examples/cli && ./import_export_workflow.sh
```

## Verification Plan

### Automated Tests

#### Unit Tests

Create `tests/cli_tests.rs` to test each subcommand:

```rust
#[test]
fn test_validate_basic() {
    // Test basic validation
}

#[test]
fn test_validate_strict_mode() {
    // Test --strict flag
}

#[test]
fn test_validate_json_output() {
    // Test --json flag
}

#[test]
fn test_round_trip_calm() {
    // Test round-trip with CALM format
}

#[test]
fn test_import_export_kg() {
    // Test KG import/export
}

#[test]
fn test_format_check() {
    // Test format --check
}

#[test]
fn test_policy_tests() {
    // Test policy test runner
}

#[test]
fn test_validate_kg_shacl() {
    // Test SHACL validation
}
```

Run with: `cargo test --test cli_tests`

#### Integration Tests

Round-trip validation tests:

- `sea validate --round-trip calm input.sea`
- `sea validate --round-trip kg input.sea`
- `sea validate --round-trip sbvr input.sea`
- Verify that export -> import preserves semantics
- Compare AST structures before and after round-trip

#### CLI Exit Code Tests

Verify exit codes for different scenarios:

- Success (0)
- Validation errors (1)
- File not found (2)
- Invalid arguments (3)
- Round-trip mismatch (4)

### Manual Verification

#### Help and Usage

```bash
# Verify help messages
cargo run --bin sea -- --help
cargo run --bin sea -- validate --help
cargo run --bin sea -- import --help
cargo run --bin sea -- project --help
cargo run --bin sea -- format --help
cargo run --bin sea -- test --help
cargo run --bin sea -- validate-kg --help
```

#### Command Testing

```bash
# Test validate command
cargo run --bin sea -- validate examples/basic.sea
cargo run --bin sea -- validate --strict examples/basic.sea
cargo run --bin sea -- validate --json examples/basic.sea

# Test import command
cargo run --bin sea -- import --format calm examples/basic.calm

# Test project command
cargo run --bin sea -- project --format calm examples/basic.sea output.calm
cargo run --bin sea -- project --format kg examples/basic.sea output.ttl

# Test format command
cargo run --bin sea -- format --check examples/basic.sea
cargo run --bin sea -- format examples/basic.sea

# Test test command
cargo run --bin sea -- test examples/**/*.sea.test

# Test validate-kg command
cargo run --bin sea -- validate-kg examples/basic.ttl
```

#### Regression Testing

```bash
# Ensure existing functionality works
just all-tests
just ai-validate
```

### Expected Outcomes

#### Functionality

- All commands execute without errors for valid inputs
- Error handling is robust and provides helpful messages
- Exit codes are consistent and documented
- JSON output is well-formed and useful
- Round-trip validation correctly identifies semantic differences

#### Code Quality

- CLI code is modular and maintainable
- Each command has clear separation of concerns
- Common functionality is abstracted into utilities
- Error handling is consistent across commands
- Code follows Rust idioms and best practices

#### User Experience

- Help messages are clear and comprehensive
- Error messages are actionable and user-friendly
- CLI is intuitive and follows common conventions
- Output formatting is consistent (colors, indentation, alignment)
- Progress indicators for long-running operations

#### Documentation

- CLI reference is complete and accurate
- Examples cover common use cases
- Troubleshooting guide addresses likely issues
- README provides quick-start instructions
- Code comments explain non-obvious logic

#### Testing

- All automated tests pass
- Manual verification steps complete successfully
- Edge cases are handled gracefully
- Performance is acceptable for typical inputs
- No regressions in existing functionality

## Implementation Checklist

### Phase 1: Setup and Structure

- [ ] Create `src/cli/mod.rs` with basic structure
- [ ] Define `Cli` and `Commands` structs
- [ ] Update `Cargo.toml` with required dependencies
- [ ] Create placeholder modules for each command

### Phase 2: Core Commands

- [ ] Implement `src/cli/validate.rs` (refactor existing logic)
- [ ] Implement `src/cli/import.rs` (refactor existing logic)
- [ ] Implement `src/cli/project.rs` (new functionality)
- [ ] Update `src/bin/sea.rs` to use new CLI structure

### Phase 3: Additional Commands

- [ ] Implement `src/cli/format.rs`
- [ ] Implement `src/cli/test.rs`
- [ ] Implement `src/cli/validate_kg.rs`

### Phase 4: Testing

- [ ] Create `tests/cli_tests.rs`
- [ ] Add unit tests for each command
- [ ] Add integration tests for round-trip validation
- [ ] Test exit codes and error handling

### Phase 5: Documentation

- [ ] Create `docs/reference/cli.md`
- [ ] Update `README.md` with CLI usage
- [ ] Create example scripts in `examples/cli/`
- [ ] Update `justfile` with CLI test recipes

### Phase 6: Polish

- [ ] Improve error messages
- [ ] Add colored output support
- [ ] Add progress indicators
- [ ] Performance optimization
- [ ] Final testing and validation

## Notes and Considerations

### Error Handling

- Use `anyhow::Result` for error propagation
- Provide context with `.context()` for all errors
- Map errors to appropriate exit codes
- Include file paths and line numbers in error messages
- Suggest fixes when possible (e.g., "Did you mean...?")

### Output Formatting

- Use `termcolor` or `colored` for colored output
- Respect `NO_COLOR` environment variable
- Support `--color` flag (auto, always, never)
- Align output in tables for readability
- Use Unicode box-drawing characters when supported

### Performance

- Stream large files instead of loading entirely into memory
- Use parallel processing for multiple files when possible
- Show progress bars for long-running operations
- Cache intermediate results when appropriate
- Optimize hot paths in validation and parsing

### Compatibility

- Follow standard CLI conventions (POSIX-style options)
- Support both short (`-f`) and long (`--format`) options
- Allow `--` to separate options from positional arguments
- Read from stdin when filename is `-`
- Write to stdout when output is omitted

### Enhancements

- Interactive mode for complex workflows
- Watch mode for continuous validation
- LSP server command for IDE integration
- Diff command for comparing SEA files
- Merge command for combining SEA models
- Shell completion generation
