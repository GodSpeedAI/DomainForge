# Implementation Plan — Semantic Infrastructure: Remaining Work (Phases 2/3/5 completion + clippy debt)

**Created:** 2026-06-14
**Author of source review:** code review of the phases 2–6 changeset on branch `remediation`
**Audit of record:** `.agents/reports/semantic_infrastructure_audit_DomainForge_2026-06-12.md` (§5 corpus, §7 roadmap)
**Status of phases 2–6 today:** functionally landed and green; this plan closes the *honestly-documented partials* in `conformance/README.md` plus one pre-existing clippy condition.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks **in the order given**. Tasks 1→2 are dependent (2 reuses the binding method added in 1). Tasks 3 and 4 are independent and may be done in any order / in parallel.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0. The gate commands are copy-pasteable.
- **One engine, one producer.** The whole point of this work (audit G5) is byte-identical output across surfaces. Never re-implement output JSON in a binding or a test — always call the *same* core function the CLI oracle calls. If you find yourself hand-building JSON in Python/TS/WASM, stop and extract a core function instead.
- Match surrounding code style. Bindings are thin wrappers (`sea-core/src/{python,typescript,wasm}/`). The CLI lives in `sea-core/src/cli/` and `sea-core/src/bin/sea.rs`.
- Keep the corpus the source of truth: any output change must be a deliberate edit to a `conformance/*/expected/` file, never a test workaround.

