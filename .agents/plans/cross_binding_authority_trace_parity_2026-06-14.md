# Implementation Plan — Cross-Binding Authority-Trace Parity (`08_authority`)

**Created:** 2026-06-14
**Source of truth:** `conformance/08_authority/README.md` ("Remaining (separate task)") and `.agents/next_steps.md` item 1 ("Remaining (separate task): cross-binding trace parity … requires … byte-comparing … through the Python/TypeScript/WASM bindings").
**Originating context:** Follow-up to the 2026-06-14 semantic-infra work on branch `remediation`. The `08_authority` golden (`packs/trace/decision.json`) is now pinned in Rust by `sea-core/tests/authority_fixture_tests.rs`, and authority hashing is deterministic (`canonical_json_string`). This plan closes the last open item: proving the **bindings** reproduce that golden trace, the same way `11_parity` proves parse/validate parity.
**Status of the work today:** All three bindings ALREADY expose the authority engine (`sea-core/src/{python,typescript,wasm}/authority.rs`, added 2026-06-08) and have *basic smoke* tests (`tests/test_authority.py`, `typescript-tests/authority.test.ts`). What does NOT yet exist: (a) the evaluation **inputs** (config/request/facts) committed to `conformance/08_authority/` so every surface drives identical inputs, and (b) parity tests that byte-compare each binding's emitted trace/decision against the shared `08_authority/{trace,decision}.json` golden.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. **Task 1 is a hard dependency** for Tasks 2–4 (it produces the shared input fixtures the bindings load). Tasks 2 (Python), 3 (TypeScript), 4 (WASM) are independent of each other. Task 5 (docs) is last.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **CORE PRINCIPLE — one engine, one golden.** Drive the *existing* binding authority entry points against the *shared* corpus inputs and byte-compare (parse JSON → normalize volatile fields → structural equality) against the *same* `conformance/08_authority/` golden the Rust test pins. Never re-implement evaluation, and never re-serialize a binding-native trace object field-by-field — call the binding, `JSON.parse`/`json.loads` its output, normalize, compare. (This is the same discipline `11_parity` uses for parse/validate.)
- Match surrounding code style. Mirror these existing parity harnesses exactly: `tests/test_conformance_parity.py`, `typescript-tests/conformance-parity.test.ts`, and the WASM `include_str!` pattern in `sea-core/tests/wasm_tests.rs`.
- The corpus is the spec: any change to an `08_authority/*.json` golden must be a deliberate regenerate via `authority_fixture_tests.rs::generate_fixtures`, in the same commit as the code that justifies it — never a test workaround.

