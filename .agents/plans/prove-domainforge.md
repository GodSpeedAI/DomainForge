# DomainForge Self-Proving Repository Execution Contract (v2, repo-grounded)

Your job is to make the DomainForge repository prove its central claims through
executable, reproducible evidence.

Do not stop after writing documentation or generating example artifacts.
Continue inspecting, implementing, testing, correcting, and regenerating until
`just prove` passes from a clean checkout and the evidence pack honestly
supports every claim marked `proven`.

## Ground Truth (verified 2026-07-10 — trust this, do not re-derive it)

You are NOT starting from scratch. The repository already contains:

**CLI** (binary `domainforge`, built from `domainforge-core/src/bin/domainforge.rs`,
requires `--features cli`). Invocation pattern used by every existing gate script:

```bash
cargo run -q -p domainforge-core --features cli -- <subcommand> ...
```

Subcommands (from `domainforge-core/src/cli/mod.rs`): `parse`, `validate`,
`import` (formats: `sbvr`, `kg`), `project`, `format`/`fmt`, `test`,
`validate-kg`, `normalize`, `registry`, `authority`, `pack`.

**Projection targets** (`domainforge project --format <fmt>`): `calm`, `kg`
(RDF Turtle/XML by output extension), `protobuf`/`proto` (with `--buf-lint`,
`--buf-breaking` flags), `cloudevents`, `asyncapi`, `cedar`, `devbox`,
`dagger`, `gauge`, `alloy`, `tla`, `rdf`, `bpmn`, `cmmn`, `archimate`,
`otel-semconv`, `baml`, `dspy`, `zenml`, and `ai-*` recipe formats. Run
`cargo run -q -p domainforge-core --features cli -- project --help` before
scripting to confirm exact ValueEnum spellings.

**Determinism support already exists**: `--created-at <RFC3339>` pins the
timestamp for byte-identical output on all directory-output projections
(`ai-*`, `rdf`, `bpmn`, `cmmn`, `archimate`, `otel-semconv`, `baml`, `dspy`,
`zenml`). `--seed` pins AI-recipe sampling.

**Semantic identity/diff already exists** (declared-vs-declared only):
`domainforge pack build` computes `source_graph_hash` and
`meaning_fingerprint` (`domainforge-core/src/cli/pack.rs`, `compute_graph_hash`);
`domainforge pack diff` produces typed diffs (`DiffClassification` in
`domainforge-core/src/semantic_pack/diff.rs`).

**Verification gates already exist** in `scripts/verify/projection-targets/`:
`cloudevents.sh`, `asyncapi.sh`, `devbox.sh`, `dagger.sh`, `cedar.sh`,
`gauge.sh`, `alloy.sh`, `tla.sh`, `roundtrip-cell.sh`, orchestrated by
`all.sh`, all running against `fixtures/projection_cell/basic/model.sea`.
Native-validator status is documented in
`docs/projection-target-implementation-status.md`:

- TLA+: real toolchain (SANY parse + TLC model-check, pinned tla2tools.jar v1.8.0; runs fully in CI, degrades to structural checks locally without Java).
- AsyncAPI: official vendored 3.0.0 JSON Schema validation (`domainforge-core/tests/asyncapi_spec_validation_tests.rs`).
- Dagger: `python3 -m py_compile`. CloudEvents: strict JSONL + RFC 3339. Cedar/Devbox/Gauge/Alloy: structural checks only (native CLIs deferred).
- Round-trip cell: CALM export → import → structural-primitive comparison (`cargo test -p domainforge-core --features cli --test roundtrip_cell_tests`).

**Justfile** (root): recipes include `ai-validate` (default), `rust-test`,
`cli-test`, `cli-validate`, `cli-workflow`, `all-tests`, `audit`,
`enterprise-verify`, plus `ci-*` recipes. There is NO `prove` recipe yet.

