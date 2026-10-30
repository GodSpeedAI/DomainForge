# Current State: EGM documentation bootstrap (2026-10-30)

- [x] Ran the `egm-bootstrap` skill over the whole repository with `target_filename=EGM.md` (chosen so the 12 existing human-maintained `README.md` files, including the 23 KB root README, stay untouched — verified: no `README.md` appears as a new/modified file from this task).
- [x] Scope after BFS discovery + bottom-up pruning: **60 code-bearing directories**. Docs-only, test-only, build-output, and gitignored trees were excluded (`docs/`, `typescript-tests/`, `tests/`, `node_modules/`, `target/`, `dist/`, `pkg/`, `fixtures/`, `evidence/`, `secrets/`, `graft/`, `docs/archive/`). The discovery count (60) exceeded the skill's 15-directory safety gate, so explicit user approval was obtained before any worker ran.
- [x] Four parallel sub-orchestrator workers partitioned the tree (Adaptive Depth Partitioning), producing 60 directory contracts, plus 1 root `EGM.md` blueprint written by the root orchestrator = **61 files**.
- [x] Classification: 58 Peanuts (direct source files) and 3 Hays (`domainforge-python`, `schemas`, and the root blueprint). Verified against direct-file counts, not recursive counts.
- [x] **Grounding verified mechanically**: a checker script resolved every backtick `[file:symbol]` citation against the filesystem — `checked=1612 ok=1612 bad=0`. Two defects were found and fixed during this pass: `scripts/verify/EGM.md` cited a non-existent `:elif` symbol (the script uses `if/else`), and `domainforge-core/src/cli/EGM.md` cited `[lib.rs]` from a subdirectory (now `[../lib.rs]`).
- [x] **Link verification**: zero broken relative `EGM.md` links across all 61 files (both markdown-link and backticked-path forms checked).
- [x] **No test/behaviour claim was weakened**: documentation-only task, so no assertions were touched. `cargo test -p domainforge-core --features cli` → **1,294 passed, 0 failed, 0 failed suites**. No `.rs`, `.py`, `.ts`, `.js`, `.sh`, `.pest`, `.toml`, or `.json` file was modified.
- [x] Deliberate non-contracts recorded as gaps rather than invented: `python/sea_dsl/` is genuinely empty and `python/domainforge/` holds only gitignored `.so` artifacts, so neither got a contract. `packaging/windows/` (only `.ps1`/`.iss`) and `schemas/{archimate,asyncapi,bpmn,cep_eval,cmmn,otel}` (vendored XSD/JSON, no source) are documented as unlinked bullets in their parents.
- [ ] **Not staged and not committed** (skill VCS-isolation constraint). The pre-existing staged changes to `README.md` and `docs/**` were present before this task and were not authored by it.
- [ ] Optional follow-up not yet run: the `egm-drift-report` skill against the 61 contracts.

## Observed debt recorded during this task

Two entries were appended to `.agents/OBSERVED_DEBT.md` while reading `domainforge-core/src`; both were independently verified against source by the root orchestrator:

- `D-doc-egm-1` — `println!("debug insert annotation ...")` at `domainforge-core/src/parser/ast.rs:1283` runs in release builds and can corrupt `domainforge parse --format json` output. Confirmed by reading the exact line.
- `D-doc-egm-2` — `domainforge test` is a registered stub returning `Ok(())` (exit 0) at `domainforge-core/src/cli/test.rs:9`. Confirmed by reading the whole 11-line file.

# Current State: Native CLI distribution (2026-10-28, branch feat/native-cli-distribution)

