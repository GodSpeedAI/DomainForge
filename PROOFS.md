# DomainForge Proof Ledger

Every material public claim DomainForge makes, with an honest status and a
copy-paste-runnable proof command. Regenerate the machine-readable evidence
with:

```bash
just prove
```

which writes `evidence/latest/proof.json` and `evidence/latest/proof.md`.

## Status legend

- **proven** — verified end-to-end by a command in this table, with an
  independent/native validator or a byte-level equality check.
- **partial** — a real check runs, but the validator is a structural
  substitute (not the ecosystem-native tool) or the guarantee is scoped;
  the remaining gap is named.
- **planned** — no code exists yet; the subsystem is named.
- **blocked** — provable, but a required external tool is absent locally;
  the exact failing command and unblocking action are recorded.

The `Status` column below is column 3 of each row and is parsed verbatim into
`proof.json.claim_status`; keep that column honest.

## Claims

| Claim ID | Claim | Status | Source | Fixture | Proof command | Independent validator | Evidence artifact | Failure condition | Remaining gap |
|---|---|---|---|---|---|---|---|---|---|
| CLAIM-LANGUAGE-PARSE | SEA models parse and validate; invalid models are rejected | proven | `domainforge-core/src/parser/`, `src/cli/mod.rs` | `fixtures/projection_cell/basic/model.sea`, `fixtures/projection_cell/invalid/*.sea` | `bash scripts/prove/language.sh` | DomainForge parser + validator (negative fixtures must fail) | `evidence/latest/fragments/language.json` | A positive fixture fails validate, or a negative fixture is accepted | none |
| CLAIM-FORMAT-STABILITY | `format` round-trips and is idempotent | partial | `domainforge-core/src/parser/` (formatter) | `fixtures/projection_cell/format_stable.sea` | `bash scripts/prove/language.sh` | parse→format→parse closure + byte fixpoint on the round-trip-clean subset | `evidence/latest/fragments/language.json` | Formatted output fails to re-parse, or format is not a byte fixpoint | Formatter output for `Relation`/`Policy`/`Metric` bodies does not re-parse (`Policy … as:` expression rejected at parse). Fix the formatter/grammar so the full projection-cell fixture round-trips |
| CLAIM-DETERMINISM | The canonical semantic projection is deterministic (byte-identical across isolated runs) | proven | `domainforge-core/src/projection/` (rdf), `--created-at` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/prove/canonical.sh` | `diff -r` byte comparison across two isolated runs (source-path provenance comment excluded, documented) | `evidence/latest/fragments/canonical.json` | RDF output differs across two pinned runs | none for RDF |
| CLAIM-PACK-FINGERPRINT | `pack build` produces a stable `meaning_fingerprint` / `source_graph_hash` for a fixed source | proven | `domainforge-core/src/cli/pack.rs`, `src/primitives/flow.rs` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/prove/canonical.sh` (builds the pack twice in isolated dirs) | JSON field comparison across two independent rebuilds | `evidence/latest/fragments/canonical.json` (`fingerprint_stable_across_rebuilds`) | `source_graph_hash` or `meaning_fingerprint` differs across rebuilds | none — Flow concept IDs are now content-addressable (`ConceptId::from_concept` over namespace+resource+from+to+quantity, `primitives/flow.rs`), replacing the prior `Uuid::new_v4()` |
| CLAIM-DRIFT | `pack diff` detects declared-model drift with typed classification and no false positives | proven | `domainforge-core/src/semantic_pack/diff.rs` | `fixtures/projection_cell/basic/model.sea` (+ in-test mutation) | `bash scripts/prove/drift.sh` | `pack diff --format json` (matching diff empty; drift diff non-empty `breaking`; nonzero exit gates CI) | `evidence/latest/fragments/drift.json` | Matching diff reports entries, or drift diff is empty / lacks a `breaking` entry | none for entity-rename drift; full per-projection mutation-impact is `planned` |
| CLAIM-ROUNDTRIP-CALM | A model round-trips through CALM without losing representable primitives | proven | `domainforge-core/src/calm/`, `tests/roundtrip_cell_tests` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/roundtrip-cell.sh` | `cargo test --test roundtrip_cell_tests` structural-primitive comparison | `evidence/latest/fragments/roundtrip.json` | Entity/resource/flow identity or names lost on re-import | none |
| CLAIM-ROUNDTRIP-KG | A model round-trips through the KG (RDF/Turtle) projection preserving entity/resource/flow counts | proven | `domainforge-core/src/kg.rs` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/prove/roundtrip.sh` | `project --format kg` → `import --format kg` count comparison | `evidence/latest/fragments/roundtrip.json` | Imported counts differ from source | none |
| CLAIM-PROJ-TLA | TLA+ projection is model-checkable by the real TLA+ toolchain | proven | `src/projection/tla/`, `scripts/verify/projection-targets/tla.sh` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/tla.sh` | SANY parse + TLC model-check (pinned `tla2tools.jar` v1.8.0) — **native in CI**; structural-only where Java absent | `evidence/latest/fragments/projections.json` | SANY/TLC report errors | Local runs without Java degrade to structural (`native:false`); CI runs it natively |
| CLAIM-PROJ-ASYNCAPI | AsyncAPI projection validates against the official AsyncAPI schema | proven | `src/projection/asyncapi/`, `tests/asyncapi_spec_validation_tests.rs` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/asyncapi.sh` | Official vendored AsyncAPI 3.0.0 JSON Schema validation | `evidence/latest/fragments/projections.json` | Document fails schema validation | none |
| CLAIM-PROJ-CLOUDEVENTS | CloudEvents projection emits valid CloudEvents 1.0 JSONL | proven | `src/projection/cloudevents/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/cloudevents.sh` | Strict JSONL parse + RFC 3339 `time` validation | `evidence/latest/fragments/projections.json` | A line fails JSON parse or `time` is not RFC 3339 | none |
| CLAIM-PROJ-DAGGER | Dagger projection emits importable Python | proven | `src/projection/dagger/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/dagger.sh` | `python3 -m py_compile` | `evidence/latest/fragments/projections.json` | Generated `main.py` fails to compile | Full `dagger develop` needs the daemon (containerized CI) |
| CLAIM-PROJ-DEVBOX | Devbox projection emits a valid JSONC manifest | partial | `src/projection/devbox/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/devbox.sh` | JSONC structural check | `evidence/latest/fragments/projections.json` | Manifest fails JSONC parse | Native `devbox` CLI validation deferred (CLI absent locally) |
| CLAIM-PROJ-CEDAR | Cedar projection emits a valid schema + scoped permits | partial | `src/projection/cedar/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/cedar.sh` | Strict JSON schema parse + scoped-permit structural check | `evidence/latest/fragments/projections.json` | Schema fails JSON parse or a permit is unscoped | `cedar validate-schema` deferred (Cedar CLI absent); SEA policy obligations not yet projected as forbid/when |
| CLAIM-PROJ-GAUGE | Gauge projection emits a valid spec (one scenario per flow) | partial | `src/projection/gauge/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/gauge.sh` | Structural check (H1, angle-bracket sanitization) | `evidence/latest/fragments/projections.json` | Scenario count or structure wrong | `gauge validate` deferred (Gauge CLI absent locally) |
| CLAIM-PROJ-ALLOY | Alloy projection emits a model with one fact per flow | partial | `src/projection/alloy/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/alloy.sh` | Structural check (scope scales with flow count) | `evidence/latest/fragments/projections.json` | Fact count does not match flow count | Alloy CLI parse deferred (Alloy CLI absent locally) |
| CLAIM-PROJ-DOMAIN-CODE | Domain code projections (Python/TS/Rust) compile and pass their generated tests | proven | `src/projection/domain/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/domain-python.sh` (and `-typescript`, `-rust`) | `mypy --strict`+`unittest`, `tsc --noEmit`, `cargo check`+`cargo test` | `evidence/latest/fragments/projections.json` | Generated code fails to compile or its tests fail | none (requires python3/mypy, node/tsc, cargo present) |
| CLAIM-PROJ-PROTOBUF | Protobuf projection lints clean with `buf` | blocked | `src/projection/` (protobuf) | `fixtures/projection_cell/basic/model.sea` | `cargo run -q -p domainforge-core --features cli -- project --format protobuf --buf-lint fixtures/projection_cell/basic/model.sea /tmp/out` | `buf lint` | (not generated — blocked) | `buf lint` reports violations | `buf` CLI absent locally; install buf or run in CI |
| CLAIM-PROJ-CALM | CALM projection is architecture-as-code and round-trippable | proven | `src/calm/` | `fixtures/projection_cell/basic/model.sea` | `bash scripts/verify/projection-targets/roundtrip-cell.sh` | CALM round-trip structural comparison | `evidence/latest/fragments/roundtrip.json` | Round-trip loses primitives | none |
| CLAIM-BINDING-PARITY | Python/TypeScript/WASM bindings run the same core | proven | `domainforge-python/`, `domainforge-typescript/`, `src/wasm/` | crate test suites | `just all-tests` | Per-language test suites over the shared Rust core | (test suites) | A binding test suite fails | Projection export is CLI-only per the status doc (bindings do not expose projections) |
| CLAIM-VERSION-CONTRACT | Enterprise verification (fmt/clippy/tests/audit) passes | partial | `justfile` (`enterprise-verify`) | whole workspace | `just enterprise-verify` | cargo fmt/clippy/test + audit | (CI logs) | Any gate fails | Requires `cargo-audit`/`cargo-deny`; not part of `just prove` |
| CLAIM-RUNTIME-OBSERVATION | DomainForge ingests runtime observations ("semantic exhaust") | planned | none (subsystem does not exist) | none | none | none | none | — | Build the observation-ingestion subsystem |
| CLAIM-REVERSE-MINING | DomainForge reverse-mines an observed model from behavior | planned | none (subsystem does not exist) | none | none | none | none | — | Build reverse semantic mining + observed-vs-declared diff |
| CLAIM-GENERATED-APPS | DomainForge generates running applications/services | planned | none (domain-code projection stops at the ports, is not a running app) | none | none | none | none | — | Build runtime adapters + deployment below the generated ports |

## Notes on honesty

- Native validators actually executed on a typical local machine (no Java, no
  buf, no cedar/gauge/alloy CLIs): AsyncAPI (official schema), CloudEvents
  (JSONL+RFC3339), Dagger (`py_compile`), domain-python/typescript/rust
  (mypy/tsc/cargo), CALM & KG round-trip (`cargo test` / CLI import). TLA+ runs
  natively **in CI** (Java + pinned `tla2tools.jar`) and degrades to structural
  checks locally.
- Structurally substituted (native tool deferred, marked `native:false`):
  Cedar, Gauge, Alloy, Devbox.
- Blocked locally: Protobuf `buf lint` (no `buf`).
