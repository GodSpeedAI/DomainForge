# Security Model

This document describes the DomainForge security model, focusing on how untrusted input is handled.

## Threat Model: Untrusted .sea Input

DomainForge processes `.sea` DSL files that may originate from untrusted sources. The primary threat vectors are:

- **Malformed input** designed to trigger panics or resource exhaustion in the parser
- **Path traversal** via import paths or namespace declarations
- **Archive extraction** exploits when unpacking semantic packs
- **Injection** of executable content through DSL constructs

The parser is the primary attack surface. It must handle arbitrary input without panicking, exhausting memory, or executing external processes.

## Parser Limits

The Pest-based parser processes input through a PEG grammar. Known behaviors:

- **Deeply nested expressions**: Excessive nesting (hundreds of levels) may hit recursion limits. The parser returns a `ParseError` rather than panicking.
- **Large input**: Files with hundreds of thousands of declarations are accepted. Memory usage scales linearly with input size.
- **Null bytes**: Input containing null bytes is handled gracefully and produces a parse error.
- **Long strings**: Individual string literals up to at least 1 MB are processed without panicking.

Test coverage for these limits lives in `sea-core/tests/parser_resource_limits_tests.rs`.

## Namespace and Import Path Restrictions

The module resolver enforces the following restrictions:

- **No path separators** in namespace declarations. Forward slashes (`/`) and backslashes (`\`) are rejected.
- **No path traversal** sequences (`..`) in namespace or import paths.
- **No absolute paths** in namespace declarations.
- **Circular imports** are detected and reported as `E505: CircularDependency` with the full cycle path.

These checks are implemented in `sea_core::projection::protobuf::validate_proto_package_namespace` and `sea_core::module::resolver::ModuleResolver`.

## Safe Archive Extraction

When extracting semantic packs (`.sea-pack` archives), the following protections apply:

- **Path traversal prevention**: Archive entries with paths containing `..` or absolute paths are rejected.
- **Size limits**: Archive entries exceeding configured size thresholds are rejected before extraction.
- **Symlink handling**: Symlinks within archives are not followed during extraction.

## No Process Execution from Model Content

DomainForge does **not** execute external processes based on the content of `.sea` files. Specifically:

- DSL declarations (Entity, Resource, Flow, Policy, etc.) are data declarations only
- Annotations and metadata are parsed as structured data, never evaluated as code
- The `export` keyword controls visibility within the DSL namespace system
- No `eval()`, `system()`, or subprocess invocation exists in the parser or graph construction pipeline
- CLI commands operate on files and produce structured output (JSON, Turtle, Protobuf) without interpreting DSL content as executable instructions

## Cargo Supply Chain Security

Dependency auditing is enforced through:

- **cargo-audit**: Scans for known vulnerabilities in dependencies
- **cargo-deny**: Enforces license allowlists, bans duplicate versions, and denies wildcard dependencies
- **Configuration**: `deny.toml` at the workspace root defines the policy

Run `just audit` to check dependencies locally, or `just enterprise-verify` for the full release verification pipeline.

## Release Workflow Linting

Three Python scripts validate CI/CD workflow files for security issues:

| Script | Checks |
|--------|--------|
| `scripts/lint_release_workflows.py` | Unfrozen npm installs, curl without checksums, continue-on-error on publish |
| `scripts/lint_release_security.py` | curl pipe to shell, publish failure masking, token exposure during builds |
| `scripts/lint_workflow_gates.py` | Artifact upload jobs gated by create-release, dependabot empty-check handling |