- [x] New reusable `.github/workflows/package.yml` builds the standalone CLI on Ubuntu 22.04 amd64/arm64, macOS (universal, with native Intel fallback), and Windows x86_64; every job installs its own package and runs the functional smoke before uploading `pkg-*` artifacts.
- [x] `deploy.yml` now calls `package.yml` at the release tag, then `publish-native` requires all four platform `built-from-*.txt` files to equal the tag commit, generates and verifies a global `SHA256SUMS`, optionally attests provenance, and attaches assets to the GitHub Release. It fails closed on any missing/short artifact set.
- [x] Linux packaging: `packaging/version.sh` (drift gate), `packaging/check-linux-binary.sh` (ELF64, glibc ≤ 2.35, loader, RPATH, Nix, NEEDED whitelist), `packaging/deb/build.sh` (`dpkg-shlibdeps`), `packaging/linux/build-tarball.sh` (reproducible tar.gz), `packaging/smoke.sh`.
- [x] macOS `packaging/macos/build.sh` (lipo, ad-hoc codesign, pkgbuild, DMG with same payload) and Windows `packaging/windows/build.ps1` + `domainforge.iss` (per-user Inno Setup, user PATH add/remove) + `smoke.ps1`.
- [x] `ci.yml` `package-linux` job is a PR merge gate: build, ABI check, .deb/tarball, install + smoke + uninstall, clean Ubuntu 22.04/Debian 12 container install, checksums.
- [x] Documentation: `docs/distribution/native-cli.md` (matrix, install/uninstall, Jetson arm64, signing status, verification, limitations); `.github/workflows/README.md` describes the packaging and publish path; `just native-package-linux`, `native-smoke`, `native-check`.
- [x] Local evidence: `sh packaging/version.sh` -> `0.21.2`; `packaging/smoke.sh` against `target/release/domainforge` PASS; `packaging/deb/build.sh` and `build-tarball.sh` exercised with a glibc-2.34 test ELF (contents/metadata/reproducible layout verified); all shell scripts pass `sh -n`; actionlint 1.7.12 (CI-pinned checksum) passes on all three workflows; repository release/security/gate lints pass.
- [x] Committed `1180ed1` and pushed `feat/native-cli-distribution`; PR #164 opened. Branch `Package Native CLI` run 40687663700 passed all four jobs (macOS universal produced, Intel fallback correctly skipped). Artifacts downloaded and all per-platform `SHA256SUMS-*` verify.
- [x] CI run 40689541396 on PR #164 is fully green, including `package-linux`: ELF amd64 max GLIBC_2.34, `.deb` `Depends: libc6 (>= 2.34), libgcc-s1 (>= 4.2)`, smoke PASS, install/uninstall clean, and clean `ubuntu:22.04` + `debian:12` container installs both report `domainforge 0.21.2`.
- [x] Downloaded arm64 `.deb` inspection: ELF64 AArch64 PIE, interpreter `/lib/ld-linux-aarch64.so.1`, max `GLIBC_2.34`; amd64 tarball re-run on the local host: `smoke: PASS`.
- [x] Artifact hashes: amd64 deb `eeba5b56...`, arm64 deb `aaa26ab3...`, x86_64 tar `cfe1defc...`, aarch64 tar `0266d663...`, macOS universal pkg `4c9a8dc8...`, dmg `de0e6128...`, Windows zip `3c5bd7b0...`, setup `66850910...` (full values in PR #164 and run artifacts).
- [ ] Not merged/released. macOS/Windows packages remain unsigned; attaching to a GitHub Release happens only on the next `vX.Y.Z` deploy run.

# Current State: Rego and OpenShell projection targets (2026-10-12, branch feat/rego-projection)

- [x] Two projection targets on `feat/rego-projection`, not merged and not released: `--format rego` (OPA/Rego bundle with a `domainforge.rego.decision/v1` contract, differential-tested against real OPA 1.21.1) and `--format openshell` (OpenShell v0.1.3 `policy.yaml` plus sandbox contracts, boundary-checked by `openshell-prover` 0.1.3). Source: `domainforge-core/src/projection/{rego,openshell}/`; tests: `rego_projection_tests.rs` (9), `openshell_projection_tests.rs` (10).
- [x] Gate scripts `scripts/verify/projection-targets/rego.sh` and `openshell.sh` (both require their tool on PATH, no skip); contracts `contracts/rego.yaml` and `contracts/openshell.yaml`; both appended to `all.sh`; `just rego-verify` and `just openshell-verify`.
- [x] CI: jobs `verify-rego` and `verify-openshell` added; `verify-projection-targets` and `prove` install OPA 1.21.1 and openshell-prover 0.1.3 because both run `all.sh`.
- [x] Docs: `docs/rego-projection.md`, `docs/openshell-projection.md`; rows in `docs/projection-target-implementation-status.md` and `docs/projection-families.md`; `PROOFS.md` rows `CLAIM-PROJ-REGO` and `CLAIM-PROJ-OPENSHELL` (both `partial`).
- [x] Run 2026-10-13 after merging `origin/main` (6d219f8, serde-saphyr/MSRV 1.89): `openshell_projection_tests` 11 passed incl. 2 prover-backed; `scripts/verify/projection-targets/openshell.sh` passes; `rego_projection_tests` 9 passed with OPA 1.21.1; `cargo test -p domainforge-core --features cli` 1293 passed, 0 failed; clippy `--all-targets --all-features -D warnings` clean; fmt clean; MSRV 1.89 `check --locked --features cli` passes. `serde_yaml` and `unsafe-libyaml` are gone from this branch.
- [x] OpenShell `policy.yaml` now serializes through `projection::yaml::to_yaml` (serde-saphyr). For the repair-agent fixture the bytes are unchanged, so `policy_digest` `sha256:f32151bf...` is unchanged. New test `policy_yaml_strings_survive_yaml_1_1_readers`.
- [ ] Recorded debt D12 (evaluator NULL handling is not consistently Kleene); the Rego library reproduces current behaviour.

## Previous state: serde_yaml -> serde-saphyr (2026-10-12, branch fix/replace-serde-yaml)

- [x] Dependency: `serde_yaml` removed from `domainforge-core`; `serde-saphyr = 1.3` (`serialize` only) in `[dependencies]`, `deserialize` added in `[dev-dependencies]`. `cargo tree -p domainforge-core -e normal,features -i serde-saphyr` shows only `serialize`. Normal build gains `serde-saphyr` v1.3.0 and `nohash-hasher` v0.2.0. `cargo tree -i unsafe-libyaml --workspace` and `cargo tree -i serde_yaml --workspace`: no match.
- [x] MSRV raised 1.77 -> 1.89 (owner-approved). Updated: `domainforge-core/Cargo.toml`, `domainforge-core/README.md` (badge, section, compatibility note), `docs/how-tos/install-cli.md`, `docs/how-tos/parse-sea-files.md`, `docs/reference/wasm-api.md`, `.agents/plans/execution-plane.md`. No sibling crate declares `rust-version`.
- [x] CI: `.github/workflows/ci.yml` job `msrv` ("MSRV (1.89)") runs `cargo check -p domainforge-core --locked` and `... --features cli` on `dtolnay/rust-toolchain@1.89.0` with `RUSTUP_TOOLCHAIN=1.89.0` (rust-toolchain.toml pins 1.92.0). Local: `cargo +1.89.0 check -j 2 -p domainforge-core --locked` passes, default and `--features cli`, separate target dir `domainforge-msrv-target`.
- [x] Emitter: `projection/asyncapi/mod.rs` `to_yaml` = `serde_saphyr::to_string_with_options` with default options except `prefer_block_scalars = false`. Tests parse with `serde_saphyr::from_str`.
- [x] Byte comparison (5 AsyncAPI fixtures): not byte-identical. All 47 changed lines are quoting only (`asyncapi: "3.0.0"`, `version: "1.0.0"`, `$ref` single -> double quotes). Key order, indentation, blank lines, width and trailing newline are unchanged. Parsed values are equal under serde_yaml (before vs after) and serde-saphyr (after), per document.
- [x] Adversarial quoting (`emitted_yaml_preserves_ambiguous_strings`, 45 values incl. 300-char and multi-line): round-trips under serde-saphyr; serde_yaml 0.9 parses the same emitted document with all values strings and equal to serde-saphyr. `emitted_yaml_is_deterministic` added.
- [x] Gates: `cargo test -j 2 -p domainforge-core --features cli --no-fail-fast`: 1278 passed, 0 failed, 7 ignored (pre-existing). `cargo fmt --all -- --check` clean. `scripts/verify/projection-targets/asyncapi.sh` passes (3 schema tests).
- [ ] `cargo clippy --all-targets --features cli -- -D warnings` fails on 2 pre-existing `clippy::unnecessary_map_or` lints in untouched files (`policy/quantifier.rs:326`, `projection/dagger/mod.rs:191`). With that lint allowed, clippy is clean. See OBSERVED_DEBT.
- [ ] Not released. Commit type/footer decides the bump (see next_steps).

# Current State: fallible CEP world mint (2026-10-04)

- [x] Development and CI toolchain pinned to Rust 1.99.0 (`rust-toolchain.toml` is the source of truth; Devbox `RUSTUP_TOOLCHAIN` override removed; CI and release workflows pin 1.99.0; `just toolchain-check`). Validation on 1.99.0: fmt clean, clippy `--all-targets --all-features -D warnings` clean, `cargo test -p domainforge-core --features cli` 1276 passed. MSRV (`rust-version`) unchanged here.
- [x] `build_cep_envelope` leaves a failed `WorldRef` mint unpinned; successful minting preserves the existing tuple. `registry_hash_validation_controls_world_pinning` reproduced the caller-supplied invalid registry hash panic before the fix and covers invalid, valid, and absent hashes after it. Evidence: `cargo test -p domainforge-core --features cli --test cep_snapshot_world_tests --test envelope_emit_tests -- --test-threads=1` passed 27 tests; CLI Clippy with `-D warnings` and workspace fmt passed under Rust 1.92.0. Two envelope tests failed in the initial concurrent run and passed serially on both original and fixed code. Graft rebuild unavailable (CLI missing).

# Current State: CEP snapshot world binding (2026-10-04, branch feat/cep-snapshot-world-ref, off feat/world-ref)

- [x] `domainforge envelope --emit cep|both --world-name <name> [--world-alias world:<n>] [--world-label <text>]` binds the snapshot to its world: `scope.world_ref`, `integrity`, `extensions["domainforge.identity"]`, `extensions["cep.profile"]` (godspeed.semantic_snapshot 1.0.0). An invalid model is flagged (`extensions.domainforge.model_validation_status`) and never pinned. Opt-in; emission without `--world-name` is unchanged. Evidence: `domainforge-core/tests/cep_snapshot_world_tests.rs` (8, written red first), `envelope_emit_tests` unchanged and green, `cargo test -p domainforge-core --features cli` 1275 passed / 0 failed, clippy `-D warnings` and fmt clean.
- [x] Conformance against the CEP profile (cep repo `GodSpeedAI/canonical-evaluation-protocol`, branch `profiles/godspeed-v1`): real emissions from this build pass the Python, standard-jsonschema, Rust and Ajv validators (gated by `CEP_TEST_DOMAINFORGE_BIN`).
- API note: `CepEnvelopeParams` gained a `world: Option<CepWorldParams>` field (struct literals need `world: None`); `CepWorldParams`, `CEP_PROFILE_SEMANTIC_SNAPSHOT`, `CEP_PROFILE_VERSION` are new public items.
- [ ] Not released. `world_ref` landed in #132 (86aaf6f); this builds on it. Release (expected 0.19.0) follows this PR.

# Current State: `world_ref` (2026-10-04, branch feat/world-ref)

- [x] Added `world_ref` / `world_alias` / `world_label` and `WorldCatalog` in `domainforge-core/src/application/world.rs`; `DomainModelIdentity::from_document` and `validate`; JSON entry points shared by bindings. The digest is exactly `DomainModelIdentity::canonical_digest()` (no second hash). Evidence: `domainforge-core/tests/world_ref_tests.rs` (16), `world_ref_golden_tests.rs` (3); red before implementation (did not compile), green after.
- [x] Cross-language golden vectors: `domainforge-core/tests/fixtures/world_ref/golden-vectors.json` generated and self-checked by the independent stdlib-only `world_ref_reference.py`; reproduced by Rust, Python (`tests/test_world_ref.py`, 24), TypeScript (`typescript-tests/world_ref.test.ts`, 23) and WASM (`wasm_tests.rs`).
- [x] Bindings (thin wrappers on `Graph`): `domain_model_identity_json`, `world_ref_from_identity_json`, `verify_world_ref`, `parse_world_ref` in `src/python`, `src/typescript`, `src/wasm`; `domainforge-typescript/index.d.ts` regenerated by `napi build`.
- [x] Full gates: `cargo test -p domainforge-core --features cli` 1267 passed / 0 failed; `just wasm-test` 23 passed; pytest all passed; vitest 221/221; clippy `-D warnings` clean; fmt clean.
- Behavior to know: `DomainModelIdentity` binds the exact source set and the producer release, so a comment-only edit or a DomainForge upgrade mints a new `world_ref` while `semantic_closure_hash` stays identical (asserted in `comment_only_edit_is_a_new_world_with_the_same_semantic_closure` and the `compiler-upgrade-same-closure` vector). Documented in `docs/reference/sea-application-contract.md` section 14.1.
- [ ] Not yet released: consumers pin 0.18.2, which lacks `world_ref`. A release (release-please) and a consumer bump are required before sea-forge, Cognate or RealityTrace can call it.


- [x] Fixed two `domainforge fmt` defects: (1) `Expression::Display` for `Quantifier` emitted a debug-style `ForAll(...)` wrapper the grammar doesn't accept instead of `forall … in …: (…)`, and every collection-printing site (`Quantifier`/`Aggregation`/`AggregationComprehension`/`GroupBy`) printed the internal `entity_instances:X` encoding verbatim instead of `entity_instances of "X"`; (2) `Formatter::format_ast` only ever re-emitted file-header comments, silently dropping every other `//` comment, and `extract_comments` treated `//` inside string literals as a comment start. Evidence: `domainforge-core/src/policy/expression.rs`, `domainforge-core/src/formatter/{printer,comments}.rs`; 16 new tests in `formatter::printer::tests` (quantifier round trips incl. non-Binary condition and `entity_instances of`, comment preservation incl. post-`@namespace`, trailing, EOF, and string-literal-not-a-comment cases); `cargo test -p domainforge-core --features cli` 0 failed (full suite incl. doctests, confirmed clean after ruling out a transient concurrent-build duplicate-crate-version doctest failure); `cargo clippy -p domainforge-core --all-targets --features cli -- -D warnings` clean; `cargo fmt --all --check` clean; `just python-test` 237 passed/5 skipped/0 failed and `just ts-test` 198 pass/0 fail against binaries rebuilt from the change (`maturin develop --release`, `bun run build`); acceptance run against copies of both cognate reconstruction `interaction-model.sea` files (copilot, deepseek-harness) — `validate` 0 violations, `parse --format human` counts unchanged, `//` comment counts unchanged (2 and 0 respectively), `fmt` idempotent. Not committed (no commit/version bump requested).

## Turtle IRI delimiter parsing (2026-09-26)

- [x] `domainforge-core/src/kg.rs` keeps semicolons and commas inside `<...>` IRIs when parsing Turtle predicate and object lists. Both new regressions failed before the fix and passed afterward; the final `cargo test -p domainforge-core --lib` run passed 383 tests with 1 ignored, `cargo test -p domainforge-core --features cli --test cli_import_tests` passed 2 tests, and Clippy, rustfmt, and `git diff --check` passed under Rust 1.92.0.
- [ ] CodeRabbit review and graft index rebuild: review is disabled for this task; graft CLI is unavailable in this sandbox.

Branch: `fix/debt-003-domainforge-accessors` (merging to `main`).

- [x] New read accessors on `Graph` in TypeScript, Python, and WASM bindings for policies, metrics, mappings, projections, patterns, concept changes, entity instances, declared units, and declared dimensions (`DeclaredUnit`/`DeclaredDimension` wrappers; mapping rules, projection overrides, and base factor as strings). Evidence: `domainforge-core/src/{typescript,python,wasm}/graph.rs` + `primitives.rs`; `typescript-tests/graph.test.ts` 12/12; `tests/test_graph.py` new test; new `wasm_tests::test_graph_exposes_parsed_declarations_in_source_order` (22/22 wasm).
- [x] Deterministic declaration order preserved (IndexMap; no HashMap iteration). Evidence: double-parse snapshot equality asserted in TS, Python, and WASM tests.
- [x] `External` confirmed absent from the language (no grammar/AST/Graph mention), so no accessor exists to add.
- [x] Fixed three adjacent defects found during skill verification, each with a regression test: D8 `Graph::absorb` now merges declared units/dimensions (`dimension_unit_tests`); D9 human parse summary labels (`cli_tests::test_parse_human_summary_counts`); D10 formatter contract-key quoting (`cli_tests::test_fmt_mapping_projection_round_trip`).
- [x] `domainforge-sea` skill re-verified against 0.17.0 CLI source plus live binary runs and updated (nonexistent `graph`/`units`/`explain`/`eval` removed; `import` corrected to sbvr|kg; `contract`/`envelope`/`test`-stub documented; all 29 project formats).
- [x] Full gates on the final tree. Evidence: `just all-tests` exit 0 (Rust 123 suites / 1230 passed / 0 failed; TS 198/198; Python 100%), `just wasm-test` 22/22, `cargo clippy -p domainforge-core --all-targets --features cli -- -D warnings` clean, `cargo fmt --all --check` clean, `git diff --check` clean.

## Repository Documentation Architecture Completed

## Verification contract and envelope follow-up (2026-09-24)

- [x] Flow verification obligations use canonical declaration occurrence references, including distinct duplicate flows. Evidence: `domainforge-core/src/application/verification_contract.rs`; `verification_contract_flow_keys_match_snapshot_occurrences` passed.
- [x] Capabilities advertise `verification-contract`, and an explicitly invalid `--registry` reports an error and exits 2. Evidence: `domainforge-core/src/cli/envelope.rs`; the focused envelope emit suite passed (18 tests).
- [x] Section 6 points to the CLI and identity references for detailed fields. The identity reference reflects all eleven serialized `DomainModelIdentity` fields, including optional `semantic_pack_set_hash`.
- [x] Validation: `mise exec rust@1.92.0 -- cargo test -p domainforge-core --features cli --test envelope_emit_tests` (18 passed), `mise exec rust@1.92.0 -- cargo fmt --all --check`, the focused documentation link assertions, and `git diff --check` passed.
- [ ] Automated review and graft index rebuild: CodeRabbit reported that review is disabled for this task; `graft build` could not run because the CLI is absent.

## Unified CEP / SXR Interface Remediation (2026-09-08)

Branch: `chore/release-please-commit-fix`. Ported Slice 0B additive components without merging, preserving PR #122 fixes.

- [x] Pre-port baseline golden snapshot captured. Evidence: `domainforge-core/tests/fixtures/envelope/golden/valid-basic.pre-port.stdout.json` byte-for-byte identical to unflagged `domainforge envelope` output.
- [x] Verification contract ported. Evidence: `domainforge-core/src/application/verification_contract.rs` and registered in `application/mod.rs`.
- [x] CEP envelope builders & F-03 omission-based failure synthesis implemented. Evidence: `domainforge-core/src/application/envelope.rs` emits `conformance_status: "conformant"`, `validation_status: "validated"`, omissions record for `representation_unavailable`, and `extensions.domainforge.model_validation_status: "invalid"` when D cannot be synthesized.
- [x] F-04 Typed DomainModelIdentity implemented. Evidence: `domainforge-core/src/application/envelope.rs` provides `DomainModelIdentity::canonical_digest()` over eleven serialized fields.
- [x] CLI flags & exit codes implemented. Evidence: `domainforge-core/src/cli/envelope.rs` supports `--emit` (`representation`, `cep`, `both`, `verification-contract`), `--capabilities`, `--pack`, `--registry`, `--default-namespace`, `--scope`, `--inline-threshold-bytes`, and `-o`.
- [x] Test suite passed. Evidence: `cargo test -p domainforge-core --test envelope_emit_tests --features cli` (16 passed) and `cargo test -p domainforge-core --lib` (381 passed).

## Repository Documentation Architecture (2026-09-02)

Delivered a comprehensive, self-contained technical knowledge system for DomainForge adhering to Diátaxis, DeepWiki, and Google Code Wiki principles.

- [x] Canonical high-level architecture documented. Evidence: `docs/architecture.md` (and root pointer `architecture.md`) with logical, runtime, dependency, and data transformation architectures, plus detailed Mermaid diagrams and prose analysis.
- [x] Documentation map and source map delivered. Evidence: `docs/documentation-map.md` (Diátaxis classification, audience needs, prerequisite tracks) and `docs/source-map.md` (symbol-level source traceability).
- [x] Layer 0 and Layer 1 orientation and mental model delivered. Evidence: `docs/index.md` (knowledge portal), `docs/orientation.md` (5-minute fast track), and `docs/mental-model.md` (deep conceptual abstractions and 3VL rationale).
- [x] Layer 3 Subsystem Guides completed (9 subsystems). Evidence: `docs/subsystems/` covers `parser-grammar.md`, `graph-store.md`, `policy-engine.md`, `units-dimensions.md`, `semantic-packs.md`, `authority-engine.md`, `application-contracts.md`, `projections-engine.md`, and `language-bindings.md`.
- [x] Layer 4 Execution Traces completed (5 workflows). Evidence: `docs/workflows/` covers `parse-and-validate.md`, `projection-generation.md`, `pack-lifecycle.md`, `application-resolution.md`, and `authority-evaluation.md`.
- [x] Layer 5 Explanations completed (5 deep rationales). Evidence: `docs/explanations/` covers `canonical-semantic-core.md`, `indexmap-determinism.md`, `three-valued-logic-rationale.md`, `adr-013-application-contract.md`, and `operator-family-design.md`.
- [x] Layer 6 Tutorials & Layer 7 How-Tos delivered. Evidence: `docs/tutorials/01-first-sea-model.md`, `02-multi-target-projection.md`, `03-building-signing-packs.md`; `docs/how-tos/add-projection-target.md`, `add-grammar-construct.md`, `configure-module-resolution.md`, `debug-policy-evaluations.md`.
- [x] Layer 8 Reference & Troubleshooting delivered. Evidence: `docs/reference/` (`dsl-grammar-reference.md`, `cli-reference.md`, `error-code-reference.md`, `primitives-api-reference.md`, `configuration-reference.md`) and `docs/troubleshooting.md`.
- [x] 100% relative markdown link verification passed. Evidence: automated PowerShell check traversed all markdown links in `docs/` and confirmed 0 missing targets.

## RDF instance/policy projection (2026-08-06)

Branch: `feat/rdf-instance-policy-projection`. No commits ahead of `main`; the
entire change is uncommitted worktree content.

Problem addressed: `KnowledgeGraph::from_graph` iterated only entities, roles,
resources, patterns, relations, and flows, so every authored `Instance` and
`Policy` was silently dropped from `model.ttl`, `model.jsonld`, and
`ontology.owl.ttl`.

- [x] Instance and policy triples emitted. Evidence: `domainforge-core/src/kg.rs`
  emits each instance as `sea:instance_<name>` typed `sea:EntityInstance`, linked
  by `sea:instanceOf`, with one `sea:<fieldName>` triple per field; each policy
  as `sea:policy_<name>` typed `sea:Policy` carrying expression, modality, kind,
  priority, rationale, and tags. Field keys and tags are sorted before emission
  because `Instance::fields` is a `HashMap` and unsorted iteration would break
  byte-identical output.
- [x] Ontology axioms emitted. Evidence:
  `domainforge-core/src/projection/rdf/ontology.rs` adds the `EntityInstance` and
  `Policy` classes, the `instanceOf` object property, the `expression`/
  `modality`/`priority` data properties, and named individuals — all gated on the
  model actually declaring instances or policies, so nothing appears
  speculatively.
- [x] Test suite added. Evidence:
  `domainforge-core/tests/turtle_instance_export_tests.rs`, 9 tests covering
  Turtle subjects, field datatypes, policies, ontology individuals, JSON-LD typed
  values, determinism across 8 reprojections, the graph-wide instance-name
  uniqueness invariant, distinct instances of different entities, and the
  no-instances model projecting unchanged.
- [x] Documentation updated. Evidence: `docs/specs/SDS-005-knowledge-graph-module.md`
  §4.2–§4.5 and `docs/rdf-projections.md`.
- [x] Corrected a doc/code contradiction left by the prior agent. The docs
  specified `sea:instance_<EntityType>_<name>`; the implementation emits
  `sea:instance_<name>`. The docs were corrected to match the code: the name
  alone is sufficient because `Graph::insert_entity_instance` rejects duplicate
  names graph-wide, and omitting the entity type means retyping an instance does
  not change its identity. The invariant is pinned by
  `instance_names_are_unique_graph_wide`.
- [x] Verified the SDS-005 §4.4 canonicalization claim against the code.
  `is_minted_node` (`domainforge-core/src/projection/rdf/mod.rs:71`) matches only
  `sea:flow_` and `sea:pattern_`, so instance and policy IRIs are never rewritten
  by `canonicalize_node_ids`.

### Deliberate non-goals in this change

- `ResourceInstance` is not projected. Its identity is `Uuid::new_v4()` rather
  than a content-derived `ConceptId`, so emitting it would break determinism. A
  content-derived identity is the prerequisite, not a projection change.
- Policies are stated, not lowered into SHACL. A partial translation would
  silently change what a policy means.
- `FieldType::EntityRef` emits the referenced key value as a literal, not an IRI,
  so an unvalidated model cannot produce dangling references.

### Gate evidence (2026-08-06)

| Gate | Result |
| ---- | ------ |
| `cargo fmt --check` | clean (rustfmt applied to the new code first) |
| `cargo clippy --all-targets --features cli -- -D warnings` | clean |
| `just rust-test` | pass |
| `just ai-validate` | pass |
| `just ts-test` | 197 pass / 0 fail |
| `just python-test` | 241 pass / 5 skipped |

Cross-binding evidence is meaningful only against freshly built artifacts: the
napi binding (`bun run build` in `domainforge-typescript/`) and the maturin
extension (`just python-setup`) were both rebuilt from current source before the
binding suites were run. That rebuild also resolved the stale-native-extension
Python golden mismatch recorded on 2026-08-03.

### Environment change

`bun` was absent, and the `nub` shim refuses npm while `bun.lock` exists, which
blocked the TypeScript gate entirely. bun 1.3.14 is now installed at
`~/.bun/bin/bun` with a guarded PATH block appended to `~/.bashrc`. Repository
dependencies were installed with `bun install --frozen-lockfile`; `bun.lock` and
`package.json` are unchanged.

## Prior state: SEA Interaction Interventions I1-I3 (2026-08-03)

- [x] Read repository instructions and the governing limitations report.
  Evidence: `AGENTS.md`, `.github/copilot-instructions.md`, and
  `.agents/reports/2026-08-03-sea-interaction-model-limitations-interventions.md`.
- [x] Verified the report against current parser, application contract, graph,
  module resolver, CLI, policy, binding, projection, and compatibility paths.
  Evidence: typed entity bodies are discarded at graph conversion; the
  filesystem CLI converts only the entry AST; policies expose resource
  `instances` but not entity instances. The report overstates existing
  instance-value validation: application resolution validates declarations and
  defaults, not concrete `Instance` values.
- [x] Established baseline. Evidence: `just ai-validate` passed; `just ts-test`
  passed 196/196; `just python-test` had one pre-existing cross-binding golden
  mismatch while all other Python tests passed, consistent with a stale native
  extension. That mismatch is resolved as of the 2026-08-06 rebuild above.
- [x] Recorded executable plan. Evidence:
  `.agents/plans/2026-08-03-sea-interaction-i1-i3.md`.
- [x] I1 typed graph contract and entity-instance validation implemented.
  Evidence: graph-owned entity/enum contracts plus focused validation tests for
  required and optional fields, supported constraints and scalar types, enums,
  typed references, key uniqueness, and dangling references.
- [x] I2 filesystem CLI closure implemented. Evidence: file-based parse,
  validate, and generic project commands use the existing deterministic module
  resolver over the transitive closure; focused tests prove imported
  same-namespace instances resolve.
- [x] I3 entity-instance policy bindings implemented. Evidence:
  `entity_instances` plural quantification and `entity_instance` singular
  bindings are covered by parsed and aggregate policy tests.
- [x] Focused, full, compatibility, projection, and faithful-fixture evidence
  complete. Evidence:
  `.agents/reports/2026-08-03-i1-i3-verification.md` and
  `evidence/latest/proof.{json,md}`. The original SEA Forge interaction model
  validates unchanged; the modular fixture accepts the formerly failing
  imported-instance shape and rejects its intentionally invalid variant.

## Prior state

Branch: `agent/projection-targets`. Accepted ADR-013 baseline: `1320fed`.
Milestone 0 implementation and remediation run through `56895f2`.

## Milestone 0 gate (2026-07-19)

- [x] Implemented the 15-task language plan. Evidence: commits `3550baf`
  through `e3e82a4` and the implementation paths named in
  `.agents/plans/2026-07-18-conversational-application-generator-m0-language.md`.
- [x] Completed adversarial human-gate review. Evidence:
  `.agents/reports/2026-07-19-m0-human-review-gate.md`.
- [x] Remediated the review findings. Evidence: commits `cc6036f` through
  `56895f2` cover patch hygiene, strict schemas, field semantics, shared
  identities, resolved envelope references, canonicalization, persisted
  metadata, diagnostics, resource budgets, compatibility evidence, semantic
  packs, and rebuilt cross-binding parity.
- [x] Human explicitly accepted Milestone 0. Evidence: repository maintainer
  instruction on 2026-07-19: “milestone 0 is done just accept it”. The gate
  disposition is recorded in the review report.
- [x] Drafted the required Milestone 1 public-contract settlement. Evidence:
  `docs/specs/ADR-014-application-review-and-approval-contract.md` fixes exact
  commands, artifact schemas, approval arguments, statement binding, statuses,
  exit codes, and pure-core/binding ownership.
- [ ] ADR-014 is accepted. Proposed pending explicit maintainer ratification;
  the executable plan cannot safely freeze its interfaces before this gate.
- [ ] Milestone 1 executable plan is complete and reviewed. Blocked on ADR-014;
  expected artifact is
  `.agents/plans/2026-07-19-conversational-application-generator-m1-review.md`.
- [ ] Milestone 1 implementation is accepted. Not started; implementation may
  begin only from the completed Milestone 1 plan.

## Gate status

Human Gates A and B and the Milestone 0 human gate are CLOSED. The RDF
instance/policy projection change passes every automated gate but has NOT been
reviewed or committed; maintainer review is the open action. Milestone 1
contract settlement remains proposed in ADR-014, with plan finalization and
implementation waiting on explicit ratification. Later milestone gates remain
closed to self-approval: the repository maintainer must explicitly accept each
reviewed milestone.

## Commit workflow blocker resolved (2026-09-02)

- [x] Made the required scoped commit format explicit. Evidence: `CONTRIBUTING.md` now states that scopes are mandatory, and `docs/governance.md` matches `commitlint.config.cjs` by documenting kebab-case scopes without claiming a static allowlist.
- [x] Made hook failures actionable. Evidence: `scripts/run_commitlint.sh` now prints the required `<type>(<scope>): <subject>` format and an example after commitlint rejects a message.
- [x] Validated the fix. Evidence: `sh -n scripts/run_commitlint.sh` passed; `docs: update guide` failed with the new hint; `docs(repo): update guide` passed; `git diff --cached --check` passed; 135 relative links in staged Markdown resolved.

## Unified CEP / SXR Interface Remediation (2026-09-08)

- [x] Ported Slice 0B additive envelope modules (`application/envelope.rs`, `application/verification_contract.rs`, fixtures, tests) onto maintained tip `chore/release-please-commit-fix`. Evidence: `domainforge-core/src/application/verification_contract.rs` created; `domainforge-core/tests/envelope_emit_tests.rs` (16 passing tests).
- [x] Implemented `--emit <representation|cep|both>` CLI option and corresponding payload synthesis. Evidence: `domainforge-core/src/cli/envelope.rs`, `docs/reference/cli-reference.md`.
- [x] Implemented omission-based failure synthesis (F-03). When model has validation errors, emitted CEP envelope omits statements rather than populating non-conformant/speculative ones. Evidence: `envelope_emit_tests::test_envelope_emit_cep_omits_statements_on_validation_failure`.
- [x] Implemented typed `DomainModelIdentity::canonical_digest()` (F-04). Returns `Option<&str>` to eliminate unwrap risks on unverified identities. Evidence: `domainforge-core/src/application/verification_contract.rs`.
- [x] Verification: `cargo test -p domainforge-core` passes with 381 library unit/integration tests and 16 envelope emit integration tests.

## Devin free-tier repository onboarding (2026-10-13)

- [x] Authored .devin/wiki.json with 30 explicit pages and 12 source-guided repo notes; all page titles/parents/length limits statically validated. This is a navigation plan, not generated documentation or new product capability.
- [x] Authored .devin/blueprint.yaml for Rust 1.99.0, Python 3.12, Node 24, Bun and just; the maintenance phase fetches locked Cargo, Bun and Python dependencies. Existing AGENTS.md, specs, Rust core, CI and proof definitions remain untouched.
- [ ] Devin-side generation of the Low/free DeepWiki and Git-backed environment snapshot has not been observed. Running the local Dockerless build and complete CI matrix is outside this documentation-only change.
- Evidence: GitHub PR for chore/devin-free-onboarding and the committed .devin/ files; only static validation and GitHub CI (when completed) can be claimed, not live Devin execution.