**CI** (`.github/workflows/ci.yml`): jobs `lint`, `test-rust` (matrix),
`test-python`, `test-typescript`, `test-integration`, `test-wasm`,
`verify-projection-targets` (line ~854, runs `all.sh` with Java + pinned
tla2tools.jar).

**Fixtures** live at `fixtures/<family>/<case>/` (e.g.
`fixtures/projection_cell/basic/model.sea`, `fixtures/rdf/basic/domain/`,
`fixtures/semantic_packs/acme_procurement/`). There is NO numbered
`00-…/13-…` fixture taxonomy; do not invent one.

**Does NOT exist — no code anywhere in the repo**: runtime observation
ingestion, "semantic exhaust", observed-model construction, reverse semantic
mining, observed-vs-declared diff, generated runnable applications/services,
telemetry emission, agentic hooks, `PROOFS.md`, `evidence/`, `just prove`.

## Mission (rescoped to what is provable)

Make the repository prove, with executable evidence, that DomainForge can:

1. represent meaningful organizational systems in `.sea`;
2. produce one deterministic canonical semantic representation (graph hash + meaning fingerprint);
3. preserve declared invariants across multiple native projection targets;
4. validate projected artifacts with the target ecosystem's native validator where one is wired in;
5. round-trip a model through a projection and back without losing representable primitives (CALM cell today);
6. detect declared-model drift through typed semantic diff (`pack diff`);
7. reproduce all evidence from a fresh clone with one root command.

Claims about **runtime observation, reverse semantic mining of observed
behavior, and generated running applications** are NOT provable from current
code. They MUST appear in `PROOFS.md` classified as `planned`, with the
missing subsystem named. Do not build those subsystems under this contract,
and do not fake them.

## Non-Negotiable Completion Condition

From a clean checkout:

```bash
just prove
```

must run the full proof suite, exit nonzero if any required proof fails, and
produce:

```text
evidence/latest/proof.json
evidence/latest/proof.md
```

Evidence must be generated by the run itself. `evidence/` must be
git-ignored except for a `README.md` explaining how to regenerate it.

## Operating Rules

1. Reuse the existing gate scripts, tests, and just recipes; wrap, don't rewrite.
2. Treat `.sea` and the canonical graph as the owners of meaning; generated artifacts are deterministic projections.
3. Use the target ecosystem's native validator wherever one is already wired in (TLA+, AsyncAPI, buf, py_compile). Where it is not (Cedar, Gauge, Alloy, Devbox), record the structural check as the validator and classify the claim `partial`, exactly as `docs/projection-target-implementation-status.md` already does.
4. Never mark a claim `proven` solely because an internal unit test passes, and never weaken a failing assertion to go green.
5. Never fabricate evidence or validator output. Do not silently reduce a public claim's scope — reclassify it as `partial`, `planned`, or `blocked`.
6. Prefer small complete proof loops over new scope.
7. New shell scripts go in `scripts/prove/`, follow the existing gate-script conventions (`set -euo pipefail`, `REPO_ROOT` resolution, mktemp + trap cleanup — copy the header of `scripts/verify/projection-targets/tla.sh`).

## Phase 1: Claim Audit → PROOFS.md

Read `README.md`, `docs/index.md`, `docs/projection-families.md`,
`docs/projection-target-implementation-status.md`, and the per-target docs in
`docs/`. Create `PROOFS.md` at the repo root with one row per material public
claim:

| Field | Required content |
| --- | --- |
| Claim ID | Stable identifier (e.g. `CLAIM-DETERMINISM`) |
| Claim | Exact claim being made |
| Status | proven, partial, planned, or blocked |
| Source | Source files implementing it |
| Fixture | Fixture exercising it |
| Proof command | Exact reproducible command |
| Independent validator | Native validator, or "structural check only" |
| Evidence artifact | Path under `evidence/latest/` |
| Failure condition | Observable condition that disproves the claim |
| Remaining gap | Work needed if not proven |

