# DomainForge / SEA-DSL — Semantic Infrastructure Audit

## 2026-07-08 Grounding Correction — READ THIS FIRST

**This correction supersedes the "2026-06-16 Verification Update" below for the purpose of planning future work.** Do not build a plan on the "Closed" verdicts in that section without re-reading this correction.

**What actually happened:** The 2026-06-16 update describes remediation work — a shared `conformance/` corpus, `schemas/seaforge-contract-v1.json`, removal of the boolean evaluation-mode toggle, and ~10 new Rust test files — as if it landed in this repository. It did not land on `main`. That work exists only on the `remediation` branch (local and `origin/remediation`), rooted at commit `d932d0a` ("refactor: remove legacy boolean evaluation mode and enforce three-valued logic", 2026-06-17) and its ancestors (`a48c906`, `ae352a1`, `cfd45f8`, `be4673e`). `d932d0a` is **not** an ancestor of `main` (`git merge-base --is-ancestor d932d0a HEAD` fails). It was never merged.

Verified against `main` on 2026-07-08:

- `conformance/` — **does not exist**. No commit on `main`'s history ever added or removed it; it simply never arrived here.
- `schemas/seaforge-contract-v1.json` — **does not exist**. `schemas/` on `main` has only `ast-v1/2/3.schema.json` and `sea-registry.schema.json`.
- The ~10 test files the update cites as passing (`conformance_corpus_tests.rs`, `kg_roundtrip_honesty_tests.rs`, `authority_fixture_tests.rs`, `seaforge_fixture_tests.rs`, `seaforge_contract_tests.rs`, `order_permutation_hash_tests.rs`, `provenance_tamper_tests.rs`, `golden_trace_stability_tests.rs`, `derived_fact_null_poisoning_tests.rs`, `evolution_enforcement_tests.rs`) — **none exist under `domainforge-core/tests/`** on `main`.
- **The boolean/three-valued runtime toggle is still live on `main`** — `use_three_valued_logic` (field + accessor), `set_evaluation_mode()`, and `evaluate_with_mode()` are all present in `domainforge-core/src/graph/mod.rs` (lines ~19, 26, 83-89, 835-838) and `domainforge-core/src/policy/core.rs` (lines ~181-204), and mirrored in `domainforge-core/src/typescript/graph.rs` and `domainforge-core/src/wasm/graph.rs`. This is exactly the G1 hazard the **original** 2026-06-12 baseline (below) describes as open — it was never closed on `main`, regardless of what the update section claims.
- `docs/specs/canonical_entrypoints.md` and `docs/specs/kg_projection_loss.md`, cited repeatedly as evidence — **do not exist** on `main`. `docs/specs/` currently holds only ADR-/PRD-/SDS-numbered design docs plus a README.

**Separately, the whole repository was renamed** in commit `9d9e209` ("chore(repo): rename sea packages and crates to domainforge (#89)", 2026-06-20) — after this audit and after the abandoned remediation branch. Every `sea-core/...` path below is now `domainforge-core/...`; the CLI binary is `domainforge`, not `sea`; the crate is `domainforge_core`, not `sea_core`. This rename is orthogonal to the remediation-branch issue above but compounds it — a plan built on this doc's literal paths would fail on both counts.

**What is actually true on `main` today, verified 2026-07-08:**

