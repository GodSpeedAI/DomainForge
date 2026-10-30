# error — Diagnostics and Fuzzy Matching

## Overview
Formatting backends for `ValidationError` (JSON, human, LSP) plus Wagner–Fischer
Levenshtein matching used to produce "did you mean?" suggestions. Re-exports the
canonical `ErrorCode`/`Position`/`SourceRange`/`ValidationError` types from
`src/validation_error.rs` rather than defining its own.

## Interface
- `[diagnostics.rs:DiagnosticFormatter]` — trait: `format(&self, error, source: Option<&str>) -> String`.
- `[diagnostics.rs:JsonDiagnostic]` (`+from_validation_error`), `[diagnostics.rs:JsonRange]` (`From<SourceRange>`), `[diagnostics.rs:JsonPosition]` — machine-readable diagnostic shape (`code`, `severity`, optional `range`, `message`, optional `hint`).
- `[diagnostics.rs:JsonFormatter]` — `DiagnosticFormatter` impl emitting pretty JSON.
- `[diagnostics.rs:format_errors_json]` — formats a whole error slice as a JSON array.
- `[diagnostics.rs:HumanFormatter]` (`+new(use_color, show_source)`) — colorized diagnostics with source snippets.
- `[diagnostics.rs:LspFormatter]` — Language Server Protocol diagnostic JSON (`source: "domainforge"`, severity 1 for errors / 2 for warnings).
- `[fuzzy.rs:levenshtein_distance]` — Wagner–Fischer edit distance, O(min(m,n)) space.
- `[fuzzy.rs:suggest_similar]` — all candidates within a distance threshold.
- `[fuzzy.rs:find_best_match]` — single best candidate within a threshold, if any.
- `[mod.rs]` — re-exports `ErrorCode`, `Position`, `SourceRange`, `ValidationError`, and `DiagnosticFormatter`, `HumanFormatter`, `JsonFormatter`, `LspFormatter`.

## Invariants
- `[diagnostics.rs:JsonDiagnostic::from_validation_error]` — severity is derived from the error variant, not from caller input: `DeterminismError` maps to `"warning"`, every other variant to `"error"`.
- `[diagnostics.rs:JsonFormatter]` — serialization is fallible-by-construction and must never panic; failures degrade to a manually built minimal JSON object, then to a fixed `{"code":"UNKNOWN",...}` literal.
- `[diagnostics.rs:LspFormatter]` — `DeterminismError` maps to LSP severity 2 (Warning); all other variants map to 1 (Error).

## Side Effects
- None observed in source. Formatters return `String`; no I/O or global state.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
