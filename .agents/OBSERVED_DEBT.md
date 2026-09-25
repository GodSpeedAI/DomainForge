# OBSERVED_DEBT.md

Debt, gaps, and inconsistencies noticed during work — NOT the current task,
just things a future change should address. Each entry: what, where, impact,
smallest fix. Append-only unless an item is resolved (then mark `[RESOLVED]`
with the commit that did it).

---

## D1. Model `@version` is parsed into the AST but not carried onto `Graph`

- **Where:** `domainforge-core/src/parser/ast.rs:26` (`metadata.version: Option<String>`,
  populated at `:298`), but `domainforge-core/src/graph/mod.rs:32` (`struct Graph`)
  has no version field and no accessor.
- **Impact:** Any projection that wants the model's declared `@version` (e.g.
  Devbox env var, AsyncAPI `info.version`, CALM schema version) cannot read it
  from the graph and must hardcode or omit. Devbox omits `DOMAINFORGE_VERSION`
  for this reason (`projection/devbox/mod.rs`, marked with a `ponytail:` comment).
- **Smallest fix:** Add `version: Option<String>` to `Graph` (+ `GraphConfig` if
  it belongs with config), populate it during `parse_to_graph`/`build_graph` from
  the AST metadata, expose `pub fn version(&self) -> Option<&str>`.
- **Scope warning:** Per `AGENTS.md` change workflow, touching `graph/mod.rs` is
  a core change — MUST also update PyO3, napi-rs, WASM bindings + their tests.
  Schedule as its own PR, not folded into a projection target.

## D2. `.agents/` state files drifted from git reality

- **Where:** `.agents/current_state.md` was stale on resume — recorded CloudEvents
  (`b44b4a5`) but not AsyncAPI (`38a102c`), even though AsyncAPI was committed.
- **Impact:** A resuming agent trusts the state file and under-counts completed
  work; risks re-doing AsyncAPI or mis-reporting progress.