### Global verification gates (must stay green after EVERY task)
```bash
cd sea-core && cargo test --features cli --workspace      # Rust core + the 08 Rust pin
just ci-test-python                                       # = .venv/bin/python -m pytest tests/
just ci-test-ts                                           # = bun test
cd sea-core && wasm-pack test --node --features wasm      # WASM suite
cargo clippy --all-targets --all-features -- -D warnings  # lint gate
```
Rebuild bindings before the Python/TS/WASM gates if you touched any `sea-core/src/{python,typescript,wasm}` file (this plan should NOT need to — the binding methods already exist — but if you do):
```bash
just python-setup        # maturin develop into .venv
bun run build            # napi build -> index.node + index.d.ts
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| Authority engine core (the single producer): `new(config)` / `validate()` / `evaluate(&request, &facts) -> (AuthorityTrace, AuthorityDecision)` | `sea-core/src/authority/environment.rs:38,95,141` |
| **Python** binding: `AuthorityEnvironment(config_json).evaluate(request_json, facts_json) -> (trace_json, decision_json)`; also free fn `evaluate_authority(config, request, facts)` | `sea-core/src/python/authority.rs:159,174,227`; registered in `sea-core/src/lib.rs:145-147` |
| **TypeScript** binding: `evaluateAuthority(configJson, requestJson, factsJson?) -> EvaluateAuthorityResult` (fields `traceJson` / `decisionJson`) | `sea-core/src/typescript/authority.rs:148`; types at `index.d.ts:33-35` |
| **WASM** binding: `evaluateAuthority(configJson, requestJson, factsJson?)` (js_name) | `sea-core/src/wasm/authority.rs:159` |
| Bindings serialize the trace with `serde_json::to_string` (compact, NOT pretty/canonical) → compare **structurally after JSON parse**, not as strings | `sea-core/src/python/authority.rs:183` (and TS/WASM equivalents) |
| The committed golden the bindings must reproduce | `conformance/08_authority/{trace,decision,packs,manifest}.json` |
| Rust generator + pinning test to EXTEND (emit + pin config/request/facts here) | `sea-core/tests/authority_fixture_tests.rs` (`generate_raw_fixture`, `generate_fixtures`, `is_volatile`, `normalize_volatile`) |
| Volatile fields that appear in trace/decision output (everything else is pinned, incl. all 7 hashes) | trace `created_at` (`sea-core/src/authority/trace.rs:38`); decision `decision_id`, `trace_ref` (`sea-core/src/authority/types.rs:506`) |
| Input structs to serialize for the corpus: `AuthorityEnvironmentConfig` (note `authority_packs: Vec<serde_json::Value>`), `AuthorityRequest`, `FactEnvelope` | `environment.rs:13`, `types.rs:90`, `types.rs:160` |
| Determinism prerequisite (already DONE): canonical sorted-key hashing | `canonical_json_string` `sea-core/src/authority/types.rs:~605`; `compute_pack_hash` `pack.rs:16`; `action_hash_input` `types.rs:125` |
| Parse/validate corpus loaders that MUST keep skipping `command:"authority_trace"` (they filter to parse/validate) | `sea-core/tests/conformance_corpus_tests.rs:178-182`; `tests/test_conformance_parity.py` `_load_corpus_items`; `typescript-tests/conformance-parity.test.ts` `loadCorpusItems` |
| Existing binding parity patterns to mirror (normalize + compare) | `tests/test_conformance_parity.py`; `typescript-tests/conformance-parity.test.ts`; `sea-core/tests/wasm_tests.rs` (`include_str!` + `normalize_flow_uuids`) |
| Existing authority *smoke* tests (not parity — do not duplicate, do not delete) | `tests/test_authority.py`; `typescript-tests/authority.test.ts` |

---

## Task 1 — Commit the shared evaluation inputs (config / request / facts) to the `08_authority` corpus and pin them in Rust

**Goal:** `conformance/08_authority/{config,request,facts}.json` exist, are the *exact* inputs from which `trace.json`/`decision.json` were generated, are referenced by `manifest.json`, and are byte-pinned by the Rust test. Without committed inputs, no binding can reproduce the golden trace.

**Why this shape:** The bindings take JSON strings (`config_json`, `request_json`, `facts_json`). For "one engine, one golden" to hold, all four surfaces (Rust, Py, TS, WASM) must feed *byte-identical* inputs. The single source of those inputs is the existing Rust builder set in `authority_fixture_tests.rs` — extend it to emit the inputs rather than hand-authoring JSON (which would drift).

### Steps
1. In `sea-core/tests/authority_fixture_tests.rs`, refactor `generate_raw_fixture` to also return the serialized `config`, `request`, and `facts` values (build the `AuthorityEnvironmentConfig` via `make_env_config(vec![prohibition, permission])`, the `AuthorityRequest` via `make_request()`, and `facts` via `vec![make_trusted_fact("customer.credit_status", json!("Hold"))]`) — file: `sea-core/tests/authority_fixture_tests.rs`.
2. Extend `generate_fixtures` (the `#[ignore]` writer) to write `config.json`, `request.json`, `facts.json` (pretty) and add `"config"`, `"request"`, `"facts"` keys to the emitted `manifest.json` — file: `sea-core/tests/authority_fixture_tests.rs`.
3. Add three pinning tests (mirror the existing `*_is_schema_valid_and_byte_pinned` pattern, minus schema validation — there is no `$defs` for config/request/facts): load each committed input file, regenerate from the builders, `normalize_volatile` both, assert equality. `is_volatile` already covers `requested_at`/`observed_at`/`expires_at`, so input timestamps normalize correctly — file: `sea-core/tests/authority_fixture_tests.rs`.
4. Regenerate and commit the inputs:
   ```bash
   cd sea-core && cargo test --features cli --test authority_fixture_tests generate_fixtures -- --ignored
   ```

