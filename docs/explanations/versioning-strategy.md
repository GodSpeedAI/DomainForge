# Versioning Strategy

DomainForge follows [Semantic Versioning 2.0.0](https://semver.org/).

## Version Format: `MAJOR.MINOR.PATCH`

- **MAJOR**: Incompatible API changes or breaking grammar changes.
- **MINOR**: Functionality added in a backward-compatible manner.
- **PATCH**: Backward-compatible bug fixes.

## What Constitutes a Breaking Change?

### 1. Grammar Changes

- **Breaking**: Removing a keyword, changing the syntax of an existing primitive, making an optional field mandatory.
- **Non-Breaking**: Adding a new primitive, adding an optional field, relaxing a constraint.

### 2. Core API (Rust)

- **Breaking**: Changing public struct fields, altering function signatures in `sea-core`.
- **Non-Breaking**: Adding new methods, internal performance improvements.

### 3. Bindings (Python/TS)

- **Breaking**: Renaming a class or method exposed to the host language.
- **Non-Breaking**: Exposing previously internal methods.

## Release Coordination

Because `sea-core` and the bindings are in the same repository (monorepo style), they share a version number.

- When `sea-core` bumps to `0.5.0`, the Python package `domainforge` and NPM package `domainforge` also bump to `0.5.0`.

## Deprecation Policy

Before removing a feature:

1. Mark it as deprecated in the documentation.
2. Add a warning log in the CLI/Parser when the feature is used.
3. Maintain support for at least one MINOR release cycle.

## See Also

- [Releasing Beta](../playbooks/releasing-beta.md)
- [Migrating Schema Versions](../playbooks/migrating-schema-versions.md)
