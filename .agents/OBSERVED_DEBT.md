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
- **Status:** [RESOLVED a5f0956] Fixed on `fix/debt-003-domainforge-accessors`:
  two `extend` lines in `absorb` plus
  `test_resolve_path_keeps_declared_units_and_dimensions_in_source_order` in
  `dimension_unit_tests.rs`. Verified: new test passes, CLI
  `parse --format json` now lists declared units/dimensions.

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
- **Status:** [RESOLVED a5f0956] Fixed on `fix/debt-003-domainforge-accessors`:
  summary now prints `Resource instances:`, `Entity instances:`, `Policies:`
  (= policy count), and `Patterns:`, plus `test_parse_human_summary_counts` in
  `cli_tests.rs` (passes).

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
- **Status:** [RESOLVED a5f0956] Fixed on `fix/debt-003-domainforge-accessors`:
  `formatter/printer.rs` writes mapping and projection keys as identifiers (the
  `parser::printer` debug printer already did), plus
  `test_fmt_mapping_projection_round_trip` in `cli_tests.rs` (passes).

## D11. CodeQL `rust/access-invalid-pointer` fires on napi-derive macro glue; not fixable in-repo

- **Where:** Every `#[napi] pub struct` wrapper in `domainforge-core/src/typescript/`
  (and the same idiom wherever napi classes are declared). PR #125 added 5
  instances; ~17 identical ones were already open on `main`.
- **Evidence (2026-09-25):** `grep unsafe` over our wrapper code returns
  nothing — zero unsafe blocks, zero raw pointers. The flagged dereferences
  are entirely inside napi-derive macro-generated N-API glue, verified in the
  vendored backend sources: `napi-derive-backend 1.0.75`
  (`src/codegen/struct.rs`: `from_napi_ref`/`from_napi_mut_ref`, `env.raw()`
  derefs, `construct` glue) and `napi-derive-backend 5.1.2` (napi 3.5:
  identical shape including `Box::leak(unsafe { Box::from_raw(this_ptr) })`
  in getter glue). Upgrading napi 2.x→3.x therefore cannot silence the
  query, and hand-rolling the glue would be less safe than the macro's.
- **Impact:** Every PR touching a napi wrapper trips the (non-required)
  CodeQL gate with unactionable findings. Dismissal via API returns 404 for
  automation credentials (needs `security_events: write`), so only a
  maintainer can clear them in the Security tab.
- **Next move (maintainer call, not silent):** either dismiss the 22 open
  alerts as false positives with this evidence attached, or migrate CodeQL
  from default to advanced setup with a scoped exclusion for the
  `rust/access-invalid-pointer` query over the binding crates. Upstream
  options: napi restructuring its codegen, or CodeQL refining the query for
  macro-generated FFI glue — both outside this repo.

---

## D12. Evaluator is not consistently Kleene; NULL handling differs between reduced and unreduced trees

- **Where:** `domainforge-core/src/policy/quantifier.rs` (`Expression::expand`,
  `reduce_binary_expression` at `:870`) folds a source-level `and`/`or` whose
  operands both reduced to literals with a NULL-absorbing rule (`false and NULL`
  is NULL, `true or NULL` is NULL). Unreduced trees (every `not`, every non-empty
  `forall`/`exists`) are evaluated with Kleene logic in `domainforge-core/src/policy/core.rs`
  (`false and NULL` is false). `exists_unique` and filtered `count` only count
  conditions that reduced to a literal `true`, so a condition containing `not` is
  never counted.
- **Impact:** The same logical policy can yield different three-valued results
  depending on whether its subtree reduced. The documented semantics (`docs/`) describe
  three-valued logic, which this does not match. The Rego projection
  (`domainforge-core/src/projection/rego/rego/lib.rego`) reproduces the current
  behaviour exactly, so the differential proof (`rego_projection_tests.rs`, `scripts/verify/projection-targets/rego.sh`)
  pins it. Any evaluator change must update `rego/lib.rego` and the differential
  tests together, or the Rego projection will diverge.
- **Smallest fix:** Decide the intended semantics (Kleene everywhere, or NULL-absorbing
  everywhere), make `reduce_binary_expression` and `core.rs` agree, count
  `exists_unique`/`count` conditions consistently, then update `lib.rego` and the
  differential tests in the same change. Not fixed here; the Rego projection is
  deliberately aligned with current behaviour.
- **Status:** Open. Observed on `feat/rego-projection` (2026-10-12); the Rego
  projection is not merged.

