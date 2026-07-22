# Agent Task: Add AI Learning Projections to DomainForge

> Revision 3 (2026-07-02). Adversarially reviewed against the actual codebase.
> Changes from v1: CLI grammar corrected to the real `project --format` interface;
> answer keys computed by the existing authority resolver instead of parallel
> template logic; recipes are JSON (no new YAML dep); "semantic dedup" replaced
> with deterministic normalized-tuple dedup; scope cut to one thin slice.
> Changes from v2: the CEP spec was located at `/home/sprime01/projects/cep/spec/`
> (CEP-0008 semantic envelope, CEP-0009 benchmark model) — the CEP eval projection
> now conforms to it instead of self-defining a schema, and RealityGapBench is
> reclassified as first follow-up (spec-sanctioned `mutation_generation`) rather
> than deferred-indefinitely.

## Objective

Extend DomainForge so `.sea` models can compile into **AI Learning Projections**:

```text
AI Learning Projections
├── LLM Dataset Projection        (instruction/chat JSONL seed data)
├── Graph ML Dataset Projection   (graph learning exports)
└── CEP Evaluation Dataset Projection (benchmark cases with answer keys)
```

Product claim this enables:

> DomainForge does not teach AI from documents. It compiles organizational
> reality into machine-learnable and machine-evaluable representations.

The implementation must be deterministic, source-grounded, and validation-first.
Do **not** add LLM calls inside DomainForge. Downstream systems consume these
artifacts for LoRA training, GNN training, eval runs, or agent testing.

---

## Codebase Facts (verified — build on these, do not re-derive)

These were confirmed against the repo at revision `a44d88d`:

1. **Core is Rust.** `domainforge-core/` is the engine; Python/TS/WASM are bindings.
   New projection logic goes in `domainforge-core/src/`.
2. **Real CLI grammar** (`domainforge-core/src/cli/project.rs`):
   `domainforge project --format <calm|kg|protobuf> [flags] <input> <output>`.
   `--format` is a clap `ValueEnum` (`ProjectFormat`); input/output are positional.
   There is no `--target` or `--out`. Extend `ProjectFormat`, do not invent a
   parallel flag scheme.
3. **Directory-output precedent exists**: protobuf `--multi-file` mode writes into
   an output directory and uses `crate::projection::protobuf::validate_output_path`
   for path-traversal safety. Reuse both patterns for the new dir-emitting targets.
4. **Projection pattern**: each target is an engine module under
   `domainforge-core/src/projection/` (see `protobuf.rs` / `ProtobufEngine`) that
   consumes a parsed `Graph` (`parse_to_graph_with_options`). The existing
   `ProjectionExporter` trait is entity/flow-granular and is NOT a good fit —
   follow the engine pattern (take `&Graph`, produce artifacts) instead.
5. **The authority module is the grounding engine.**
   `domainforge-core/src/authority/` already provides `AuthorityRequest`
   (actor + operation + resource + context), `FinalDecision { Allow, Deny,
   Escalate }`, `ThreeValuedResult` / `Unknown` handling, fact envelopes, and
   decision traces. **All authorization answers/labels/answer keys MUST be
   computed by invoking this resolver**, never by re-implementing policy logic
   in templates. This is what makes artifacts mechanically traceable.
6. **Canonical hashing exists**: `semantic_pack/canonical_json.rs`. Reuse it for
   `source_model_hash` — do not add a new canonicalization scheme.
7. **Dependencies**: `serde_json` is present; `serde_yaml` and `csv` are NOT.
   Recipes are JSON in v0. CSV emission is trivial hand-rolled writing (values
   are our own identifiers; quote-escape defensively anyway).
8. **Fixtures/tests conventions**: rich `.sea` fixtures live under `fixtures/`
   (see `fixtures/semantic_packs/acme_procurement/`) and
   `domainforge-core/examples/*.sea`; Rust integration tests live in
   `domainforge-core/tests/*.rs` (see `cli_tests.rs`, `authority_conformance_tests.rs`).
   Python tests in `tests/` cover bindings — binding parity is out of scope for v0.