Minimum claim set to classify: language parses/validates, canonical
determinism, each projection target in the status doc (copy its honest
per-target validator column), CALM round-trip, pack fingerprint stability,
pack diff drift detection, binding parity (Python/TS/WASM run the same core —
proof command: `just all-tests`; note projections are CLI-only per the status
doc), runtime observation (`planned`), reverse mining (`planned`), generated
running applications (`planned`).

**Done when**: `PROOFS.md` exists, every row's proof command is copy-paste
runnable, and no claim is unclassified.

## Phase 2: Root Proof Harness (`just prove`)

Add to the root `justfile`:

```just
prove: prove-language prove-canonical prove-projections prove-drift prove-evidence

prove-language:      # parse + validate + format stability on fixtures
prove-canonical:     # determinism: two isolated runs, byte compare
prove-projections:   # bash scripts/verify/projection-targets/all.sh
prove-drift:         # pack build + pack diff fixtures
prove-evidence:      # bash scripts/prove/collect-evidence.sh
```

Rules:

- `prove-projections` is exactly `bash scripts/verify/projection-targets/all.sh` — do not duplicate its logic.
- Each `prove-*` recipe must be independently runnable and fail nonzero on any error.
- Reuse the `cargo` variable already defined at the top of the justfile.
- `prove` must also run `just rust-test` (or depend on a recipe that does) so unit/integration tests gate the evidence.

**Done when**: `just prove` runs end-to-end locally and each sub-recipe fails
nonzero when its inputs are deliberately broken (test this once with a
temporary syntax error in a fixture, then revert).

## Phase 3: Language and Canonical-Determinism Proof

### 3a. prove-language

Script `scripts/prove/language.sh`:

1. For every `model.sea` and `*.sea` under `fixtures/`: run `domainforge parse` and `domainforge validate`; all must succeed except files under a directory named `invalid/` (create `fixtures/projection_cell/invalid/` with at least 3 syntactically/semantically broken `.sea` files if no negative fixtures exist — check `domainforge-core/tests/` first; if negative cases already exist as Rust tests, reference those instead and keep the script to positive fixtures).
2. Format stability: `domainforge format` the projection-cell fixture to a temp file, parse both, and assert `domainforge parse` output is identical (parse → format → parse equivalence). If `parse` output embeds file paths, strip them with `sed` before comparing and document the exclusion.

### 3b. prove-canonical

Script `scripts/prove/canonical.sh`:

1. Copy `fixtures/projection_cell/basic/model.sea` into two fresh `mktemp -d` dirs.
2. In each, run `domainforge pack build` (check `pack --help` for exact args) and one directory-output projection with pinned inputs, e.g.:

   ```bash
   cargo run -q -p domainforge-core --features cli -- project --format rdf \
     --created-at 2026-01-01T00:00:00Z model.sea out/
   ```

3. Assert: `source_graph_hash` and `meaning_fingerprint` identical across both runs; projection output directories byte-identical (`diff -r`).
4. Document every exclusion from byte-determinism in `PROOFS.md` (there should be none once `--created-at` is pinned; if a target still differs, that is a real finding — fix or classify `partial`, do not exclude silently).

**Done when**: both scripts pass, and the determinism script fails if you
remove `--created-at` from a timestamp-embedding target (verify once, revert).

## Phase 4: Projection Conformance Contracts

For each of the 9 gated targets plus `calm`, `kg`, `protobuf`, add a contract
file `scripts/verify/projection-targets/contracts/<target>.yaml`:

```yaml
projection_contract:
  target: tla
  status: proven            # copy honestly from the status doc
  preserves: [entity_identity, flow_actions, type_invariant]
  intentionally_collapses: [policy_execution_semantics]
  prohibited_losses: [source_element_identity]
  deterministic_output_required: true
  target_validator:
    command: "SANY parse + TLC model-check via scripts/verify/projection-targets/tla.sh"
    native: true            # false for structural-check-only targets
```

