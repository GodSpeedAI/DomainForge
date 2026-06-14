# cli — the `sea` binary (reference oracles)

Subcommands behind `--features cli` (implies `three_valued_logic`). `bin/sea.rs`
dispatches `mod.rs::Commands` to one module per verb. These commands ARE the
canonical answer; `docs/specs/canonical_entrypoints.md` is the contract.

## Canonical oracles

| Command | Module | Canonical output |
|---------|--------|------------------|
| `sea parse M.sea --format json` | `parse.rs` | Serialized `Graph` (canonical structure) |
| `sea validate M.sea --format json` | `validate.rs` | Per-policy tristate + `evaluation_mode` + aggregate violations |

`validate --format json` emits `{ evaluation_mode, error_count, policies[], violations[] }`;
each policy carries `is_satisfied_tristate` and its `evaluation_mode`. Keys are
serde-sorted → deterministic.

## Other verbs

`import` (kg/calm in), `project` (calm/kg/protobuf out), `format`/`fmt`, `normalize`,
`registry`, `authority`, `pack` (build/validate/inspect/diff/sign/verify),
`validate-kg`, `test`. See each `*.rs`.

## Gotchas

- `validate` exits non-zero when policies are violated, but still writes the JSON
  oracle to **stdout** — callers/tests must read stdout regardless of exit code.
- `pack build --format json` prints meta to **stderr**; the pack itself goes to `--out`.
- Adding a verb: add to `mod.rs::Commands` + a `match` arm in `bin/sea.rs`.
- Output JSON shape is pinned by `../../conformance/` — a shape change must update
  the corresponding `expected/` file (intentional spec change).
