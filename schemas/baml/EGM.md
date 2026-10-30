# schemas/baml — BAML Structural Checker

## Overview
A standalone Python checker that validates the structural well-formedness of
DomainForge-generated `.baml` source. It substitutes for `baml-cli generate`,
which cannot run against the emitted project because the generated `client<llm>`
block is a deliberately commented-out, vendor-neutral placeholder.

## Interface
- `[check_baml.py:main]` — CLI entrypoint `check_baml.py <baml_src_dir>`; walks
  `root.rglob("*.baml")` and exits `0` (well-formed), `1` (malformed), or `2`
  (bad usage / no `.baml` files).
- `[check_baml.py:strip_noise]` — removes raw-prompt blocks (`#"` … `"#`), ordinary
  double-quoted strings, and `//` line comments so delimiter/keyword scanning sees
  only structural tokens.
- `[check_baml.py:check_delimiters]` — reports unbalanced `#"`/`"#` raw-string
  delimiters and unbalanced `{}`, `[]`, `()` on the stripped text.
- `[check_baml.py:BUILTINS]` — `{"string", "int", "float", "bool", "null"}`.
- `[check_baml.py:BLOCK_KEYWORDS]` — `{"enum", "class", "function", "test",
  "client", "generator", "retry_policy"}`.
- `[check_baml.py:resolve]` — resolves a referenced type (stripping a trailing
  `[]`) to a builtin, a declared `enum`, or a declared `class`.
- Assets (not source): `VENDORED.md` — documents why `baml-cli generate` is not
  used and where `BAML_TARGET_VERSION` is pinned
  (`domainforge-core/src/projection/baml/ir.rs`).

## Invariants
- `[check_baml.py:check_delimiters]` — bracket balance is counted only after
  `strip_noise`, so braces inside prompts, strings, or comments never mask or
  fabricate an imbalance.
- `[check_baml.py:BLOCK_KEYWORDS]` — every column-0 block opener (optionally with
  `<...>` generics) must be one of the seven known BAML keywords.
- `[check_baml.py:resolve]` — every parameter type and return type of every
  `function` must resolve to a builtin, a declared `class`, or a declared `enum`;
  an unresolved type is an error.
- `[check_baml.py:test_fns]` — every name inside `test ... functions [ ... ]` must
  reference a `function` that exists in the scanned set.
- `[check_baml.py:files]` — zero `.baml` files under the given directory exits `2`,
  so an empty/mis-pointed run can never be reported as a pass.

## Side Effects
- None observed in source: the checker only reads files and prints to stdout/
  stderr; it writes no files.

## Verification
- Run: `python3 schemas/baml/check_baml.py <dir-containing-baml>`
