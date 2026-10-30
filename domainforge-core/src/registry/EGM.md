# registry — `.sea-registry.toml` Namespace Resolution

## Overview
Loads `.sea-registry.toml`, compiles its namespace glob patterns into a
`GlobSet`, and resolves files (or globs) to namespaces using longest-literal-prefix
precedence. Backs `parse_to_graph_with_options` module resolution and the
`domainforge registry resolve` subcommand.

## Interface
- `[mod.rs:NamespaceRegistry]` — `root`, `default_namespace`, and compiled glob rules.
- `[mod.rs:NamespaceRegistry::new_empty]` — empty registry rooted at a path.
- `[mod.rs:NamespaceRegistry::from_file]` — parse a `.sea-registry.toml`.
- `[mod.rs:NamespaceRegistry::discover]` — walk up from a start path looking for `.sea-registry.toml`; returns `Ok(None)` if none is found.
- `[mod.rs:NamespaceRegistry::root]`, `[mod.rs:NamespaceRegistry::default_namespace]` — accessors.
- `[mod.rs:NamespaceRegistry::namespace_for]` — infallible wrapper over `namespace_for_with_options(path, false)`.
- `[mod.rs:NamespaceRegistry::namespace_for_with_options]` — resolve one path to a namespace; `fail_on_ambiguity` toggles erroring on ties.
- `[mod.rs:NamespaceRegistry::resolve_files]` — infallible wrapper over `resolve_files_with_options(false)`.
- `[mod.rs:NamespaceRegistry::resolve_files_with_options]` — glob-walk the registry root into `NamespaceBinding`s.
- `[mod.rs:NamespaceBinding]` — `{ namespace: String, path: PathBuf }`.
- `[mod.rs:RegistryError]` — `Io`, `ParseToml`, `InvalidVersion`, `MissingNamespaces`, `MissingPatterns`, `InvalidPattern`, `InvalidGlob`, `Walk`, `Conflict`, `Ambiguous`.

## Invariants
- `[mod.rs:REGISTRY_FILE_NAME]` — the registry file name is exactly `.sea-registry.toml`.
- `[mod.rs:namespace_for_with_options]` — when several rules match a path, the rule with the longest literal prefix (before any glob metacharacter) wins; equal-prefix ties error with `RegistryError::Ambiguous` when `fail_on_ambiguity` is set, otherwise the alphabetically smallest namespace is chosen deterministically.
- `[mod.rs:namespace_for_with_options]` — a path outside the registry root resolves to the default namespace instead of an error; no matching rule also resolves to the default namespace.
- `[mod.rs:resolve_files_with_options]` — glob walking sets `follow_links(false)`, so symlinked trees are not traversed.
- `[mod.rs:resolve_files_with_options]` — for a file matched by rules of different namespaces, the longest literal prefix wins; on an equal-prefix tie with `fail_on_ambiguity` set the call returns `RegistryError::Ambiguous` with the sorted namespace list.

## Side Effects
- `[mod.rs:NamespaceRegistry::from_file]` — reads the registry from disk via `fs::read_to_string`.
- `[mod.rs:NamespaceRegistry::discover]` — calls `Path::canonicalize` and probes for the registry file with `is_file` while walking up the directory tree.
- `[mod.rs:resolve_files_with_options]` — walks the filesystem with `globwalk::GlobWalkerBuilder` from the registry root.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