### Gate
```bash
cd sea-core && cargo test --features cli --test authority_fixture_tests
ls conformance/08_authority/{config,request,facts}.json
```
**Done when:** the three input files exist, the suite passes, and deliberately editing one byte of `request.json` (a non-timestamp field, e.g. `operation`) makes the new pinning test fail.

**Redesign trigger:** if `AuthorityEnvironmentConfig`/`AuthorityRequest`/`FactEnvelope` do not round-trip through `serde_json` cleanly (e.g. a field is `#[serde(skip)]` and the binding `new(config)` then rejects the re-read JSON), pin the *binding-accepted* form: serialize, deserialize, re-serialize once and pin that — never hand-edit to make it parse.

---

## Task 2 — Python authority-trace parity test

**Goal:** a pytest loads `08_authority/{config,request,facts}.json`, calls `AuthorityEnvironment(config_json).evaluate(request_json, facts_json)`, and byte-matches (volatile-normalized) the parsed `trace_json`/`decision_json` against the committed `trace.json`/`decision.json`.

**Why this shape:** proves the PyO3 surface reproduces the Rust golden through the same core — closing G5 for the authority spine. Reuse the volatile-normalization idiom from `tests/test_conformance_parity.py`; do not re-implement evaluation.

### Steps
1. Add `tests/test_authority_parity.py` (keep the smoke test in `tests/test_authority.py` as-is) — file: `tests/test_authority_parity.py`.
2. Load the four corpus files; call `sea_core.AuthorityEnvironment(config_json).evaluate(request_json, facts_json)` → `(trace_json, decision_json)` (signature: `sea-core/src/python/authority.rs:159,174`).
3. `json.loads` each output and each committed golden; normalize volatile keys (`created_at`, `decision_id`, `trace_ref`) recursively; assert equal. Include a regenerate hint in the failure message (`cargo test … generate_fixtures -- --ignored`).

### Gate
```bash
just ci-test-python
```
**Done when:** the new test passes and a deliberate edit to `08_authority/trace.json` (e.g. flip `final_decision`) makes Python fail.