### Global verification gates (must stay green after every task)
```bash
# Rust (workspace, cli feature)
cd sea-core && cargo test --features cli --workspace
# Python parity + suite
just ci-test-python      # = .venv/bin/python -m pytest tests/
# TypeScript parity + suite
just ci-test-ts          # = npm test  (bun vitest run)
# Lint gate (the CI gate — see Task 4)
cargo clippy --all-targets --all-features -- -D warnings
```
Rebuild bindings before running Python/TS gates if you changed any `sea-core/src/{python,typescript,wasm}` file:
```bash
just python-setup        # maturin develop into .venv
bun run build            # napi build -> index.node + index.d.ts
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| CLI `validate --format json` aggregate `{ evaluation_mode, error_count, policies[], violations[] }` is built here | `sea-core/src/cli/validate.rs` (~lines 160–190), from `ValidationResult` in `sea-core/src/validation_result.rs` |
| CLI `parse --format json` = `serde_json::to_string_pretty(&graph)` | mirrored by `Graph::to_json()` already added to Python/TS bindings |
| Structural parity tests (parse items only today) | `tests/test_conformance_parity.py`, `typescript-tests/conformance-parity.test.ts` |
| Corpus oracle runner + flow-UUID `normalize()` | `sea-core/tests/conformance_corpus_tests.rs` |
| Corpus items: `parse` = 01,07 · `validate` = 02,03,04,05,06 · `kg_roundtrip` = 10 | `conformance/*/manifest.json` |
| Determinism rule: every concept ID is content-derived UUIDv5 **except flows** (UUIDv4, must be normalized) | `conformance/README.md`, `sea-core/src/primitives/flow.rs` |
| SEA-Forge contract schema + in-process validation test | `schemas/seaforge-contract-v1.json`, `sea-core/tests/seaforge_contract_tests.rs` |
| WASM binding: `Graph::parse`, `to_json` (→ `toJSON`, returns `JsValue`), `evaluate_policy` (→ `evaluatePolicy`), `set_evaluation_mode` | `sea-core/src/wasm/graph.rs` |
| WASM CI build target today | `--target web` (`.github/workflows/ci.yml:311`); WASM tests run via `wasm-pack test --node` (ci.yml ~313) |
| `AuthorityError` is a struct `{ code, message:String, recoverable:bool, recoverability_hint:Option<String>, context:serde_json::Value }`; **no `AuthorityResult` type alias exists** | `sea-core/src/authority/error.rs` |
| clippy `result_large_err` fires on ~30 fns, **all in `sea-core/src/authority/*.rs`** | run gate in Task 4 to list |

---

## Task 1 — Cross-language **tristate/validate** parity (Phase 2 completion · audit G5 "Highest")

**Goal:** Python and TypeScript reproduce the CLI `validate --format json` aggregate **byte-for-byte** for corpus items 02–06, so a binding divergence in tri-valued policy evaluation fails CI.

**Why this shape:** the bindings expose `evaluate_policy(policy_json)` (one policy at a time) but **not** the whole-graph aggregate the CLI emits. Build one core producer and wrap it; do not reconstruct the aggregate in tests.

### Steps
1. **Extract a single core producer.** In `sea-core/src/validation_result.rs` (or a new `sea-core/src/cli/validate.rs` helper promoted to the lib), add a pure function that takes a `&Graph` and returns the canonical validate `serde_json::Value` exactly as `cli/validate.rs` currently assembles it. Refactor `cli/validate.rs` to call this function (no behavior change — the existing `conformance_corpus_tests` for 02–06 must still pass unchanged).
   - Suggested signature: `pub fn validate_to_canonical_json(graph: &Graph) -> serde_json::Value`.
2. **Add binding wrappers** that serialize that value with `to_string_pretty` (match `to_json`'s formatting):
   - Python: `Graph.validate_json(&self) -> PyResult<String>` in `sea-core/src/python/graph.rs` (mirror the existing `to_json` at ~line 268).
   - TypeScript: `#[napi] pub fn validate_json(&self) -> Result<String>` in `sea-core/src/typescript/graph.rs` (mirror existing `to_json`). Add `validateJson(): string` to `index.d.ts`.
   - (WASM method is added in Task 2.)
3. **Extend the parity tests** to also run `validate` items:
   - `tests/test_conformance_parity.py`: in `_load_corpus_items`, accept `command in {"parse","validate"}`; for `validate` items call `graph.validate_json()` and compare to the `expected/validate.json`, reusing `_normalize_flow_ids`.
   - `typescript-tests/conformance-parity.test.ts`: same — branch on `manifest.command`, call `graph.validateJson()` for validate items.
   - Keep parse items calling `to_json()/toJson()` as today.
4. **Update docs:** in `conformance/README.md` flip the `11_parity` row from **Partial** to **Wired** and update the prose to state that both `parse` and `validate` items are byte-compared across Rust/Python/TS.

### Gate
```bash
cd sea-core && cargo test --features cli --test conformance_corpus_tests   # oracle unchanged
just python-setup && bun run build
just ci-test-python && just ci-test-ts                                     # validate items now compared
```
**Done when:** parity tests parametrize over 01,02,03,04,05,06,07 and pass; deliberately editing one expected `validate.json` makes Python *and* TS fail.

**Redesign trigger:** if a binding cannot reproduce the aggregate byte-for-byte (e.g. number/null coercion at `pythonize`/`napi`), that is the audit's predicted failure — fix by making the core producer emit a canonical string and have bindings pass it through verbatim (never re-serialize binding-native objects).

---

## Task 2 — Put **WASM** in the byte-compared parity matrix (Phase 3 completion)

**Goal:** WASM runs parse + evaluate(tristate) + canonical-JSON comparison against the **same** `expected/` files, in CI.

**Approach — keep it in the existing `wasm-pack test --node` harness using `include_str!`** (WASM has no filesystem; embed the corpus at compile time). Do **not** stand up a separate nodejs pkg unless the include_str approach proves insufficient.

### Steps
1. **Add WASM string methods** in `sea-core/src/wasm/graph.rs`:
   - `to_canonical_json(&self) -> Result<String, JsValue>` = `serde_json::to_string_pretty(&self.inner)` (a `String`, distinct from the existing `toJSON` which returns a `JsValue`).
   - `validate_json(&self) -> Result<String, JsValue>` = wrap the Task 1 core `validate_to_canonical_json`.
2. **Add WASM conformance tests** in `sea-core/tests/wasm_tests.rs` (extend the existing `#[wasm_bindgen_test]` module). For items 01 (parse) and 02+03 (validate, tristate true / tristate null):
   - `let input = include_str!("../../conformance/02_policy_complete_facts/input.sea");`
   - `let expected = include_str!("../../conformance/02_policy_complete_facts/expected/validate.json");`
   - Parse, call `validate_json()`, parse both JSON strings, **normalize flow UUIDs** (port the small `normalize` from `conformance_corpus_tests.rs`), assert equal. For 03 assert `is_satisfied_tristate == null` and the violation text contains `UNKNOWN`.
3. **CI already runs `wasm-pack test --node`** (added in the phases 2–6 changeset) — the new tests run automatically. No ci.yml change needed unless you split a separate job.
4. **Docs:** in `conformance/README.md` update the WASM (Phase 3) paragraph to state WASM is now in the byte-compared matrix for items 01–03.

### Gate
```bash
cd sea-core && wasm-pack test --node --features wasm
```
**Done when:** WASM parses item 01 and evaluates items 02/03 with canonical JSON matching the Rust `expected/` files (flow IDs normalized), and tristate-null is asserted for 03.

**Redesign trigger:** `uuid/js` producing different flow IDs in WASM → IDs must be content-derived or injected (audit Phase 3 note). If flow IDs differ in *kind* (not just value), normalization will still equalize them; if a *non-flow* ID diverges, escalate — that is a real cross-runtime determinism break.

---

## Task 3 — Pinnable **SEA-Forge fixture** directory (Phase 5 completion)

**Goal:** a committed `conformance/12_seaforge_fixture/` holding real `pack.json` + `trace.json` + `decision.json` that conform to `schemas/seaforge-contract-v1.json`, that SEA-Forge CI can pin/import.

### Steps
1. **Reuse the builders** already in `sea-core/tests/seaforge_contract_tests.rs` (`build_prohibition_pack`, `make_env_config`, `make_request`, `make_trusted_fact`) to produce one pack, one trace, one decision.
2. **Normalize volatile fields** so the fixture is byte-stable: replace `decision_id`, `created_at`, `requested_at`, `observed_at`, `expires_at` with fixed sentinels (e.g. `"<decision_id>"`, `"1970-01-01T00:00:00Z"`). Follow the stripping pattern in `sea-core/tests/golden_trace_stability_tests.rs` (it already removes `created_at`/`decision_id`).
3. **Create files:**
   - `conformance/12_seaforge_fixture/manifest.json` — `{ "description": "...", "command": "seaforge_contract", "pack": "pack.json", "trace": "trace.json", "decision": "decision.json", "schema": "../../schemas/seaforge-contract-v1.json" }`.
   - `conformance/12_seaforge_fixture/pack.json`, `trace.json`, `decision.json` (normalized).
4. **Add a pinning test** (new `sea-core/tests/seaforge_fixture_tests.rs`, `#![cfg(feature = "cli")]`): load each file, validate against the matching `$defs/{pack,trace,decision}` (reuse `validate_against_schema` from `seaforge_contract_tests.rs`), and re-generate from the builders + normalize and assert equality to the committed file (so any shape change forces a deliberate fixture update).
5. **Ensure `command:"seaforge_contract"` is ignored by the parse/validate loaders** — confirm `conformance_corpus_tests.rs`, the Python and TS parity loaders only act on `parse`/`validate` (they already filter; verify 12 is skipped, like 10).
6. **Docs:** flip the `12_seaforge_fixture` row in `conformance/README.md` to **Wired** and point SEA-Forge at the directory.

### Gate
```bash
cd sea-core && cargo test --features cli --test seaforge_fixture_tests
cd sea-core && cargo test --features cli --test conformance_corpus_tests   # still only runs parse/validate
```
**Done when:** the three fixture files exist, validate against v1 schema, and are reproducibly regenerated by the test; mutating one byte of `pack.json` fails the pinning test.

**Redesign trigger:** SEA-Forge needs a field the trace lacks → extend the trace struct and **bump the schema** (`-v2.json`), do not silently widen v1.

---

## Task 4 — Resolve pre-existing clippy `result_large_err` (lint debt, authority module)

**Goal:** `cargo clippy --all-targets --all-features -- -D warnings` exits 0. ~30 findings, **all in `sea-core/src/authority/*.rs`**, caused by functions returning `Result<_, AuthorityError>` where `AuthorityError` exceeds clippy's 128-byte Err threshold. Unrelated to phases 2–6; isolate it in its own commit.

### Steps (decision tree — pick the smallest fix that makes the gate pass)
1. **First, confirm it is real for CI**, not just a newer local clippy:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -c "result_large_err"
   rustc --version && cargo clippy --version
   ```
   If CI pins an older toolchain that does not flag it, still fix (future-proof), but note it is non-blocking.
2. **Measure the struct** to choose the minimal fix:
   ```rust
   // scratch test
   assert!(std::mem::size_of::<sea_core::authority::AuthorityError>() <= 128);
   ```
3. **Preferred fix — shrink the error (least churn):** box the heavy field(s) in `sea-core/src/authority/error.rs`. Start with `context: Box<serde_json::Value>` (and if still over, `recoverability_hint`/`message` are already heap-backed; the real weight is usually `context`). Update the few construction sites in `error.rs`. Re-run the gate. This fixes all ~30 sites with one struct change and **no signature changes**.
4. **If boxing a field still leaves it >128 / lint still fires:** introduce `pub type AuthorityResult<T> = Result<T, Box<AuthorityError>>;` in `error.rs`, switch authority signatures to it (`?` auto-boxes via `From<AuthorityError> for Box<_>`; wrap explicit `Err(AuthorityError::..)` in `Box::new`). Update the bindings that match on the error.
5. **Last resort (document it):** a scoped `#[allow(clippy::result_large_err)]` at the top of `sea-core/src/authority/mod.rs` **with a comment** explaining the error intentionally carries rich audit `context`. Only if 3 and 4 are disproportionate.

### Gate
```bash
cargo clippy --all-targets --all-features -- -D warnings   # exits 0
cd sea-core && cargo test --features cli --workspace        # no behavior regression
```
**Done when:** the lint gate is clean and all tests still pass. Keep this change in a separate commit titled e.g. `chore(authority): shrink AuthorityError to satisfy clippy::result_large_err`.

---

## Final acceptance checklist (whole plan)
- [ ] Parity tests compare **01–07** (parse) **and 02–06** (validate) across Rust/Python/TS; a forced expected-file edit fails Python and TS. *(Task 1)*
- [ ] `wasm-pack test --node` parses 01 and evaluates 02/03 with canonical JSON equal to `expected/`. *(Task 2)*
- [ ] `conformance/12_seaforge_fixture/{pack,trace,decision}.json` exist, schema-valid, byte-pinned by a test. *(Task 3)*
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` exits 0. *(Task 4)*
- [ ] `conformance/README.md` roadmap table shows `11_parity` ✅, `12_seaforge_fixture` ✅, and the WASM paragraph reflects matrix inclusion — **no claim ahead of a passing test.**
- [ ] All four global gates green; bindings rebuilt.

## Guardrails (from the audit — do not violate)
- Do **not** add new export formats, new CLI verbs, or new DSL surface to accomplish any task. Every item above is *proving existing behavior*, not extending it.
- Do **not** keep the boolean logic-mode alive as a second meaning; `evaluation_mode` is recorded in output and that is the canonical record.
- The corpus is the spec: change `expected/` files deliberately and in the same commit as the code change that justifies them.
