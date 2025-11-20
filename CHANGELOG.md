# Changelog

## Unreleased

- Implemented workspace `NamespaceRegistry` improvements:
  - Added longest-literal-prefix precedence for overlapping glob patterns to choose the most specific namespace instead of erroring on conflict.
  - Added deterministic alphabetical tie-breaker as fallback when literal prefixes are equal.
  - Added `sea registry` CLI with `list` and `resolve` subcommands.
  - Added Python and TypeScript FFI wrappers and tests for `NamespaceRegistry`/`NamespaceBinding`.
  - Added JSON Schema `schemas/sea-registry.schema.json` and tests for schema validation.
  - Added comprehensive unit and integration tests covering discovery, precedence, tie-breaker behavior, CLI listing, and registry resolution.
  - Updated docs and examples (README, `docs/reference/sea-registry.md`).
    - Added optional `--fail-on-ambiguity` CLI flag (default behavior remains alphabetical tie-breaker).
    - Added `namespace_for_with_options` and `resolve_files_with_options` to allow programmatic control of ambiguity behavior.
    - Added unit tests to verify ambiguous-match error handling when `fail_on_ambiguity` is enabled.
     - Added FINOS CALM import/export support to `sea-core`, including:
       - SEA → CALM `export_calm()` and CALM → SEA `import_calm()` conversions
       - Policy (Constraint) export/import serialized as `metadata.sea:expression` (SEA or SBVR formats supported)
       - Association mapping exported as `relationship-type: "association"` and imported into `sea:associations` metadata on source entity
       - Python/TypeScript/WASM bindings: `export_calm`, `import_calm`, `add_policy`, and `add_association` methods
       - Comprehensive unit & integration tests for CALM round-trip & schema validation