Fill `preserves`/`collapses` by reading each gate script and projector module
(`domainforge-core/src/projection/<target>/mod.rs`) — the gate scripts'
header comments already state what each gate proves. `status` must match the
validator reality: `proven` only for TLA+ and AsyncAPI today; `partial` for
structural-check-only targets. Do NOT install new target CLIs (Cedar, Gauge,
Alloy) under this contract; upgrading those gates is listed in `PROOFS.md`
as the remaining gap.

**Done when**: every gated target has a contract file, and a check in
`prove-projections` (extend `all.sh` or add a small script) fails if a gate
script exists without a contract or vice versa.

## Phase 5: Round-Trip and Drift Proof (prove-drift)

### 5a. Round-trip (already exists — wire it in)

`scripts/verify/projection-targets/roundtrip-cell.sh` already proves the CALM
round-trip. Additionally add a KG round-trip if cheap: `project --format kg`
to `.ttl`, then `import --format kg` and compare entity/resource/flow counts
against the source graph (the import command prints them). If the `shacl`
feature is required and not in the default build, note that and skip with an
explicit `partial` classification rather than a silent pass.

### 5b. Declared-drift via pack diff

Script `scripts/prove/drift.sh`:

1. `pack build` the projection-cell fixture → pack A.
2. Copy the fixture, make one semantic mutation with `sed` (rename one entity — pick an entity name actually present in `fixtures/projection_cell/basic/model.sea`; read the file first), `pack build` → pack B.
3. Assert `pack diff A B` reports a non-empty typed diff and the two `meaning_fingerprint`s differ.
4. Matching case: `pack build` the unmutated fixture twice; assert `pack diff` reports no semantic change and fingerprints match.
5. Staleness: assert that `pack diff` between the committed fixture's fresh pack and a pack built from the mutated copy is detected — this is the grounded substitute for "stale generated artifact detection". Full mutation-impact testing across all projections is `planned` in `PROOFS.md`.

**Done when**: drift script demonstrates one correct matching diff and one
correct drift diff, both machine-checked (grep/jq on `pack diff --json`
output if available — check `pack diff --help`).

## Phase 6: Evidence Pack (prove-evidence)

Script `scripts/prove/collect-evidence.sh` runs LAST and writes
`evidence/latest/proof.json` + `proof.md`. Build it with bash + `jq` (jq is
acceptable to require; check for it and fail with a clear message if absent).

`proof.json` minimum schema:

```json
{
  "schema_version": "1.0.0",
  "domainforge_version": "<from cargo metadata or --version>",
  "commit": "<git rev-parse HEAD>",
  "generated_at": "<date -u>",
  "clean_worktree": true,
  "language": {"fixtures_parsed": 0, "fixtures_validated": 0, "negative_fixtures_rejected": 0},
  "canonical_determinism": {"passed": false, "runs_compared": 2, "graph_hash": ""},
  "projection_gates": {"<target>": {"passed": false, "validator": "", "native": false}},
  "roundtrip": {"calm": {"passed": false}},
  "drift": {"matching_diff_correct": false, "drift_diff_correct": false},
  "claim_status": {"<CLAIM-ID>": "proven|partial|planned|blocked"},
  "overall_result": "failed"
}
```

Mechanism: each earlier `prove-*` script writes a small JSON fragment to
`evidence/latest/fragments/<name>.json` as it runs; `collect-evidence.sh`
merges fragments with `jq -s`, cross-checks `claim_status` against
`PROOFS.md` statuses, renders `proof.md` (a readable summary of what was
proved, how, with which validators, what remains partial/planned, and
reproduction instructions), and sets `overall_result`.

Add `evidence/` to `.gitignore` (keep `evidence/README.md` tracked, explaining
`just prove` regenerates it).

**Done when**: `just prove` produces both files, `overall_result` is
`"passed"`, and deleting `evidence/` then re-running reproduces them.