9. **The CEP spec exists — in a sibling repo, not this one.** Normative sources:
   `/home/sprime01/projects/cep/spec/CEP-0008-semantic-envelope.md` (envelope
   identity §7.1, minimal metadata §12, benchmark-case envelope §49.1, minimal
   valid envelope §52, invalidity conditions §53, anti-collapse rules §55) and
   `/home/sprime01/projects/cep/spec/CEP-0009-benchmark-model.md` (benchmark
   families §6–12, case fields §13.1, dataset fields §15, answer key §16,
   generation §31). Nothing in domainforge references CEP yet. The projection
   emits JSON Schemas under `schemas/cep_eval/` that encode the MUST fields from
   those sections, and records `cep_version` from the spec. Vendor a pinned copy
   of the two spec files (or a pinned ref + hash) into `docs/specs/cep/` so this
   repo builds and validates without a path dependency on a sibling checkout.

---

## Core Design Principle

DomainForge is the authority. The `.sea` model is the source of truth.

No generated artifact may invent domain facts not present in the `.sea` model,
the canonical AST/Graph, or the authority resolver's output over them.

Corollary (this constrains scope, see CEP section): a static `.sea` model
contains **declared** state only. Benchmarks needing completion claims or
settlement history cannot be generated from `.sea` alone and are deferred until
DomainForge accepts such input. Observed-state divergence is the one exception:
CEP-0009 §31.2 sanctions `mutation_generation`, where a controlled mutation of
declared state *is* the synthetic observed state and fully determines the
answer key — labeled as such, this invents no domain facts.

---

## Explicitly Out of Scope

Do **not** implement: LoRA/QLoRA/PEFT/HF training, model downloads, model
inference, LLM-based synthetic generation, any model-provider API calls, GNN
training, PyTorch training loops, benchmark runner execution, leaderboards,
human review UI. Also out of scope for v0: YAML recipes, Python/TS binding
surface for these projections, PyG/NetworkX/DGL/GraphML/Parquet emitters,
alpaca/sharegpt/dpo/tool-use JSONL formats (design record shapes so they can be
added, but emit only what v0 specifies).

---

## CLI (corrected to real grammar)

Extend `ProjectFormat` in `cli/project.rs`:

```rust
pub enum ProjectFormat {
    Calm, Kg, Protobuf, Proto,
    AiLlm,       // --format ai-llm       → LLM JSONL dataset (dir output)
    AiGraphMl,   // --format ai-graph-ml  → graph ML dataset (dir output)
    CepEval,     // --format cep-eval     → CEP eval dataset (dir output)
    AiLearning,  // --format ai-learning  → all enabled kinds via recipe (dir output)
}
```

Commands:

```bash
domainforge project --format ai-llm      model.sea out/llm_dataset/
domainforge project --format ai-graph-ml model.sea out/graph_dataset/
domainforge project --format cep-eval    model.sea out/cep_eval/
domainforge project --format ai-learning --recipe recipes/ai_learning.json model.sea out/ai_learning/
```

New flags (only meaningful for ai-* formats; error politely otherwise, matching
how protobuf-only flags behave today):

```text
--recipe <FILE>        JSON recipe (required for ai-learning, optional for others)
--seed <u64>           split/sampling seed (default 0)
--families <a,b,c>     subset of families (ai-llm, cep-eval)
```

For all four new formats the positional `output` is a directory; create it,
reuse `validate_output_path` for every file written under it.

### Recipe (JSON, v0)

```json
{
  "name": "manufacturing_quality_ai_learning_v0",
  "projections": {
    "llm_dataset":     { "enabled": true, "format": "chatml_jsonl",
                         "families": ["authorization_qa", "policy_explanation",
                                      "counterexample_detection", "entity_lookup"] },
    "graph_ml_dataset": { "enabled": true,
                          "tasks": ["node_classification", "link_prediction",
                                    "policy_violation_detection"] },
    "cep_eval_dataset": { "enabled": true,
                          "families": ["AuthorityBench", "ConformanceBench"] }
  },
  "splits": { "train": 0.8, "validation": 0.1, "test": 0.1 },
  "seed": 0
}
```

