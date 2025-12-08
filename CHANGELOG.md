# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
