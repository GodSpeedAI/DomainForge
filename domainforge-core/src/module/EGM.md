# module — Multi-Module Import Resolution

## Overview
Resolves `import … from "…"` declarations across a namespace registry, detects
circular dependencies and symbol collisions, and produces the flat resolved
module set that `application::resolve` consumes during closure resolution.

## Interface
- `[resolver.rs:ModuleResolver]` (`+new/validate_entry/validate_dependencies`) — recursive resolver over a `NamespaceRegistry` binding set.
- `[resolver.rs:ModuleInfo]` — `namespace`, `exports`, `ast` for one loaded module.
- `[resolver.rs:parse_with_registry]` — reads a file from disk given a registry, parses it, and returns `(Ast, ParseOptions)` with `namespace_registry`/`entry_path` pre-filled.
- `[resolver.rs:source_map_from_filesystem]` — `pub(crate)` and `#[cfg(feature = "cli")]`; adapts one filesystem entry plus its transitive import closure into the normalized `SourceMap` consumed by application resolution, returning `(entry_logical_path, SourceMap)`.
- `[resolver.rs:ResolvedModuleSet]` — `pub(crate)`; `modules`, `import_graph`, `symbols`, `aliases`, the single resolver output consumed by both graph and application resolution.
- `[mod.rs]` — `pub mod resolver;` only.

## Invariants
- `[resolver.rs:ModuleResolver::visit]` — a module currently in the `visiting` set cannot be re-entered; hitting it returns `ParseError::circular_dependency` with the accumulated cycle path.
- `[resolver.rs:ModuleResolver::visit]` — a module already in `loaded_modules` is not re-parsed (memoized by canonical path).
- `[resolver.rs:MAX_FILESYSTEM_IMPORT_DEPTH]` (128) and `[resolver.rs:MAX_FILESYSTEM_MODULES]` (1024) — filesystem import traversal is bounded; both constants are `#[cfg(feature = "cli")]`.
- `[resolver.rs:ModuleResolver::visit]` — the effective namespace is the file's `@namespace` annotation when present, else the registry binding for that path, else the registry default.
- `[resolver.rs]` — diagnostics are sorted via `crate::application::diagnostic::sort_diagnostics` before being returned as errors, so module errors are deterministically ordered.
- `[resolver.rs]` — a path whose string form starts with `__std__` bypasses `canonicalize`, keeping virtual stdlib modules addressable.

## Side Effects
- `[resolver.rs:ModuleResolver::new]` — calls `NamespaceRegistry::resolve_files`, which walks the filesystem via globwalk.
- `[resolver.rs]` — reads module sources from disk with `fs::read_to_string` during dependency traversal; `[resolver.rs:parse_with_registry]` and `[resolver.rs:source_map_from_filesystem]` likewise read files.
- `[resolver.rs:validate_entry]`/`[resolver.rs:validate_dependencies]` — call `Path::canonicalize`, which touches the filesystem.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