Validate the recipe strictly (`serde(deny_unknown_fields)`): unknown keys,
ratios not summing to 1.0 (±ε), or unknown family names are hard errors.
YAML support is future work behind the same struct.

---

## Shared Components

One new module: `domainforge-core/src/projection/ai_learning/` with submodules
`mod.rs`, `recipe.rs`, `provenance.rs`, `split.rs`, `report.rs`, `llm.rs`,
`graph_ml.rs`, `cep_eval.rs`. Adapt naming to repo style; prefer plain structs
over trait hierarchies — three engines with a shared context struct is enough.
Do not build `TemplateRegistry`/`TemplateFamily` abstractions in v0; a match on
a family enum plus per-family generator functions is sufficient and honest.

Provenance struct embedded in every generated record and every manifest:

```text
source_model_ref        (input path, repo-relative where possible)
source_model_hash       (canonical JSON hash of the parsed Graph — reuse
                         semantic_pack::canonical_json)
source_refs             (ConceptIds of every entity/role/resource/policy/relation used)
generation_method       ("template_generation" | "authority_resolver" | "graph_export")
generator_version       (crate version via env!("CARGO_PKG_VERSION"))
recipe_ref / recipe_hash
seed
created_at              (RFC3339; injectable clock for test determinism —
                         golden tests compare with created_at normalized)
validation_status
review_status           (default "unreviewed"; "not_required" only where a
                         stated rule proves review unnecessary, e.g. verbatim
                         entity_lookup rows)
```

### Deterministic splits

Assignment is a pure function: `split(record) = bucket(stable_hash(record_id ‖
source_model_hash ‖ seed))` mapped to ratios. No RNG state, no shuffle order —
stable across runs and insertion order by construction. `record_id` is a content
hash of the normalized record (family, template id, slot values, source refs).
Record the split in each record's metadata. Ratios configurable; defaults
80/10/10. Fixed rule for tiny datasets: every non-empty dataset places ≥1
record in train; validation/test may be empty and reports must say so.
CEP split profiles (public/private/hidden/…) are future work; v0 maps to
ordinary splits.

### Deduplication (was "reject_duplicate_semantics")

True semantic dedup is not deterministically definable — dropped. v0 rule:
two records are duplicates iff their normalized tuples
`(family, template_id, sorted slot values, sorted source_refs)` are equal.
Reject the later one, count it in the validation report as `duplicate_exact`.

---

## Projection 1: LLM Dataset Projection (v0)

Compile `.sea` into deterministic seed JSONL for SFT/instruction/chat tuning.
Boring but correct; correctness, provenance, validation are the goal.

**Format: `chatml_jsonl` only.** Keep the record-shaping code format-agnostic
internally (one internal record struct, one ChatML serializer) so alpaca etc.
are additive later.

**Families (4, all mechanically derivable):**

```text
authorization_qa          "Can {role} {action} {resource}?" — label and answer
                          text derived from the authority resolver decision
                          (Allow/Deny/Escalate/Unknown), citing policy refs.
policy_explanation        "Which policy governs {action} {resource}?" — answer
                          enumerates the governing policy/policies verbatim.
counterexample_detection  Present a claim that contradicts resolver output;
                          assistant identifies and corrects it. The correction
                          text is assembled from resolver decision + policy refs.
entity_lookup             "What is {entity}? What roles/relations does it have?"
                          — answer is a verbatim rendering of graph facts.
                          review_status: not_required (rule: verbatim graph
                          rendering, no inference).
```

`rule_application`, `role_routing`, `workflow_lookup`, and the "optional"
families from v1 are deferred — they need flow/workflow query support that
should be proven by the graph projection first.

