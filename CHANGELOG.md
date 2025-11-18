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
