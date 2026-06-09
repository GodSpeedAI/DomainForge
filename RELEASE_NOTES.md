# Release v0.11.0 (2026-06-09)

## What's Changed

### Added

- **Semantic Pack System**: Canonical semantic pack support with schema definitions, builder APIs, validation, diffing, pack-set resolution, canonical JSON, review records, and signing helpers.
- **Authority Evaluation**: Authority policy compilation and evaluation with environments, fact resolvers, resolver traces, policy transforms, and conformance coverage.
- **CLI Pack Workflows**: `sea pack` and authority-oriented CLI support for building, validating, reviewing, signing, and checking semantic packs.
- **Cross-Language Bindings**: Semantic pack and authority APIs exposed through Python, TypeScript napi, and WASM bindings.
- **Test Fixtures And Coverage**: ACME procurement fixtures plus Rust, Python, TypeScript, and WASM coverage for semantic packs and authority behavior.
- **Docs And Guardrails**: Security model, generated-artifact policy, semantic pack docs, diagnostics, release workflow linting, and enterprise verification gates.

### Changed

- **README And Binding Guides**: Clearer domain modeling and binding usage across the main README and Python, TypeScript, and WASM docs.
- **Protobuf/CALM/RDF**: Improved projection validity and round-trip fidelity.
- **Parser And AST**: Tighter namespace semantics, resource limits, AST conversion/schema behavior, and printer output.
- **Release Packaging**: Hardened npm/WASM packaging, release artifact checks, and published-release workflows.

### Fixed

- **Security**: Hardened path handling and removed a committed private-key fixture.
- **Bindings**: Regenerated Python stubs and TypeScript declarations to match exported native APIs.
- **CI Stability**: Stabilized Rust, Python, TypeScript, macOS, and release jobs.
- **Release Prechecks**: Fixed false version mismatches when TOML files use CRLF line endings.
- **Parser/Projection Issues**: Fixed parser, decimal quantity, Protobuf validity, CALM/RDF, and deferred CI findings.

## Contributors

- @SPRIME01
- @SPrime01
- @coderabbitai[bot]
- @copilot-swe-agent[bot]
- @dependabot[bot]

## Links

- [Full Changelog](./CHANGELOG.md)
- [Documentation](./docs/)
- [Compare with previous version](https://github.com/GodSpeedAI/DomainForge/compare/v0.10.0...v0.11.0)
