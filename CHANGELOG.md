# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2025-12-18

### Added

- **Expression Normalization Engine**: Canonical normalization for policy expressions

  - Transforms expressions to canonical form for equivalence checking
  - Implements boolean algebra simplifications (identity, idempotence, double negation)
  - Commutative operand sorting for deterministic output
  - Stable hashing via xxhash-rust for caching and comparison

- **`sea normalize` CLI Command**: New command for expression normalization

  - Normalize policy expressions: `sea normalize "a AND b"`
  - Check equivalence: `sea normalize "a AND b" --check-equiv "b AND a"`
  - JSON output mode: `--json` for programmatic use

- **Expression Bindings**: Full Expression API across all language bindings

  - **Python**: `Expression`, `NormalizedExpression`, `BinaryOp`, `UnaryOp`, `Quantifier`, `AggregateFunction`, `WindowSpec`
  - **TypeScript**: Same APIs via napi-rs bindings
  - **WASM**: Same APIs for browser environments
  - Factory methods for all expression types (literals, variables, binary, unary, quantifiers, aggregations)
  - `normalize()`, `is_equivalent()`, `stable_hash()` methods

- **ADR-001**: Architectural Decision Record for canonical normalizer design

### Changed

- **License**: Standardized to Apache-2.0 across all package configurations
- **Documentation**: Updated CLI reference and language binding docs with expression APIs

### Fixed

- **Parser Trailing Input Detection**: `parse_expression_from_str` now correctly rejects malformed expressions with unconsumed trailing input (e.g., `"NOT (a"` no longer parses as just `NOT`)
- **SBVR Test Expression**: Fixed invalid quantifier syntax in test data
- **CI Network Failures**: Added `RUSTUP_MAX_RETRIES: 10` to all Rust toolchain installations to handle transient network failures on macOS runners
- **Dependency Review**: Added BSL-1.0 to allowed licenses for xxhash-rust dependency
- **Rust Toolchain Action**: Added explicit `toolchain: stable` input to all `dtolnay/rust-toolchain@stable` action invocations

## [0.5.0] - 2025-12-17

### Changed

- **macOS CI Runners**: Updated from macos-13 to macos-14 (macos-13 retired Dec 2024)
- **Release Workflow**: Enhanced version syncing across pyproject.toml and package.json

### Fixed

- **CHANGELOG Update**: Fixed printf-based YAML generation to avoid comment parsing issues
- **PR Token**: Use dedicated token for creating pull requests

## [0.4.0] - 2025-12-15

### Added

- **ARM Platform Support**: Native binaries for Apple Silicon and ARM Linux

  - `aarch64-apple-darwin` (Apple Silicon Macs)
  - `aarch64-unknown-linux-gnu` (ARM Linux servers)
  - Cross-compilation using `cross` and `zig` for CI builds

- **WASM npm Package**: `@domainforge/sea-core-wasm` now published to npm

  - Enables browser-based SEA DSL parsing and formatting
  - Published alongside existing napi bindings

- **Release Automation Workflow**: New `prepare-release.yml` for streamlined releases

  - Automatic version bumping (major/minor/patch)
  - CHANGELOG.md template generation
  - PR creation with checklist

- **VS Code Multi-Root Workspace**: Configuration for monorepo development
  - Unified workspace for domainforge, domainforge-lsp, and domainforge-vsc-extension

### Changed

- **CI/CD Improvements**:

  - Harmonized WASM bundle size threshold to 2MB across CI and release workflows
  - Better error handling in crates.io publishing (explicit "already published" detection)
  - GitHub Release creation explicitly required for publishing workflows

- **Documentation**:
  - Updated workflows README with comprehensive release process guide
  - Added notes about `release: published` trigger requirement

### Fixed

- **Clippy Errors**: Resolved `field_reassign_with_default` and `collapsible_if` warnings
- **Test Expectations**: Updated `test_format_check` to expect success (format now implemented)
- **CodeRabbit Review Issues**: Addressed all 4 issues from automated review
  - Fixed README release instructions
  - Removed unsafe `--allow-dirty` from crates publish
  - Fixed fragile sed patterns in CHANGELOG update
  - Preserved backward-compatible job IDs

## [0.3.0] - 2025-12-14

### Added

- **SEA Formatter (`sea fmt`)**: New CLI command to format SEA source code

  - Supports 14 declaration types (Entity, Resource, Flow, etc.)
  - Indentation and spacing standardization
  - Comment preservation (file headers)
  - Sorting of imports
  - `--check` flag for CI validation

- **Formatter Bindings**: Programmatic access to the formatter

  - **Python**: `sea_dsl.format_source()`, `sea_dsl.check_format()`
  - **TypeScript**: `formatSource()`, `checkFormat()` from `@domainforge/sea`
  - **WASM**: `formatSource()`, `checkFormat()` for browser environments