**Redesign trigger:** if the trace differs in a NON-volatile, NON-hash field, that is a real cross-runtime determinism break (the audit's predicted failure) — escalate; do not normalize it away. If only a hash differs, a hashed code path bypassed `canonical_json_string` — fix at the producer (`authority/types.rs`), not the test.

---

## Task 3 — TypeScript authority-trace parity test

**Goal:** same as Task 2, via the TS binding `evaluateAuthority(configJson, requestJson, factsJson)` returning `{ traceJson, decisionJson }`.

**Why this shape:** proves the napi surface; mirror `typescript-tests/conformance-parity.test.ts`'s `normalizeFlowIds`/compare structure with an authority-specific `normalizeVolatile`.

### Steps
1. Add `typescript-tests/authority-parity.test.ts` (leave `typescript-tests/authority.test.ts` smoke test intact) — file: `typescript-tests/authority-parity.test.ts`.
2. Read the four corpus files; call `evaluateAuthority(configJson, requestJson, factsJson)` (types: `index.d.ts:33-35`); `JSON.parse` `traceJson`/`decisionJson` and the goldens.
3. Recursively normalize `created_at`/`decision_id`/`trace_ref`; `expect(actual).toEqual(expectedGolden)`.

### Gate
```bash
just ci-test-ts
```
**Done when:** the new test passes and editing `08_authority/decision.json` makes TS fail.

**Redesign trigger:** same as Task 2.

---

## Task 4 — WASM authority-trace parity test

**Goal:** same as Task 2, via the WASM binding `evaluateAuthority`, with corpus files embedded at compile time (`include_str!`) since WASM has no filesystem.

**Why this shape:** keep it in the existing `wasm-pack test --node` harness (`sea-core/tests/wasm_tests.rs`) using `include_str!`, exactly as the parse/validate WASM tests do — do not stand up a separate node pkg.

### Steps
1. In `sea-core/tests/wasm_tests.rs`, add a `#[wasm_bindgen_test]` that `include_str!`s `../../conformance/08_authority/{config,request,facts,trace,decision}.json` — file: `sea-core/tests/wasm_tests.rs`.
2. Call the WASM `evaluateAuthority` (js_name; `sea-core/src/wasm/authority.rs:159`); parse its trace/decision and the goldens into `serde_json::Value`.
3. Reuse/extend the file's normalize helper to blank `created_at`/`decision_id`/`trace_ref`; `assert_eq!`. Assert `final_decision == "deny"` and `pack_hashes.len() == 2` as a teeth-check on real content.

### Gate
```bash
cd sea-core && wasm-pack test --node --features wasm --test wasm_tests
```
**Done when:** the new WASM test passes alongside the existing 21, and editing `08_authority/trace.json` makes it fail.

**Redesign trigger:** if WASM `evaluateAuthority` returns a differently-shaped result wrapper than Py/TS (e.g. an object vs a tuple), adapt the accessor only — the trace/decision JSON content must still match the shared golden; a content divergence is a real break, escalate.

---

## Task 5 — Flip the docs to "cross-binding parity wired" (only after Tasks 2–4 pass)

**Goal:** `conformance/08_authority/README.md`, `conformance/README.md`, and `.agents/{current_state,next_steps}.md` state that the 08 trace is byte-compared across Rust/Python/TS/WASM — with no status claim ahead of a passing gate.

### Steps
1. `conformance/08_authority/README.md`: replace the "Remaining (separate task)" section with the wired state (Py/TS/WASM each drive `evaluateAuthority` against the committed inputs and byte-match the golden) — file: `conformance/08_authority/README.md`.
2. `conformance/README.md`: update the `08_authority` roadmap row to note cross-binding parity (not just the Rust pin) — file: `conformance/README.md`.
3. `.agents/next_steps.md`: mark the remaining item DONE with evidence; `.agents/current_state.md`: add the parity entry — files: `.agents/next_steps.md`, `.agents/current_state.md`.

### Gate
```bash
just ci-test-python && just ci-test-ts && cd sea-core && cargo test --features cli --workspace && wasm-pack test --node --features wasm
```
**Done when:** all four surfaces pass and every "Wired" claim in the docs is backed by one of the gates above.

**Redesign trigger:** none plausible (docs only).

---

## Final acceptance checklist (whole plan)
- [ ] `08_authority/{config,request,facts}.json` exist, are builder-generated, byte-pinned; editing `request.json` fails the Rust pin. *(Task 1)*
- [ ] Python `AuthorityEnvironment.evaluate` reproduces the golden trace+decision (volatile-normalized); editing `trace.json` fails Python. *(Task 2)*
- [ ] TypeScript `evaluateAuthority` reproduces the golden; editing `decision.json` fails TS. *(Task 3)*
- [ ] WASM `evaluateAuthority` reproduces the golden (`deny`, two `pack_hashes`); editing `trace.json` fails WASM. *(Task 4)*
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` exits 0.
- [ ] `conformance/README.md` + `08_authority/README.md` reflect reality — no status ahead of a passing test. *(Task 5)*
- [ ] All global gates green; bindings rebuilt only if a `src/{python,typescript,wasm}` file changed.

## Guardrails (do not violate)
- Do **NOT** add new binding methods, CLI verbs, or DSL surface. `evaluate`/`evaluateAuthority` already exist; this plan PROVES parity, it does not extend the surface.
- Do **NOT** hand-author the corpus inputs or goldens — they are generated by the single Rust builder in `authority_fixture_tests.rs` and regenerated via `generate_fixtures`. Compare structurally (parse → normalize → equal); never string-compare the bindings' compact JSON against the pretty golden.
- Do **NOT** normalize away anything except `created_at`, `decision_id`, `trace_ref`. All seven hashes and every policy/decision field stay pinned — a divergence there is the finding, not noise.
- Keep `command:"authority_trace"` skipped by the parse/validate loaders (it already is — verify, don't refactor the filters).
- Keep the smoke tests (`tests/test_authority.py`, `typescript-tests/authority.test.ts`) — add parity as new files, don't repurpose them.
