# parser — grammar → AST → Graph

Pest PEG grammar (`../../grammar/sea.pest`) → internal AST → `Graph`. **Grammar-first:
every syntax change starts in `sea.pest`, never here.**

## Files

| File | Role |
|------|------|
| `ast.rs` (2.9k) | Pest pairs → internal AST → `Graph` (multi-pass: entities/resources → flows → relations). The big one. |
| `ast_schema.rs` | Stable, serializable AST schema (`--ast --format json` output) |
| `ast_convert.rs` | Internal AST ↔ schema AST conversion |
| `printer.rs` (702) | AST → source (formatter backend) |
| `error.rs` | Parse errors + Pest recovery; fuzzy suggestions (limited) |
| `string_utils.rs` | String/escape handling |
| `profiles.rs`, `lint.rs` | Parse profiles, lint passes |
| `mod.rs` | `parse`, `parse_to_graph`, `parse_to_graph_with_options(ParseOptions)` |

## Entry points (re-exported from crate root)

`parse`, `parse_to_graph`, `parse_to_graph_with_options`.

## Gotchas

- Multi-pass build: entities/resources/roles resolve first, then flows resolve
  names → `ConceptId`, then relations. Forward references work; order within a pass
  uses `IndexMap` insertion order (determinism is parse-order dependent).
- Missing flow `quantity` defaults to `Decimal::ZERO` (`ast.rs`).
- Flow annotations parse **before** `from` (`Flow "X" @tag "v" from ... to ... quantity N`).
- Concept IDs are content-derived (v5) except flows (v4 random — events).
- After grammar changes: regenerate parser tests (`tests/parser_*.rs`) and check
  projections (CALM/KG) still round-trip.
