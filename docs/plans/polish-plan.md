# Polish Plan

The polish track captures follow-up work items that improve developer ergonomics
and round out the SEA CLI experience.

## Step 4 – Logical Namespace System with `.sea-registry.toml` and Glob Patterns

**Goal:** Automatically map files to logical namespaces so parser defaults remain
stable, even when files omit `in <namespace>` clauses.

**Status:** ✅ Implemented

### Deliverables

- Workspace-level `.sea-registry.toml` that maps glob patterns to namespaces.
- `NamespaceRegistry` loader in `sea_core` with glob validation, duplicate-match
  detection, and helper APIs for both per-file lookups and registry expansion.
- CLI support for validating directories: the `sea` binary now discovers the
  registry, expands all matching `.sea` files, applies the correct namespace
  defaults, and merges the resulting graphs before running validation.
- Documentation under `docs/reference/sea-registry.md` detailing the registry
  format and CLI workflow, plus sample `.sea` files in `examples/namespaces`.