## D-serde-saphyr. [2026-10-12] Pre-existing gate failures seen during the serde-saphyr migration
- **Where:** `policy/quantifier.rs:326`, `projection/dagger/mod.rs:191` (`clippy::unnecessary_map_or`, new in Rust 1.92 clippy); `cargo deny check advisories` (roxmltree quadratic duplicate-attribute and namespace allocation advisories; yanked `spin` via `lol_alloc`).
- **Evidence:** `cargo clippy -p domainforge-core --all-targets --features cli -- -D warnings` exit 101 on untouched files; the same run with `-A clippy::unnecessary_map_or` is clean. `cargo deny check advisories` exit 1; no serde-saphyr or yaml advisory.
- **Impact:** the pre-commit clippy hook (`lefthook.yml`) and the `--all-features` CI lint job fail on main-line code, not on this change.
- **Next move:** fix the two lints in a separate change, and bump roxmltree and spin.
- **Also:** `Cargo.lock` header moved from version 3 to version 4 when cargo resolved the new crate. The CLI-binary test expects `<repo>/target/debug/domainforge`, so it fails when `CARGO_TARGET_DIR` is shared; a symlink was used locally.

## D-native-1. macOS and Windows packages are unsigned

- **Where:** `packaging/macos/build.sh` (ad-hoc `codesign --sign -`, no Developer ID, no notarization), `packaging/windows/build.ps1` / `domainforge.iss` (no Authenticode).
- **Impact:** Gatekeeper and SmartScreen may warn or block. The docs state this explicitly; no signature claim is made.
- **Smallest fix:** Add Apple Developer ID signing + `notarytool` and a Windows code-signing certificate in CI, then verify with `codesign --verify --deep --strict`, `spctl -a -vv`, and `Get-AuthenticodeSignature`. Requires credentials the repository does not hold.

## D-native-2. Windows ARM64 is not built

- **Where:** `.github/workflows/package.yml` `windows` job is x86_64 only.
- **Impact:** Windows-on-ARM users cannot use a native package.
- **Smallest fix:** Add a `windows-11-arm` matrix entry with the `aarch64-pc-windows-msvc` target, plus an arm64 Inno Setup architecture entry, once that runner is available to the repository.

## D-native-3. macOS universal is best-effort; Intel fallback is a separate artifact set

- **Where:** `package.yml` `macos`/`macos-intel`.
- **Impact:** If the x86_64 slice cross-build fails, the release carries arm64 and x86_64 packages instead of one universal package. The publish gate accepts either shape and rejects a mislabeled universal artifact.
- **Smallest fix:** None required; re-evaluate once `macos-latest` cross-builds are reliable, or always build both slices natively and lipo them in the Intel job.

## D-native-4. package.yml also runs on every push to main

- **Where:** `package.yml` `on.push.branches`.
- **Impact:** Adds four platform builds per main merge (release candidates are downloadable from the run). The release path is unaffected.
- **Smallest fix:** Remove `main` from the push trigger if cost outweighs the convenience; `workflow_dispatch` and `workflow_call` remain.

## D-doc-egm-1. `println!` debug statement left in the flow-annotation parse path

- **Date:** 2026-10-30
- **Location:** `domainforge-core/src/parser/ast.rs:1283`
- **Debt:** `println!("debug insert annotation {} = {:?}", key, value)` runs on every flow annotation parsed in release builds. It pollutes stdout for library consumers (the Python/TypeScript/WASM bindings and any caller that parses into a `Graph`) and can corrupt machine-readable output such as `domainforge parse --format json`, since the debug line is interleaved with the intended payload.
- **Evidence:** Code reading while documenting `domainforge-core/src/parser`. The statement sits inside `parse_annotation`'s flow-annotation arm, immediately before `annotations.insert(key, value)`, with no `cfg(test)` or `log::debug!` guard. Not executed to reproduce.
- **Confidence:** medium
- **Closure:** Delete the line or convert it to `log::debug!`. File touched: `domainforge-core/src/parser/ast.rs`. Size S.

## D-doc-egm-2. `domainforge test` subcommand is a registered stub

- **Date:** 2026-10-30
- **Location:** `domainforge-core/src/cli/test.rs:9` (`run`); surface declared at `domainforge-core/src/cli/mod.rs:34` (`Commands::Test`)
- **Debt:** `Commands::Test` is part of the public CLI subcommand enum and is documented in `--help`, but `test::run` only prints `"Test runner not yet implemented. Pattern: {}"` and returns `Ok(())` — exit code 0. A script or CI step invoking `domainforge test <pattern>` therefore reports success while testing nothing.
- **Evidence:** Code reading while documenting `domainforge-core/src/cli`. `run` body is a single `println!` followed by `Ok(())`. Not executed.
- **Confidence:** medium
- **Closure:** Either implement the runner, or make the stub fail non-zero (or hide the subcommand behind `#[command(hide = true)]`) so it cannot be mistaken for a passing gate. Files: `domainforge-core/src/cli/test.rs`, possibly `domainforge-core/src/cli/mod.rs`. Size S to M depending on chosen direction.