- The CLI subcommand set is real and present: `domainforge-core/src/cli/{parse,validate,validate_kg,import,normalize,pack,project,authority,format,registry,test}.rs`, dispatched from `domainforge-core/src/cli/mod.rs`'s `Commands` enum. Binary: `domainforge-core/src/bin/domainforge.rs`.
- `fixtures/semantic_packs/acme_procurement/{domain,tests,review}/` exists and matches the G6 evidence description.
- `docs/reference/error-codes.md` exists (the report's `docs/specs/error_codes.md` citation is wrong — the file was always at `docs/reference/`, or was relocated there; either way, use `docs/reference/error-codes.md`). A stale copy also sits at `docs/archive/specs/error_codes.md` — archived, not current.
- `docs/how-tos/install-cli.md` exists and matches its citation.
- Most of the individual Rust test files cited in the **original 2026-06-12 baseline** (not the 2026-06-16 update) do exist under `domainforge-core/tests/`, e.g. `runtime_toggle_tests.rs`, `three_valued_quantifiers_tests.rs`, `temporal_evaluation_tests.rs`, `authority_conformance_tests.rs`, `calm_round_trip_tests.rs`, `phase_14_determinism_tests.rs`, `phase_17_round_trip_tests.rs`. These are genuine and current (paths need the `sea-core`→`domainforge-core` substitution only).

**How to use this document for future plans:** Treat the "Original 2026-06-12 Audit Baseline" section (below the historical update) as the actually-current state description, with paths mentally substituted `sea-core/` → `domainforge-core/`. Treat the "2026-06-16 Verification Update" section as a **historical record of abandoned work on the `remediation` branch** — useful as a design reference (the conformance corpus proposal in §5 is a good design to revive), not as a status report. Before starting any new plan from this document, re-verify every cited path; this repo has had a crate rename and at least one abandoned remediation effort since 2026-06-12, and may have moved again since 2026-07-08.

---

## 2026-06-16 Verification Update (historical record — see correction above; describes the unmerged `remediation` branch, not `main`)

This section supersedes the 2026-06-12 verdict below. The original audit is retained as the historical baseline; the table and command results here reflect repository state on the `remediation` branch as of 2026-06-16 — **not** the `main` branch, then or now.

**Verdict on that branch (never merged to `main`):** The remaining blockers found during the 2026-06-16 verification pass were resolved in the `remediation` working tree. `conformance/08_authority/facts.json` was made time-stable because the trusted fact no longer expires, and G8 enforcement was made part of canonical validation because `Graph::validate()` reported breaking `ConceptChange` compatibility as an error. Per the project owner's directive at the time, the legacy boolean policy mode was removed entirely on that branch: three-valued logic was the single canonical semantics across the Rust core and all bindings, closing G1 at the root — on that branch only. The repo still intentionally did not claim order-independent canonicalization (G2), which remained a bounded design choice rather than an unresolved audit blocker.

### Guarantee Matrix as claimed on the `remediation` branch (NOT current on `main` — see per-row status)

| Guarantee | Claimed status (remediation branch, 2026-06-16) | Evidence cited (paths need `sea-core`→`domainforge-core`; branch itself never merged) | Status on `main` today (2026-07-08) |
|---|---|---|---|
| G1. Canonical Meaning | "Closed." Boolean toggle removed entirely. | `sea-core/src/policy/core.rs`, `sea-core/src/graph/mod.rs`, `sea-core/tests/runtime_toggle_tests.rs` | **Still open.** Toggle (`use_three_valued_logic`/`set_evaluation_mode`/`evaluate_with_mode`) is present in `domainforge-core/src/graph/mod.rs` and `domainforge-core/src/policy/core.rs` today. Treat as the original baseline's G1 describes it. |
| G2. Determinism | "Closed for fixed input; order-independence explicitly not claimed." | `sea-core/tests/order_permutation_hash_tests.rs` | File does not exist on `main`. Re-verify against `domainforge-core/tests/phase_14_determinism_tests.rs` instead (this one does exist) before assuming anything about order-permutation coverage. |
| G3/G4. KG Projection and Round-Trip | "Closed by honesty contract." | `docs/specs/kg_projection_loss.md`, `sea-core/tests/kg_roundtrip_honesty_tests.rs` | Neither the doc nor the test exists on `main`. Original baseline's G3/G4 findings (KG import is thin, loss undocumented) still apply. |
| G5. Cross-Language Parity | "Closed for audit scope" via a shared `conformance/` corpus. | `conformance/` dir, `conformance_corpus_tests.rs`, `test_conformance_parity.py`, `conformance-parity.test.ts` | `conformance/` does not exist on `main`. Original baseline's G5 gap (inlined per-language fixtures, no shared corpus, no WASM CI job) still applies. |
| G6. Authority Provenance | "Closed." | `schemas/seaforge-contract-v1.json`, `conformance/08_authority/*` | Schema and conformance dir absent on `main`. The underlying authority module itself (traces, hashing, signing) is real and unrelated to this claim — see original baseline G6, which remains accurate. |
| G7. Policy Logic Safety | "Improved / largely closed." | Same test set as above | Re-verify independently; do not assume closure. Original baseline's G7 analysis is the safer reference. |
| G8. Evolution | "Closed." `Graph::validate()` calls `validate_concept_change_compatibility()`. | `domainforge-core/src/graph/mod.rs::Graph::validate()`, `evolution_enforcement_tests.rs` | Verify directly: `grep -n "validate_concept_change_compatibility" domainforge-core/src/graph/mod.rs`. If absent, original baseline's "declared, not enforced" verdict for G8 still holds. |
| G9. Developer Adoption | "Improved." | `docs/specs/canonical_entrypoints.md` | Doc does not exist on `main`. Original baseline's G9 notes stand. |
| G10. SEA-Forge Integration | "Closed for repo-local contract fixtures." | `schemas/seaforge-contract-v1.json`, `conformance/12_seaforge_fixture/` | Neither exists on `main`. Original baseline's G10 ("no integration fixture, contract schema, or consumer-driven test exists") is the accurate current statement. |

### Verification commands as run on the `remediation` branch (do not assume these pass on `main` — several reference files that don't exist there)

The original list of `cargo test`/`pytest`/`bun test`/`wasm-pack test` invocations from the 2026-06-16 pass is omitted here because it references test binaries (`conformance_corpus_tests`, `kg_roundtrip_honesty_tests`, `authority_fixture_tests`, `seaforge_fixture_tests`, `seaforge_contract_tests`, `order_permutation_hash_tests`, `provenance_tamper_tests`, `golden_trace_stability_tests`, `derived_fact_null_poisoning_tests`, `evolution_enforcement_tests`) that do not exist on `main`. Running them today would fail with "no test target named X," not the "pass" results originally recorded. If this remediation work is revived, re-run and re-record from scratch on `main` (or on a fresh branch off current `main`), using `domainforge-core` naming throughout.

### Current Classification (revised)

DomainForge's `main` branch has **not** closed the original 2026-06-12 gaps. A parallel remediation effort addressed several of them on an isolated branch that was never merged and predates the subsequent `sea`→`domainforge` rename, so even reviving it requires reconciling both the rename and roughly three weeks of intervening `main` history. Treat the design (§5 conformance corpus proposal, the authority trace hashing approach, the KG honesty-contract idea) as sound and worth reviving; treat the completion claims as not reflecting `main`.

---

## Original 2026-06-12 Audit Baseline

**Original date:** 2026-06-12
**Scope:** Can DomainForge serve as the semantic substrate / standard layer for the GodSpeed stack?
**Inputs:** Outcome-to-Code Map (`outcome_code_map_DomainForge_2026-06-12.json`) + direct repo inspection.
**Original verdict:** Strong DSL core with projection gaps. The Rust semantic spine (parser → graph → policy → authority) is genuinely well-built and better-proven than the outcome map suggests. The standard-layer claim fails today on four points: no shared cross-language conformance corpus, KG round-trip is structure-counted not semantics-asserted, a runtime logic toggle creates two meanings for one model, and WASM is untested in CI.

**This verdict is still the accurate description of `main` as of 2026-07-08** (see grounding correction above) — the 2026-06-16 update's "Closed" claims describe an unmerged branch, not this baseline being superseded in fact.

**Corrections to the Outcome-to-Code Map (it is stale in three places):**
- GAP-003 (CLI unclear) is **wrong**: `domainforge-core/src/bin/domainforge.rs` exists (binary name `domainforge`, `sea` in the original report is the pre-rename name), `required-features = ["cli"]`, subcommands `parse, validate, validate_kg, import, normalize, pack, project, authority, format, registry, test`. CI builds and smoke-verifies the binary.
- GAP-005 (WASM untested) is **half-wrong**: `domainforge-core/tests/wasm_tests.rs` exists (`wasm_bindgen_test` cases, `run_in_browser`). But no CI job executes them, so the gap is "untested in CI," not "no tests." (Re-check current test count; the original report's "17" figure was not independently re-verified in this correction pass.)
- `tests/determinism_tests.rs` does not exist; the file is `domainforge-core/tests/phase_14_determinism_tests.rs`.

**Note on paths below:** the original 2026-06-12 text used `sea-core/...`; the crate is now `domainforge-core/...` (renamed 2026-06-20, commit `9d9e209`). Substitute mentally, or better, re-run a path check before citing a specific line number — line numbers were not re-verified in this correction pass beyond the spot checks noted in the grounding correction above.

---

## 1. Semantic Infrastructure Guarantee Matrix

### G1. Canonical Meaning
- **Why load-bearing:** Every downstream consumer (SEA-Forge, projections, bindings) assumes one authoritative meaning per `.sea` model. If meaning varies by surface, the whole stack inherits the ambiguity.
- **Evidence:** Single Rust core (`domainforge-core`) behind all surfaces; Pest grammar → AST → `Graph` (IndexMap-backed, `domainforge-core/src/graph/mod.rs`); bindings are thin wrappers (`src/python/`, `src/typescript/`, `src/wasm/` all call core). `normalized_expression()` provides canonical policy form (`src/policy/core.rs`).
- **Contradicting evidence:** `Policy::evaluate` dispatches on `graph.use_three_valued_logic()` — a **runtime toggle** between boolean and three-valued semantics (`src/policy/core.rs:181-204`). One model therefore has two possible meanings depending on a graph flag. **Confirmed still present on `main` as of 2026-07-08** — see grounding correction above. `test_runtime_toggle` exists in Python and TS, which confirms the toggle is exposed cross-language.
- **Areas to inspect:** `src/policy/core.rs`, `src/graph/mod.rs` (toggle default and persistence — is the toggle serialized into packs/exports?).
- **Maturity:** **partially proven.** Confidence: medium-high for structure, medium for semantics.
- **Failure modes:** Same pack evaluated under different toggle settings yields different decisions with no trace of which semantics applied *outside* the authority path (authority does hash `unknown_handling_config`; plain policy evaluation does not record the mode in `EvaluationResult`).
- **Smallest proof:** Add evaluation-mode to `EvaluationResult` and a test asserting a Null-producing model yields the documented result under each mode; document one mode as canonical for the standard layer.
- **Priority:** High. **Still open on `main`** — this is the single most load-bearing correction in this document.

### G2. Determinism
- **Why:** Governance and audit replay require identical outputs for identical inputs.
- **Evidence:** `domainforge-core/tests/phase_14_determinism_tests.rs`; `Graph` uses `IndexMap` everywhere (insertion-ordered, no HashMap nondeterminism); deterministic flow/hash helpers in CALM export and semantic packs (re-verify exact function names/lines before citing — not re-checked in this pass).
- **Caveat:** IndexMap is insertion-ordered, so determinism holds for a fixed parse order but two semantically identical models written in different declaration order may export differently. That is acceptable if documented, but it means "canonical form" is parse-order-dependent.
- **Maturity:** **proven** for fixed input; **declared** for order-independence. Confidence: high / low respectively.
- **Failure modes:** Hash mismatch on re-serialized packs built from re-ordered but equivalent models; spurious drift detection in SEA-Forge.
- **Smallest proof:** Test: parse two declaration-order permutations of the same model → assert equal canonical pack hash (or document that order is significant). Note: a test file with this exact purpose (`order_permutation_hash_tests.rs`) was built on the abandoned `remediation` branch and never merged — reviving it is lower-effort than writing from scratch if that branch is still reachable (`git show remediation:domainforge-core/tests/order_permutation_hash_tests.rs` after renaming `sea-core`→paths, or cherry-pick).
- **Priority:** Medium.

### G3. Projection Fidelity
- **Why:** Exports are how meaning leaves the system; silent loss poisons every consumer.
- **Evidence:** CALM strong (see G4). KG export is large (`src/kg.rs`) vs a much smaller **import** (`src/kg_import.rs`) — structurally asymmetric; import covers entities/resources/flows via `validate_and_convert` but cannot plausibly reconstruct policies, quantifiers, units, authority constructs from Turtle. Phase-17 tests assert **counts** ("preserves_entity_count") and "round_trip_structure," not semantic equivalence. (Re-verify current line counts before citing exact figures — not re-checked in this pass.)
- **Maturity:** CALM **proven**; KG **partially proven**, with the lossy subset undocumented; JSON/AST **declared**.
- **Failure modes:** KG consumers treat the Turtle export as the full model; re-import silently drops policies/units; downstream reasoners derive conclusions from a weakened model believed complete.
- **Smallest proof:** A manifest test enumerating exactly which constructs survive KG round-trip, asserting the rest fail loudly on import.
- **Priority:** High.

### G4. Round-Trip
- **Evidence:** CALM: dedicated tests including semantic-equivalence and policy round-trip (`domainforge-core/tests/calm_round_trip_tests.rs`), plus schema validation tests. KG: `domainforge-core/tests/round_trip_tests.rs` / `phase_17_round_trip_tests.rs` exercise structure, but **no test asserts graph ≅ import(export(graph)) at the semantic level for KG**, confirming the projection-fidelity gap in spirit.
- **Maturity:** CALM **proven**; KG **partially proven**. Confidence: high / medium.
- **Smallest proof:** One test: build graph with entity+resource+flow+policy+unit → export Turtle → import → assert semantic equivalence or explicit documented loss list.
- **Priority:** High.

### G5. Cross-Language Parity
- **Why:** "One engine, many wrappers" is the core standard-layer claim.
- **Evidence:** Architecture genuinely is wrapper-based (feature-gated bindings over one crate — strong by construction). Mirrored test suites exist across `domainforge-core/tests/`, `tests/` (Python), and `typescript-tests/` (e.g. `test_three_valued_eval.py`, `test_golden_payment_flow.py`, `test_semantic_pack.py`, `test_authority.py`, `test_role_relation_parity.py`, and TypeScript counterparts). CI runs Rust (multi-OS), Python (matrix), and TypeScript jobs (`.github/workflows/ci.yml`).
- **Gaps:** The "golden" DSL fixture is **duplicated inline per language**, not shared — suites can drift independently and CI would not notice. No job asserts Rust/Python/TS produce byte-identical canonical output for the same input. **No WASM job runs the same fixture set** (WASM does have its own dedicated CI job per `.github/workflows/ci.yml`, but it is not part of the cross-language parity assertion). A shared `conformance/` corpus that would close this gap was built on the abandoned `remediation` branch and never merged — see grounding correction above; §5's corpus design is worth reviving.
- **Maturity:** **partially proven.** Confidence: medium.
- **Failure modes:** Binding-level divergence in attribute coercion (e.g., `serde_wasm_bindgen` vs `pythonize` vs napi conversions of numbers/nulls), exactly where parity tests don't look.
- **Smallest proof:** One shared `.sea` fixture file + expected canonical JSON, loaded (not inlined) by all four runtimes, asserting identical output and identical tristate policy results.
- **Priority:** **Highest.**

### G6. Authority Provenance
- **Why:** SEA-Forge must trust decisions it didn't compute.
- **Evidence:** The strongest module in the repo. `AuthorityTrace` hashes the decision context (`ir_hash`, `pack_hashes`, `resolver_semantics_hash`, `specificity_profile_hash`, `unknown_handling_config_hash`, `action_request_hash`, plus `derived_fact_lineage`) in `domainforge-core/src/authority/trace.rs`. Packs carry hash + optional signature with self-verification; `signature_required` sources reject unsigned facts with an explicit reason in `domainforge-core/src/authority/fact_resolver.rs`. Conflict resolution emits explicit `ConflictResolutionStep` lists. `domainforge-core/tests/authority_conformance_tests.rs` exists. Semantic packs have canonical JSON, signing, diff, validator modules and a fixture (`fixtures/semantic_packs/acme_procurement`, confirmed present 2026-07-08).
- **Maturity:** **proven** (within Rust). Confidence: high. The open question is whether traces survive binding serialization intact — untested on `main` (a golden-trace fixture for this was attempted on the abandoned `remediation` branch; not merged).
- **Smallest proof:** Golden trace fixture: same inputs → byte-identical trace JSON across Rust and one binding.
- **Priority:** Medium (verification, not construction).

### G7. Policy Logic Safety
- **Why:** Unknown collapsing to certainty is the classic governance failure.
- **Evidence:** Real `ThreeValuedBool` with a dedicated evaluator; quantifier tests (`domainforge-core/tests/three_valued_quantifiers_tests.rs`); aggregation usage validated pre-evaluation; temporal tests (`temporal_evaluation_tests.rs`, `temporal_semantics_tests.rs`); unit mismatch tests. The critical seam: `EvaluationResult` keeps `is_satisfied_tristate: Option<bool>` **and** a back-compat `is_satisfied = tristate.unwrap_or(false)` in `domainforge-core/src/policy/core.rs`. Null also produces a Violation labeled "UNKNOWN (NULL)" at the policy's modality severity.
- **Assessment:** This is **fail-closed** (Unknown → not-satisfied → violation), which is the safe default for obligations. Two residual hazards: (a) any consumer reading only `is_satisfied` cannot distinguish "violated" from "unknown" — for a *prohibition*, fail-closed flips meaning (unknown evidence reported as the prohibition being violated/triggered, which can mean wrongly blocking — safe — or wrongly alarming — noisy — but for *permissions* "unknown→false" can silently deny); (b) the boolean-mode toggle bypasses three-valued logic entirely, and it is confirmed still live on `main`. The authority path is better: `UnknownHandlingConfig` is explicit, `unknown_handling_applied`/`unknown_handling_result` are recorded and hashed into the trace.
- **Maturity:** **proven** core semantics; **partially proven** at the consumer seam. Confidence: medium-high.
- **Smallest proof:** Tests per policy modality (obligation/prohibition/permission) with a Null condition, asserting the documented decision and that the trace distinguishes Unknown from False.
- **Priority:** High.

### G8. Evolution
- **Evidence:** `ConceptChange` is a metadata record (`from_version`, `to_version`, `migration_policy` as a free string, `is_breaking_change`) in `domainforge-core/src/primitives/concept_change.rs`. `domainforge-core/tests/evolution_semantics_tests.rs` exists. Semantic pack `diff.rs` supports pack comparison.
- **Assessment:** Evolution is **declared, not enforced** on `main`. Verify directly before treating this as settled either way: `grep -n "validate_concept_change_compatibility" domainforge-core/src/graph/mod.rs`. If that function isn't called from `Graph::validate()`, nothing connects a ConceptChange to actual revalidation of dependent policies, projection invalidation, or authority pack version constraints, and `migration_policy: String` remains prose, not semantics. (The abandoned `remediation` branch claimed to wire this; not merged.)
- **Maturity:** **declared**, pending the grep check above. Confidence: medium.
- **Failure modes:** A breaking concept change ships, old policies keep evaluating against new meaning silently; SEA-Forge drift detection has nothing to anchor on.
- **Smallest proof:** Test: model v1 with policy → apply breaking ConceptChange → assert the system *refuses or flags* evaluation of the v1 policy against v2 concepts.
- **Priority:** Medium (becomes High once external adopters exist).

### G9. Developer Adoption
- **Evidence:** `domainforge` CLI with a broad verb set, but `required-features=["cli"]` and default features are empty — `cargo install domainforge-core` does not give you the CLI without `--features cli`; that's a real onboarding trap. Structured validation-error types plus `docs/reference/error-codes.md` (note: not `docs/specs/error_codes.md` as the original draft of this report said — that path never existed; there is an archived copy at `docs/archive/specs/error_codes.md`). Three language READMEs, examples dir, extensive docs tree. Fuzzy suggestions limited to Pest recovery.
- **Maturity:** **partially proven.** Confidence: medium.
- **Failure modes:** Semantic drift via agents/teams writing DSL the parser accepts but the author misunderstands — mitigated only by error quality; no lint/canonical-format gate (`format` and `normalize` subcommands exist but aren't documented as a required workflow).
- **Priority:** Low-Medium.

### G10. SEA-Forge Integration
- **Evidence:** All ingredients exist (packs with hashes/signatures, traces, exports, CLI `project`/`pack`/`authority` verbs). **No integration fixture, contract schema, or consumer-driven test exists in the repo** (confirmed on `main` 2026-07-08 — `schemas/seaforge-contract-v1.json` does not exist here regardless of what the abandoned remediation branch claims). The interface SEA-Forge would consume is implied, not specified.
- **Maturity:** **unclear.** Confidence: low.
- **Failure modes:** SEA-Forge builds against incidental output shapes; any internal refactor becomes a silent breaking change to the governance layer.
- **Smallest proof:** A versioned JSON-schema for "pack + trace + decision" outputs, plus one fixture file SEA-Forge CI can pin.
- **Priority:** High (it is the stated purpose of the whole exercise).

---

## 2. Semantic Spine Map

**Semantic spine** (one meaning must survive end-to-end; any break here is stack-fatal):
- Grammar/parser (`domainforge-core/grammar/sea.pest`, `domainforge-core/src/parser/`)
- AST (`domainforge-core/src/parser/ast.rs`)
- Graph (`domainforge-core/src/graph/mod.rs`)
- Policy engine (`domainforge-core/src/policy/` — including `normalize.rs` canonical form)
- Authority module (`domainforge-core/src/authority/`)
- Semantic packs (`domainforge-core/src/semantic_pack/` — canonical_json + hashing makes packs the *portable* spine format)

**Authority/trust surface:** authority traces, pack signing (`semantic_pack/signing.rs`), fact resolver signature enforcement, conflict resolution steps. Healthy.

**Projection surface:** CALM import/export (proven), KG/RDF/Turtle export (proven one-way), KG import (thin — treat as experimental until G4 closes), JSON/AST serialization (declared), plus `protobuf`/`buf` projection added since the original 2026-06-12 audit (not covered by this document — re-audit if it becomes load-bearing).

**Binding surface:** Python (pyo3), TypeScript (napi), WASM (wasm-bindgen). Architecturally wrappers (good); behaviorally unproven as identical (G5). No projection family (CALM/KG/protobuf) is exposed through any of the three bindings as of 2026-07-08 — projection is CLI-only.

**Developer adoption surface:** CLI (`domainforge-core/src/bin/domainforge.rs`), validation errors/diagnostics, docs, examples. Functional but the CLI's feature-gating undermines "canonical entrypoint" status.

**Optional extension:** SHACL/oxigraph feature, dimensional registry extensions, RDF/XML variant, `wasm_demo.html`.

**Unclear:** the runtime three-valued toggle (`use_three_valued_logic`) — currently spine-adjacent configuration that changes spine semantics; it must be either promoted into the model (declared in `.sea`/pack, hashed everywhere) or collapsed to one canonical mode. **Confirmed still unresolved on `main` as of 2026-07-08.**

The minimum spine that must be conformance-locked: **parser → graph → policy(three-valued) → pack hash → authority trace.** Everything else can be lossy if documented.

---

## 3. Projection Loss Matrix

| Projection | Expected fidelity | Actual evidence | Known/likely gaps | Minimum test |
|---|---|---|---|---|
| Rust → Python | Total (wrapper) | Mirrored suites (three_valued, units, authority, packs); pyo3 over core | Number/None coercion at pythonize boundary; no output-equality assertion vs Rust | Shared fixture → canonical JSON equality vs Rust |
| Rust → TypeScript | Total (wrapper) | Mirrored suites incl. native-binding tests | napi number precision (i64/f64), undefined-vs-null mapping | Same shared fixture equality |
| Rust → WASM | Total (wrapper) | `wasm_tests.rs` exist; runs in its own CI job (re-verify job scope — not fully re-checked); coverage breadth (policy/authority eval vs primitives-only) not re-verified in this pass | Policy evaluation + pack hashing in WASM breadth unconfirmed; `uuid/js` feature implies ID generation differs in WASM | CI wasm job running parse→evaluate→hash on a shared fixture, joined with the Rust/Python/TS parity assertion |
| `.sea` → AST | Total | Parser tests, ~80 Rust test files under `domainforge-core/tests/` | Comment/formatting loss (acceptable); error-recovery paths may accept near-miss syntax | Golden AST snapshot per corpus model |
| AST/Graph → CALM | High, documented loss | Dedicated round-trip tests incl. semantic equivalence + policy round-trip; schema validation | Authority/pack constructs, units in CALM unverified; metadata preservation tested only generically | Extend round-trip to a model using every construct; assert loss list |
| CALM → AST/Graph | High | Import tested via round-trips | Foreign CALM (not produced by DomainForge) handling unknown | Import a hand-written/third-party CALM doc |
| AST/Graph → RDF/Turtle/KG | Medium, loss **undocumented** | Export rich; count-preservation + escaping/URI tests | Policies, quantifiers, units, temporal semantics likely not representable; loss list nowhere stated | Manifest test: enumerate exported constructs vs model constructs |
| KG → AST/Graph | Low | `import_kg_turtle`/`import_kg_rdfxml`; structure-count tests only | Cannot reconstruct policies/units; silent partial import is the danger | Semantic round-trip test or hard error on unsupported triples |
| SemanticPack → AuthorityPack/trace | Total | Pack hashing, trace hashes every input incl. unknown-handling config; conformance tests | Trace stability across binding serialization untested | Golden trace fixture, byte-compared cross-language |
| DomainForge → SEA-Forge | Total (it's the product) | None — no fixture, no schema, no consumer test | Entire contract implicit | Versioned output schema + pinned fixture in both repos |

---

## 4. Policy Logic Safety Review

**True/False/Null:** Sound at the core. `ThreeValuedBool` is threaded through the three-valued evaluator; Null is preserved into `is_satisfied_tristate: Option<bool>`. The back-compat collapse `is_satisfied = unwrap_or(false)` in `domainforge-core/src/policy/core.rs` is fail-closed, and Null additionally emits an explicitly labeled "UNKNOWN (NULL)" violation — so unknown does not silently become *allow*. The residual risks:

1. **Unknown → deny-shaped noise.** For prohibitions and permissions, `unwrap_or(false)` plus a modality-severity violation can convert missing evidence into an apparent policy violation. A consumer counting violations cannot distinguish "broke the rule" from "couldn't tell." The tristate field exists; the question is whether bindings and SEA-Forge are *forced* to read it. Recommendation: deprecate bare `is_satisfied` at binding surfaces.
2. **The boolean-mode toggle is the biggest hazard, and it is still live on `main`.** `evaluate_with_mode(graph, false)` routes through the boolean evaluator, where Null semantics don't exist — unknown evidence must coerce somewhere, and that coercion is invisible in `EvaluationResult` (mode not recorded). Two deployments with different toggle defaults will disagree about the same pack and neither result self-describes its semantics. The authority path hashes `unknown_handling_config`; plain policy evaluation should similarly record its mode.
3. **Quantifiers:** dedicated `three_valued_quantifiers_tests.rs` exists — the vacuous-truth (`forall` over empty set) and Null-element cases appear addressed. Spot-verified, not exhaustively.
4. **Aggregation:** aggregation-usage validation rejects misplaced aggregation pre-evaluation — good. Aggregation over partially-Null collections is the untested edge: does `sum` over facts where one is unknown yield Null (correct) or skip the element (dangerous)?
5. **Units:** mismatch tests exist (`validation_unit_mismatch_tests.rs`); the unsafe path would be comparison of unconverted quantities silently — test inventory suggests this errors. Adequate.
6. **Temporal:** two dedicated test files; semantics of open intervals/missing timestamps versus Null not verified in this audit — include in corpus.
7. **Contradictory facts / conflict resolution:** Authority resolver records every `ConflictResolutionStep`, errors on `SpecificityConflict` rather than picking silently, and rejects conflicting specificity profiles across packs. This is the right shape: **conflicts halt or trace, never silently resolve.**
8. **Derived facts:** `derived_fact_lineage` flows into the trace — derivations are auditable. Whether a derived fact built *from* a Null premise is itself Null is the key untested invariant (Null-poisoning must propagate through derivation).

**Net:** the engine itself is unlikely to turn unknown into allow. The danger lives at the **consumer seams**: bare-boolean readers, the mode toggle (still present), and aggregation/derivation over partial evidence.

---

## 5. Conformance Corpus Proposal (design reference — not yet built on `main`)

**Status note:** an attempt at this corpus was built on the abandoned `remediation` branch (never merged) and is not present on `main`. The design below is still sound and worth implementing fresh against current `main` and `domainforge-core` naming; do not assume any of it exists.

One directory, `conformance/`, consumed by all four runtimes and the CLI. Every item = `.sea` input + `expected/` (canonical graph JSON, policy results as tristate, projection outputs, traces). Parity assertion for every item: Rust (CLI `domainforge validate --json`), Python, TS, WASM produce byte-identical canonical JSON.

1. **`01_minimal_domain.sea`** — the existing payment flow (Role Payer/Payee, Resource Money, Entities Alice/Bob, Flow qty 10, Relation Payment). Expected: canonical graph JSON (2 entities, 1 resource, 1 flow, IDs, order). This promotes the already-duplicated inline fixture to shared ground truth.
2. **`02_policy_complete_facts.sea`** — obligation "every Flow of Money ≤ 100" with all facts present. Expected: `is_satisfied_tristate: true`, zero violations, identical under both logic modes.
3. **`03_policy_null_facts.sea`** — same policy, one flow missing quantity. Expected: tristate `null`, `is_satisfied: false`, violation message contains "UNKNOWN"; **boolean-mode expected result also pinned** (documenting the coercion) — this only makes sense as long as the boolean toggle exists; if it's later removed, drop this pin.
4. **`04_quantifiers.sea`** — `forall` over empty set (expect True), `exists` over set containing a Null-attribute element (expect Null, not False), `exists_unique` with duplicates (expect False).
5. **`05_units.sea`** — Resource in kg, policy threshold in g; expected: conversion applied, result True; plus a dimension-mismatch model expected to **fail validation** with the documented error code.
6. **`06_temporal.sea`** — two flows with timestamps, `before` relation policy; one variant with a missing timestamp expecting Null.
7. **`07_evolution.sea`** — concept v1 + policy, ConceptChange v1→v2 `is_breaking_change: true`. Expected: documented behavior (flag/refusal), pack diff output pinned.
8. **`08_authority.sea` + pack fixture** — extend `fixtures/semantic_packs/acme_procurement`: two conflicting policies from sources of different specificity, one fact from a `signature_required` source. Expected: full `AuthorityTrace` JSON with all seven hashes, `conflict_resolution_steps`, `unknown_handling_applied` — byte-stable.
9. **`09_calm_roundtrip`** — model 1 exported to CALM, re-imported; expected: semantic-equivalence pass + pinned CALM JSON (schema-validated).
10. **`10_kg_roundtrip`** — model using entity/resource/flow **and** a policy; export Turtle, re-import; expected: documented surviving subset equal, documented-lossy subset listed in a manifest, import of the policy triples either reconstructs or errors loudly.
11. **`11_parity`** — items 1–6 executed by all four runtimes; assertion = identical canonical JSON and tristate results. WASM run via wasm-pack/node.
12. **`12_seaforge_fixture`** — `pack.json` + `trace.json` + `decision.json` conforming to a new versioned `schemas/seaforge-contract-v1.json`; SEA-Forge pins this exact fixture in its CI.

~12 files, each small. This corpus *is* the standard; everything else is implementation. **Building it is still Phase 1 of the roadmap below — it has not been done on `main`.**

---

## 6. Standard-Layer Risk Assessment

**Credible as an open semantic standard today (2026-07-08, `main`)?** Not yet — for the same reasons as 2026-06-12. The single-core/wrapper architecture, canonical-JSON pack hashing, and authority trace design are exactly what a standard layer needs and are unusually disciplined. What's missing is not machinery but **proof that the machinery means one thing everywhere**, and that proof was attempted once, on a branch that never merged.

**What makes external adoption premature/dangerous now:**
1. The three-valued toggle: an external adopter can run the same pack with different semantics and both look valid. **Still true.**
2. KG export looks like a full projection but is silently partial; semantic-web consumers will over-trust it. **Still true.**
3. No shared conformance corpus means binding drift is undetectable until a production disagreement. **Still true — the one attempt at fixing this never landed.**
4. The SEA-Forge contract is implicit — anything built on it now is built on incidental output shapes. **Still true.**

**Stack-threatening gaps:** G1 (toggle, confirmed still live), G5 (parity, no shared corpus on `main`), G10 (contract, no schema on `main`). These break trust transitively.
**Developer-experience-only gaps:** fuzzy diagnostics, CLI feature-gating/docs, doc polish. Annoying, not fatal.
**Overbuilt relative to the proof spine:** RDF/XML variant, SHACL/oxigraph feature, breadth of CLI verbs (`project`, `registry`, `format`, `normalize` — useful, but unproven verbs widen the trust surface), the dual logic modes themselves (back-compat machinery serving no external consumer yet, and still present).
**Under-proven despite appearing implemented:** WASM (tests exist, breadth vs CI integration not fully re-verified in this pass); KG import; ConceptChange (metadata without confirmed enforcement — check `Graph::validate()` directly); trace stability across binding serialization.

---

## 7. Minimal Remediation Roadmap

**None of the phases below have landed on `main`.** The abandoned `remediation` branch attempted parts of Phases 0–1 and 4; if reachable, treat it as a source of ideas and possibly cherry-pickable code (after a `sea-core`→`domainforge-core` rename pass and reconciliation with ~3 weeks of `main` history it never saw), not as completed work.

**Phase 0 — Canonical entrypoints (days).**
Objective: one documented way to get one canonical answer. Make `domainforge validate --json` / `domainforge pack` the reference oracle; document `--features cli`; **decide the logic-mode question** — recommend: three-valued is canonical, boolean mode deprecated or recorded in every `EvaluationResult`. Files: `domainforge-core/src/bin/domainforge.rs`, `domainforge-core/src/policy/core.rs`, README. Settlement: a newcomer produces the canonical JSON for a model in <5 minutes. Redesign trigger: none plausible.

**Phase 1 — Golden conformance corpus (1–2 weeks).**
Objective: corpus items 1–8 of §5 in a new `conformance/` directory, run by `cargo test` against the CLI. Settlement: corpus green and treated as the spec — any output change requires a corpus change. Redesign trigger: discovery that canonical JSON is unstable across runs (would indicate hidden nondeterminism — fix before anything else).

**Phase 2 — Cross-language parity CI (1 week after Phase 1).**
Objective: Python/TS jobs load the *shared* corpus (delete inlined duplicates in the Python and TypeScript golden-payment-flow tests) and byte-compare canonical JSON against Rust expected files. Files: `.github/workflows/ci.yml`, both test dirs. Settlement: a deliberately introduced binding divergence fails CI. Redesign trigger: structural inability to get identical JSON (e.g., napi i64 precision) → would force a canonical-serialization layer in core.

**Phase 3 — WASM conformance (days).**
Objective: CI job running `wasm_tests.rs` + corpus items 1–3 (parse, evaluate, tristate, pack hash) under wasm-pack/node, joined into the same parity assertion as Phase 2. Settlement: WASM in the parity matrix. Redesign trigger: `uuid/js` causing ID divergence → IDs must become content-derived or injected.

**Phase 4 — KG round-trip honesty (1 week).**
Objective: corpus item 10; a loss manifest (new `docs/specs/kg_projection_loss.md` — this path does not currently exist, create it); import errors loudly on unsupported constructs. Files: `domainforge-core/src/kg_import.rs`, `domainforge-core/src/kg.rs`, new test. Settlement: every construct is either round-trip-tested or documented-lossy. Redesign trigger: none — worst case KG is formally demoted to a one-way projection, which is acceptable if stated.

**Phase 5 — SEA-Forge contract fixture (1 week, jointly).**
Objective: new `schemas/seaforge-contract-v1.json` + corpus item 12; SEA-Forge CI pins the fixture. Settlement: DomainForge cannot change pack/trace/decision shape without a versioned schema bump. Redesign trigger: SEA-Forge needing data the trace lacks → extend trace, bump schema.

**Phase 6 — Provenance hardening (ongoing).**
Objective: golden-trace byte-stability across bindings; signed-pack tamper test (mutate one byte → hash-mismatch error); derived-fact Null-poisoning test; order-permutation hash test (G2 caveat); evolution enforcement test (G8, contingent on the `validate_concept_change_compatibility()` grep check above). Settlement: the trace is demonstrably replayable and tamper-evident from any binding. Redesign trigger: trace instability across serializers → canonical trace serializer in core.

---

## Final Classification

**DomainForge's `main` branch is a strong DSL core with projection gaps — not yet a credible semantic substrate, but architecturally on the correct path.** This was true on 2026-06-12 and remains true on 2026-07-08; an intervening remediation effort that would have closed several gaps was built on a branch that never merged and now also needs a repo-wide rename reconciled before it can land. The spine (parser → graph → three-valued policy → hashed packs → authority traces) is real, deterministic-by-construction, and fail-closed. What's missing is the proof layer that turns "one engine" into "one meaning": a shared conformance corpus and the discipline of treating it as the standard — plus, this time, actually merging that work to `main`.

**Single highest-leverage next proof:** the shared golden corpus executed identically across Rust (CLI), Python, TypeScript, and WASM with byte-compared canonical JSON and tristate policy results (Phases 1–3), built fresh against current `main` and `domainforge-core` naming rather than resurrecting the unmerged branch as-is.

**Do not expand until the spine is proven:** no new export formats, no new CLI verbs, no new DSL surface area (metrics/mapping/projection contracts), no second logic mode kept alive for back-compat (it is currently alive — decide and act on this explicitly), and no SEA-Forge feature work built against today's unversioned output shapes. Every one of those widens the trust surface that the conformance corpus has not yet covered.