## Phase 7: CI as Auditor

Extend `.github/workflows/ci.yml` with ONE new job `prove` (do not create the
14-job matrix from earlier drafts — the existing jobs already cover lint,
per-language tests, WASM, integration, and projection gates):

- `needs: lint`, ubuntu-latest, installs Rust + just + Java (copy the Java/tla2tools steps from the existing `verify-projection-targets` job so the TLA+ gate runs natively).
- Runs `just prove`.
- Uploads `evidence/latest/` as a workflow artifact.
- Fails if `overall_result != "passed"` (the exit code already guarantees this; the artifact upload must use `if: always()`).

Leave `verify-projection-targets` in place (it is referenced by ADR-011) —
`prove` may call the same `all.sh`; duplication of ~2 CI minutes is acceptable.

**Done when**: the job is green on this branch's PR, and the uploaded artifact
contains a `proof.json` with `overall_result: "passed"`.

## Phase 8 (optional, only if Phases 1–7 are complete): Flagship Fixture

Add `fixtures/payment_authorization/basic/model.sea` modeling Customer,
Checkout, PaymentProcessor, Payment, a PaymentAuthorization flow, and an
authorization-before-capture policy — using ONLY constructs the SEA grammar
supports today (mirror the constructs used in
`fixtures/projection_cell/basic/model.sea`; if policy syntax there is thin,
check `fixtures/semantic_packs/acme_procurement/` and the `domainforge-sea`
skill docs). Wire it into `prove-language`, `prove-canonical`, and project it
to `kg`, `protobuf` (with `--buf-lint` if `buf` is installed; otherwise
record `buf` as a blocked native validator with the exact failing command),
and `cloudevents`. No runtime/observation claims — those stay `planned`.

## Required Minimum Settlement (v1)

`just prove` from a fresh clone must demonstrate:

```text
one canonical .sea source (fixtures/projection_cell/basic/model.sea)
byte-identical canonical determinism across two isolated runs
all nine existing projection gates passing
at least two natively validated projections (TLA+ via SANY/TLC, AsyncAPI via official schema)
one round-trip proof (CALM cell)
one correct matching semantic diff and one correct drift diff (pack diff)
PROOFS.md with every claim honestly classified
machine-readable + human-readable evidence pack, regenerated by the run
CI job that runs it all and archives the evidence
```

## Anti-Cheating Conditions

The following do not count as proof:

```text
handwritten or committed evidence JSON
tests that mock the behavior being claimed end to end
internal validation recorded as "native" when it is a structural check
a README statement with no PROOFS.md row
a claim marked proven whose proof command does not run from a fresh clone
disabled or softened failing assertions
claiming runtime observation, reverse mining, or generated-app execution in any status other than planned
```

## Blocker Policy

When blocked (missing tool, credential, platform): do not fabricate. Mark the
claim `blocked` in `PROOFS.md`, record the exact failing command and its real
output in the evidence pack, state the smallest unblocking action, and
complete all unaffected proofs. Known expected blockers: `buf`, `cedar`,
`gauge`, `alloy` CLIs absent locally; Java absent locally (TLA+ gate degrades
by design — record `native: false` for that run and note CI runs it natively).

## Final Response Required From You

Return, precisely:

```text
1. Exact claims proved (with PROOFS.md IDs).
2. Exact claims partial, planned, or blocked, each with its gap.
3. Files added or changed.
4. Commands used.
5. Native validators actually executed vs structurally substituted.
6. Evidence-pack paths.
7. CI job added and its status.
8. Known technical debt.
9. Reproduction instructions from a fresh clone.
10. The next smallest proof that would materially strengthen DomainForge
    (expected answer: wiring one deferred native validator, e.g. cedar
    validate-schema, or building the observation-ingestion subsystem).
```

You are done only when `just prove` passes from a fresh clone and the
generated evidence pack honestly supports the claims marked `proven`.