**Prompt variation** is template-variant based (the fixed variant list from v1
stands: "Can {role} …", "Is {role} allowed to …", "Does {role} have authority
to …", "Who is authorized to …", "Which policy governs …", "What role is
required to …"). Answers are never varied independently of the resolver output.

**Unknown handling**: when the resolver returns Unknown, the assistant answer
must say the model does not contain enough authority information — never guess.
This is a required test case.

Output layout:

```text
out/llm_dataset/
  train.jsonl  validation.jsonl  test.jsonl
  dataset_manifest.json  validation_report.json
  coverage_report.json   generation_report.json
```

Validation — reject rows that: reference nonexistent ConceptIds; carry a label
disagreeing with the resolver decision for the same (actor, operation,
resource); lack metadata/source refs/split assignment; are exact duplicates
(rule above). "Contradict policy effect / invent permissions" from v1 is
subsumed: labels *come from* the resolver, so a contradiction is a generator
bug — the validator re-resolves each authorization row as a check.
Report fields: rows_generated, rows_accepted, rows_rejected, rejection_reasons,
family_counts, split_counts, source_coverage, duplicate_exact_count,
unsupported_reference_count, resolver_disagreement_count.

---

## Projection 2: Graph ML Dataset Projection (v0)

Export the semantic graph for downstream graph ML. No training here.

**Formats: `graph.json` + `nodes.csv`/`edges.csv`.** (Hand-rolled CSV writer
with quote-escaping; no `csv` crate.) PyG/NetworkX/DGL/GraphML/Parquet deferred
— document the JSON schema so downstream loaders can be written.

Node types come from the actual `Graph` accessors (entities, roles, resources,
flows, instances, policies, relations, metrics, …) — enumerate what `Graph`
exposes, do not hard-code the v1 wishlist. Edge types come from the actual
relation/assignment vocabulary in `Graph` (role assignments, policy
applicability, flow steps, relations, metric attachments). Node/edge features
in v0 are categorical (type, namespace) — no invented numeric features.

**Tasks (metadata + labels): `node_classification` (node type as label),
`link_prediction` (positive edges + negative samples),
`policy_violation_detection` (see below).** Edge/workflow/subgraph
classification deferred.

**Negative sampling** — deterministic, resolver-grounded:

```text
Candidates: (role, operation, resource) triples enumerable from the graph.
Label source: the authority resolver.
  Deny      → negative sample, reason = forbidden_by_policy or missing_required_role
              (from the decision trace)
  Unknown   → EXCLUDED from negatives by default (open-world safety); a recipe
              flag `treat_unknown_as_negative` may include them with
              reason = unknown_authority, off by default
  Allow     → positive sample; never emitted as negative
Every negative carries its reason and the resolver trace refs.
Candidate enumeration is bounded: cap via recipe (`max_negative_samples`,
default e.g. 10× positives) selected by the same stable-hash order as splits.
```

Output layout:

```text
out/graph_dataset/
  graph.json  nodes.csv  edges.csv
  labels.json  masks.json  negative_samples.json
  dataset_manifest.json  validation_report.json  coverage_report.json
```

(node_features/edge_features fold into graph.json in v0 — categorical only.)

Validation — reject/warn: node without stable ConceptId; edge referencing a
missing node; unknown edge type; missing label for a labeled task; negative
sample whose resolver decision is Allow (generator bug — hard error);
overlapping masks; missing source refs. Coverage report: node/edge type
distribution, label distribution, connected components, orphan nodes,
negative sample counts + reason distribution, known_gaps.

---

## Projection 3: CEP Evaluation Dataset Projection (v0)

Compile `.sea` into structured benchmark cases with machine-checkable answer
keys, **conforming to CEP-0008 (semantic envelope) and CEP-0009 (benchmark
model)**. Ship JSON Schema files under `schemas/cep_eval/` that encode the MUST
fields of CEP-0008 §12/§49.1/§52 and CEP-0009 §13.1/§15.1/§16.1/§17.1 — the
schemas are the executable form of the spec that downstream runners and our
validator code against. Record both `cep_version` (from the spec) and
`cep_eval_schema_version: 0.1.0` (our schema files) in every artifact.

### Scope decision (grounding rule applied)

| Family | v0? | Why |
| --- | --- | --- |
| AuthorityBench | **Yes** | Fully derivable: cases from the policy/role/resource graph, answer keys from the authority resolver. CEP-0009 §8. |
| ConformanceBench | **Yes** | Checks artifacts against the published CEP-derived schemas; CEP-0009 §12, CEP-0008 §53 invalidity conditions. |
| RealityGapBench | First follow-up | Needs an observed-state ref (CEP-0009 §7.3); derivable by controlled mutation of declared state — `mutation_generation` is spec-recognized (§31.2) and the applied mutation fully determines the answer key. Excluded from v0 for scope only; design the case schema so it slots in. |
| SettlementBench | Deferred | Needs completion claims / consequence records / evidence streams (§9.3) — not in `.sea` and not synthesizable without inventing an evidence model. |
| AffordanceBench | Deferred | Needs capability/payment/current-state context (§11.3) — not in `.sea`. |
| CapabilityBench | Deferred | Needs settlement history under variation (§10.3), as v1 already conceded. |

Settlement/Affordance/Capability unlock when DomainForge accepts an
observed-state / settlement-trace input alongside the `.sea` model. Record this
in docs as the roadmap; do not stub their generators.

### AuthorityBench

Primary question: *Was this transformation permissible under applicable
authority constraints?*

Case generation: enumerate (actor-role, operation, resource, context) tuples
from the graph; the answer key is the resolver's `FinalDecision` + trace:

```text
allow    ← resolver Allow
deny     ← resolver Deny
escalate ← resolver Escalate
unknown  ← resolver Unknown (correct behavior is NOT to infer permission)
```

Answer shape (per case): decision, authority_refs, constraint_refs,
evidence_refs, confidence, reasoning_summary, limitations.

Scoring rules penalize: false_allow, false_deny, missed_escalation,
capability_authority_collapse, missing_policy_reference, scope_error,
actor_identity_error, unsupported_decision. Critical metric: false_allow_rate.

### ConformanceBench

Primary question: *Does this artifact satisfy the published cep_eval schema
requirements?*

Generate valid cases (well-formed envelopes from AuthorityBench output) and
deterministic invalid cases by removing/corrupting one required property each,
drawn from the CEP-0008 §53 invalidity conditions:
missing envelope identity, missing primary question, missing answer shape,
missing provenance, missing evidence requirements, decision outside the
allowed set, generated case marked as reviewed, raw prompt text presented as an
envelope. Each invalid case's answer key names the exact expected failure.
Conformance status values: conformant, partially_conformant, non_conformant,
not_evaluable, unknown.

### Case envelope (spec-conformant)

Envelope metadata — the CEP-0008 §12/§52 MUST set:

```text
envelope_id, cep_version, envelope_version, envelope_kind: benchmark_case,
created_at, created_by, scope, boundary_record, completeness_status,
provenance_refs, omission_status, validation_status, lineage_refs
```

Case fields — the CEP-0009 §13.1 MUST set (with CEP-0008 §49.1 "when
applicable" semantics for transformation_refs):

```text
case_id, case_version, benchmark_ref, family_ref, primary_question_ref,
case_question_ref, input_representations (actor/operation/resource/context
refs for AuthorityBench), state_refs, transformation_refs,
evidence_requirements, expected_answer_shape, answer_key_ref, metric_refs,
scoring_rules, invalidity_conditions, provenance_refs
```

plus conditional fields we always populate: review_status (§13.2), and the
generation record of §31.1 (generation_id, generator_ref, generation_method:
template_generation | policy_derived_generation, source_refs, seed,
constraints_used, target_family, human_review_status, validation_status,
known_limitations).

Answer keys — the CEP-0009 §16.1 MUST set:

```text
answer_key_id, case_ref, question_ref, expected_answer_shape, expected_values,
acceptable_variants, required_evidence_refs, required_reasoning_elements,
partial_credit_rules, invalid_answer_conditions, confidence_requirements,
provenance_refs
```

with `answer_key_type: programmatic` (§16.2) — expected_values come from the
authority resolver; required_evidence_refs point at the resolver trace/policy
refs. Fields with no meaningful v0 content (e.g. partial_credit_rules) are
present and explicitly empty, not omitted — §16.3 makes shape mismatch a
scoring-invalidating failure.

Raw prompt text is never an envelope (CEP-0008 §4.3). Answer keys live in
separate files so a runner can serve cases without leaking keys.

Output layout:

```text
out/cep_eval/
  dataset.json
  cases/*.benchmark_case.json
  answer_keys/*.answer_key.json
  answer_shapes/*.answer_shape.json
  metrics/*.metric.json
  scoring_rules/*.scoring_rule.json
  reports/{coverage,validation,generation}_report.json
```

dataset.json record (CEP-0009 §15.1): dataset_id, dataset_version, cep_version,
cep_eval_schema_version, benchmark_family_refs, case_refs, source_refs,
generation_method, review_status: unreviewed, license, provenance_refs,
distribution_profile, difficulty_distribution, coverage_report path,
known_limitations (must list the non-v0 families and why).

Validation: every case validates against the JSON Schema (add `jsonschema`
crate only if not already a transitive dev-dep; otherwise hand-validate the
required-field list — check before adding the dep); every answer key
re-verified against a fresh resolver run; anti-collapse checks per CEP-0008 §55
and CEP-0009 §44: benchmark ≠ prompt (envelope required), answer ≠ score (keys
separate from scoring rules), generated ≠ reviewed (review_status must be
unreviewed), score ≠ truth, authority ≠ capability, completion ≠ settlement
(documented in schema descriptions; the checkable subset enforced by the
validator, the rest asserted as invalidity_conditions on each case).

---

## Build Reports (all three projections)

Generation report: projection_kind, recipe_ref/hash, source_model_ref/hash,
generator_version, seed, started_at, completed_at, artifacts_generated,
families_generated, generation_methods_used, warnings, errors.
Validation report: status, passed/failed requirement counts, warnings, errors,
rejected_artifacts + reasons, repair_recommendations.
Coverage reports: as specified per projection above, always ending with
known_gaps (uncovered policies/entities/families — computed, not hand-written).

---

## Test Fixture

Add `fixtures/ai_learning/manufacturing_quality/` following the
`fixtures/semantic_packs/acme_procurement/` layout, containing valid `.sea`
(model the syntax on `acme_procurement` and `domainforge-core/examples/*.sea`
— the v1 English sentences below are *domain intent*, not `.sea` syntax):

- Roles: Certified Auditor, Trainee, Operations Manager, EHS Officer
- Resources: audit finding, audit evidence, corrective action extension
- Policies: Certified Auditor may close audit finding; Trainee may prepare
  audit evidence but not close findings; Operations Manager may approve
  corrective action extension; high-risk safety finding requires EHS
  escalation (drives an Escalate decision); at least one deliberately
  un-modeled (role, action, resource) combination to drive Unknown.
- If `.sea`/the authority pack cannot yet express one of these (e.g. the
  escalation condition), simplify the fixture to what the language supports
  and note the gap in known_limitations — do not extend the language in this
  task.

Use this one fixture to prove all three projections.

---

## Tests

Rust integration tests in `domainforge-core/tests/ai_learning_tests.rs`
(+ CLI coverage added to the existing `cli_tests.rs` pattern):

```text
1. Determinism: same fixture + recipe + seed ⇒ byte-identical output
   (created_at normalized/injected), across two runs.
2. Split stability & disjointness: assignments stable, no overlap, ratios ±ε.
3. authorization_qa deny rows exist for Trainee/close/finding and their labels
   match a fresh resolver run (resolver_disagreement_count == 0).
4. Unknown-authority prompts produce "insufficient information" answers, never
   allow/deny.
5. Graph: no edge references a missing node; no negative sample has resolver
   decision Allow (hard error path tested).
6. AuthorityBench: every case has an answer key; keys match resolver decisions;
   escalate and unknown cases are present for the fixture.
7. ConformanceBench: each invalid case fails schema validation for exactly its
   expected reason.
8. All generated artifacts default review_status: unreviewed (entity_lookup
   rows are the tested not_required exception).
9. Recipe rejection: unknown keys, bad ratios, unknown family ⇒ hard error.
10. CLI: each of the four --format values succeeds on the fixture and writes
    the specified layout; ai-learning without --recipe errors clearly;
    output path traversal is rejected (reuse validate_output_path tests).
11. Reports: validation/coverage/generation reports exist, parse, and
    known_gaps is computed (non-empty for a deliberately-uncovered policy).
```

---

## Documentation

Add `docs/ai-learning-projections.md`: overview, the three projections, real
CLI examples (grammar above), JSON recipe example, validation philosophy,
out-of-scope boundaries, deferred-benchmark roadmap + rationale, and a short
downstream example (how a trainer loads the JSONL; how a runner uses
cases/answer keys). Product framing:

> DomainForge compiles `.sea` domain models into learning artifacts.
> LLM Dataset Projection teaches language models domain behavior.
> Graph ML Dataset Projection teaches graph systems domain structure.
> CEP Evaluation Dataset Projection tests whether systems preserve domain reality.
> DomainForge does not train models. It makes business meaning trainable and testable.

Avoid: "Doc-to-LoRA", "LoRA projection", "fine-tuning inside DomainForge",
"LLM-generated truth".

---

## Implementation Sequence

```text
Phase 1  Fixture first: write fixtures/ai_learning/manufacturing_quality/,
         prove it parses (domainforge validate) and that the authority
         resolver yields Allow/Deny/Escalate/Unknown for the intended tuples.
         Everything downstream depends on this; discover language gaps now.
Phase 2  Shared skeleton: projection/ai_learning/{recipe,provenance,split,report}.rs;
         extend ProjectFormat + cli/project.rs dispatch (dir output +
         validate_output_path); recipe parsing with strict validation.
Phase 3  LLM projection v0 (4 families, chatml_jsonl) + its reports + tests 1–4, 8–9.
Phase 4  Graph ML projection v0 + resolver-grounded negative sampling + test 5.
Phase 5  CEP eval v0: vendor pinned CEP-0008/0009 into docs/specs/cep/, encode
         their MUST fields as schemas/cep_eval/ JSON Schemas, AuthorityBench,
         ConformanceBench + tests 6–7.
Phase 6  Validation hardening: re-resolution checks, dedup, coverage known_gaps;
         prefer rejecting bad artifacts over emitting pretty unsupported data.
Phase 7  Docs + CLI tests 10–11 + demo polish.
```

---

## Success Criteria

```text
1. The fixture compiles into an LLM chatml JSONL dataset, a graph ML dataset,
   and a CEP eval dataset via the four real CLI commands.
2. Every generated record embeds the shared provenance struct incl.
   source_model_hash (canonical JSON hash) and source_refs.
3. Every projection emits validation, coverage, and generation reports;
   known_gaps is computed.
4. All authorization labels and answer keys are produced by the authority
   resolver, and the validator re-verifies them (disagreement = 0).
5. Same input + recipe + seed ⇒ byte-identical output (timestamps normalized).
6. Invalid artifacts and invalid recipes are rejected with reasons.
7. No LLM, training, or model-provider dependency is added; no new heavyweight
   deps (recipes are JSON; CSV is hand-rolled; jsonschema only if cheap).
8. Emitted cases/keys/dataset validate against schemas derived from CEP-0008
   §12/§49.1/§52 and CEP-0009 §13.1/§15.1/§16.1; non-v0 families are documented
   with their unblocking path (RealityGapBench: mutation_generation follow-up;
   Settlement/Affordance/Capability: observed-state or settlement-trace input),
   not stubbed.
9. All tests above pass; existing test suite stays green.
```

## Final Quality Bar

One thin, working, defensible slice: one fixture, one chatml JSONL dataset,
one graph dataset, AuthorityBench + ConformanceBench, full provenance, full
validation, deterministic output, tests passing.

Demo:

```bash
domainforge project --format ai-learning \
  --recipe recipes/manufacturing_quality_ai_learning.json \
  fixtures/ai_learning/manufacturing_quality/domain/model.sea \
  out/manufacturing_quality_ai_learning/
```

The proof is not that generated examples sound impressive. The proof is that
every artifact is mechanically traceable to `.sea`, its labels and answer keys
are re-derivable from the authority resolver, and everything regenerates
identically when the business reality changes.