- **Documentation**:
  - Comprehensive CLI reference for `fmt`
  - Updated language binding APIs with formatter documentation
  - Usage examples for all platforms

### Changed

- Updated dependencies to align with new features
- Enhanced documentation for all supported bindings

## [0.2.1] - 2025-12-14

### Added

- **Protobuf Projection Engine**: Full Protocol Buffers export support for SEA models

  - Entity and Resource projection to Protobuf messages
  - Type mapping from SEA types to proto scalar types
  - Deterministic field numbering for schema stability
  - CLI command: `sea project --format protobuf model.sea output.proto`

- **gRPC Service Generation**: Generate gRPC services from Flow patterns (Phase 2)

  - Flows automatically become RPC methods
  - Streaming support: `@streaming`, `@client_streaming`, `@bidirectional` attributes
  - Service naming convention: `{DestinationEntity}Service`

- **Well-Known Types Support**: Automatic mapping to Google's WKT (Phase 1)

  - `DateTime` → `google.protobuf.Timestamp`
  - `Duration` → `google.protobuf.Duration`
  - `Any` → `google.protobuf.Any`
  - Auto-import generation for used WKT types

- **Custom Proto Options**: Per-file and per-message options (Phase 3)

  - `--option java_package=com.example` CLI flag
  - Support for `java_multiple_files`, `go_package`, `csharp_namespace`, etc.
  - Custom option values in generated output

- **Schema Compatibility Checking**: Prevent breaking changes

  - Three modes: `Additive`, `Backward`, `Breaking`
  - Validates field additions, removals, renames, and type changes
  - `--compatibility` and `--against` CLI flags

- **Buf.build Integration**: Optional CLI integration (Phase 4)

  - `--buf-lint` for proto linting
  - `--buf-breaking` for breaking change detection
  - Graceful degradation when buf CLI not installed
  - Auto-generation of `buf.yaml` and `buf.gen.yaml`

- **Multi-File Output**: Separate `.proto` files per namespace (Phase 5)
  - `--multi-file --output-dir ./proto` CLI flags
  - Fully qualified imports for cross-namespace references
  - Directory structure mirrors package hierarchy

### Documentation

- New how-to guide: `docs/how-tos/export-to-protobuf.md`
- New API reference: `docs/reference/protobuf-api.md`
- Updated CLI reference with protobuf options
- Updated SDS-001 status to Implemented

## [0.2.0] - 2025-12-07

### Added

- **Bun Runtime Support**: Adopted Bun as the primary JavaScript runtime and package manager for TypeScript tooling, with Node.js as fallback
- **FINOS CALM Integration**: Full import/export support for architecture-as-code
  - `export_calm()` and `import_calm()` methods across all bindings
  - Policy (Constraint) export/import with SEA/SBVR expression formats
  - Association mapping for relationship types
- **Namespace Registry Improvements**:
  - `sea registry` CLI with `list` and `resolve` subcommands
  - Longest-literal-prefix precedence for overlapping glob patterns
  - Deterministic alphabetical tie-breaker for equal prefixes
  - `--fail-on-ambiguity` flag for strict resolution
  - Python, TypeScript, and WASM FFI bindings for registry operations
- **Instance Primitives**: New `Instance` class for entity type instances with named fields
- **Role and Relation Primitives**: Support for roles and relationships between entities
- **Enhanced Graph Methods**: `add_role()`, `add_relation()`, `add_policy()`, `add_association()`, `evaluate_policy()`, `set_evaluation_mode()`
- **Three-Valued Logic**: Policy evaluation with true/false/null semantics

### Changed

- **API Simplification**: Unified constructor patterns across Python/TypeScript/WASM
  - Standard constructors instead of static factory methods
  - Consistent `id()`, `name()`, `namespace()` method signatures
- **Documentation Updates**: Comprehensive README updates for all bindings
  - Accurate API documentation matching actual implementations
  - Updated test counts (544+ Rust tests across 69 test files)
  - Fixed code examples with correct method signatures

### Fixed

- Module import/export correctness: wildcard imports require aliases, trailing commas allowed
- Parser keyword ambiguity resolved (fixed `Resource ... units` parsing)
- Cross-language binding stability: `ReferenceType` serialization fixes
- Parser entry options handle missing registry gracefully
- CI/CD optimizations with smarter caching strategies

## [0.1.0] - 2024-11-15

### Added

- Initial release with core SEA DSL primitives
- Entity, Resource, Flow, ResourceInstance primitives
- Graph-based domain modeling with validation
- Parser for SEA DSL syntax
- Python bindings via PyO3
- TypeScript bindings via napi-rs
- WASM bindings via wasm-bindgen
- CLI tool for validation and projection
- Knowledge Graph (RDF/Turtle) export