- **Smallest fix:** Update `current_state.md` at every commit (the workflow rule
  already says this; it just wasn't followed for `38a102c`). No code change —
  process discipline. Consider a pre-commit/lefthook check that fails if
  `current_state.md` mtime < latest commit on an `agent/*` branch. (May be more
  cost than benefit; leave as a manual rule unless drift recurs.)

## D3. Dead git stash `stash@{0}`

- **Where:** `stash@{0}` "On main: scaffolding: projection-target verify scripts..."
- **Impact:** None functional; clutter. Its content was committed as `9eb5049`.
- **Smallest fix:** `git stash drop stash@{0}`. Needs user ok (deletion). Other
  stashes (`stash@{1}`..`stash@{5}`) are older work on unrelated branches
  (`feature/v2plan`, `phase-1-parser-enhancements`, `remediation`) — not touched.

## D4. PR #114 still in draft; CodeRabbit review skips drafts

- **Where:** PR #114 (`agent/projection-families`), OPEN, all CI green, mergeable,
  but `is_draft: true`.
- **Impact:** No automated review posted. `agent/projection-targets` stacks on it
  and can't merge until #114 merges first.
- **Smallest fix:** `gh pr ready 114` (user call — surfaces the review).

## D5. Python/TypeScript bindings still expose projections; status doc says CLI-only

- **Where:** `docs/projection-target-implementation-status.md` "Bindings" section
  says "None of the projection families ... are exposed through the
  Python/TypeScript/WASM bindings today; projection export is CLI-only." But
  only WASM had projections stripped (per prior request); Python (`src/python/`)
  and TypeScript (`src/typescript/`) bindings still expose projection surfaces.
- **Impact:** Documentation contradicts the code. A consumer reading the status
  doc will assume no binding-level projection access exists when it does.
- **Smallest fix:** Either (a) strip projections from Python/TS bindings to match
  the doc, or (b) correct the doc to "WASM-only; Python/TS retain projection
  access." Decide which is the intended end state first — this is a policy call,
  not a mechanical edit.

## D6. CALM import is not exposed through the `domainforge import` CLI

- **Where:** `domainforge-core/src/cli/import.rs:18` — `ImportFormat` has only
  `Sbvr` and `Kg` variants. The library function `calm::import`
  (`domainforge-core/src/calm/import.rs:12`, `pub fn import(calm_json: Value)
  -> Result<Graph, String>`) exists and is exercised by `calm_round_trip_tests.rs`,
  but there is no way to reach it from the CLI.
- **Impact:** No shell-level CALM round-trip is possible. The roundtrip-cell
  gate (`scripts/verify/projection-targets/roundtrip-cell.sh`) had to run a Rust
  integration test instead of a pure `project --format calm` → `import --format
  calm` shell loop, which would have been more consistent with the other gates.
- **Smallest fix:** Add a `Calm` variant to `ImportFormat` + an arm in
  `import::run` that reads the CALM JSON and calls `crate::calm::import`. Low
  risk; mirrors the existing Sbvr/Kg arms.

## D7. [RESOLVED 27660f1] AsyncAPI 2.6→3.0 breaking change — no remaining consumers

- **Where:** The AsyncAPI upgrade (`27660f1`) changed the output filename
  (`asyncapi.json` → `asyncapi.yaml`), version (2.6.0 → 3.0.0), and structure
  (`publish`/`subscribe` channels → top-level `operations` with `send`/`receive`).
  Flagged at commit time as a potential break for any consumer depending on the
  old shape.
- **Resolution (verified read-only sweep, working tree clean):** Repo-wide
  `rg` for `asyncapi` (excluding `target/`, `.git/`, vendored `schemas/asyncapi/`)
  returns exactly 8 files — every one created or updated by the upgrade
  (projection module, CLI dispatch, gate, gate-wiring in `all.sh`, status doc,
  dedicated doc, spec-validation test, `mod.rs` registration). None reference the
  old shape (`asyncapi.json`, `2.6.0`, or `publish`/`subscribe` as structure).
  No Python tests (`tests/`), TypeScript tests (`typescript-tests/`), bindings
  (`src/python|typescript|wasm/`), CI workflows (`.github/workflows/`), or other
  docs reference asyncapi at all. AsyncAPI is CLI-only (per the status doc's
  Bindings section), so the upgrade breaks nothing in-repo. **External consumers
  (outside this repo) are out of scope — not detectable from here.**

---

*Add new items below as noticed during ongoing work. Mark `[RESOLVED <sha>]`
when a commit closes one. Do not delete resolved entries — they record what was
fixed and why.*

## D8. `Graph::absorb` drops `declared_dimensions`/`declared_units`

- **Where:** `domainforge-core/src/graph/mod.rs:194` (`absorb` merges every
  collection except the two declared unit-registry slices). The CLI path
  (`parse`, `validate`, `project` via `build_graph_from_set` in
  `domainforge-core/src/application/resolve.rs`) absorbs one converted graph per
  namespace, so CLI-produced graphs always carry empty `declared_units` and
  `declared_dimensions`. Binding `Graph::parse` (`parse_to_graph`) is unaffected.
- **Impact:** `domainforge parse --format json` omits declared units/dimensions
  that the bindings return; any CLI consumer reading unit declarations from
  graph JSON silently gets none. Found 2026-09-25 while verifying the
  `domainforge-sea` skill: a fixture with `Unit "m" of "Length"` validates clean
  and `Graph.parse` returns it, but CLI graph JSON shows `"declared_units": {}`.
- **Smallest fix:** Add `self.declared_dimensions.extend(other.declared_dimensions);`
  and `self.declared_units.extend(other.declared_units);` to `absorb`, plus a
  Rust test parsing a unit-bearing fixture through `build_graph_from_set`
  (or CLI `parse --format json`) asserting non-empty `declared_units`.
  Left unfixed: the operator is mid-release on `fix/debt-003-domainforge-accessors`
  and the fix, though two lines, would widen that diff.
- **Why absorb excluded them (found 2026-09-25):** no deliberate decision.
  `absorb` predates the fields (blame: untouched since c8f7300) and merges
  field-by-field, so the compiler never flagged the new fields; only the
  exhaustive-destructuring `extend_from_graph` was updated by DEBT-003.
- **Status:** Fixed in working tree (uncommitted) on
  `fix/debt-003-domainforge-accessors`: two `extend` lines in `absorb` plus
  `test_resolve_path_keeps_declared_units_and_dimensions_in_source_order` in
  `dimension_unit_tests.rs`. Verified: new test passes, CLI
  `parse --format json` now lists declared units/dimensions. Mark
  `[RESOLVED <sha>]` at commit time.

## D9. `parse --format human` mislabels counts

- **Where:** `domainforge-core/src/cli/parse.rs` human branch prints
  `Policies: {}` with `graph.pattern_count()` and `Instances: {}` with
  `graph.instance_count()` (resource instances only; entity instances from
  `instance x of "Entity"` are excluded).
- **Impact:** A model with 1 policy + 1 pattern prints `Policies: 1` for the
  wrong reason; a model with only entity instances prints `Instances: 0`.
  Misleads agents and humans reading CLI output.
- **Smallest fix:** Print `graph.policy_count()` under `Policies:` (add the
  accessor if missing), add a `Patterns:` line, and either add
  `EntityInstances:` or document that `Instances:` is resource instances only.
- **Status:** Fixed in working tree (uncommitted) on
  `fix/debt-003-domainforge-accessors`: summary now prints `Resource instances:`,
  `Entity instances:`, `Policies:` (= policy count), and `Patterns:`, plus
  `test_parse_human_summary_counts` in `cli_tests.rs` (passes). Mark
  `[RESOLVED <sha>]` at commit time.

## D10. `fmt` rewrites mapping/projection contract keys into unparseable form

- **Where:** `domainforge fmt` emits quoted keys in mapping/projection bodies
  (`Entity "Warehouse" -> component { "name": "warehouse" }`), but the grammar
  expects unquoted identifiers there — the formatted file fails to parse
  (`Syntax error ... expected identifier`). Verified 2026-09-25: a fixture that
  validates clean (`0 violations`) breaks after `fmt --out`.
- **Impact:** `fmt` is not round-trip safe on files with Mapping/Projection
  contracts; running it in CI or by habit corrupts valid files.
- **Smallest fix:** Make the formatter emit unquoted identifier keys in mapping
  rule / projection override position (grammar-side fix in the fmt module), plus
  a round-trip test: validate → fmt → validate on a mapping-bearing fixture.
  Until fixed, always re-validate after formatting (the `domainforge-sea`
  skill now says this).
- **Status:** Fixed in working tree (uncommitted) on
  `fix/debt-003-domainforge-accessors`: `formatter/printer.rs` writes mapping
  and projection keys as identifiers (the `parser::printer` debug printer
  already did), plus `test_fmt_mapping_projection_round_trip` in `cli_tests.rs`
  (passes). Mark `[RESOLVED <sha>]` at commit time.
