# AI Learning Projections

DomainForge compiles `.sea` domain models into learning artifacts.

> LLM Dataset Projection teaches language models domain behavior.
> Graph ML Dataset Projection teaches graph systems domain structure.
> CEP Evaluation Dataset Projection tests whether systems preserve domain reality.
> DomainForge does not train models. It makes business meaning trainable and testable.

DomainForge does not teach AI from documents; it compiles organizational
reality into machine-learnable and machine-evaluable representations. There
are no LLM calls, no training loops, and no model-provider dependencies in
the generator. Everything is deterministic, source-grounded, and
validation-first.

## The three projections

| Format | Output | Purpose |
| --- | --- | --- |
| `ai-llm` | chatml JSONL (`train/validation/test.jsonl`) | SFT/instruction seed data |
| `ai-graph-ml` | `graph.json`, `nodes.csv`/`edges.csv`, labels/masks/negative samples | GNN training exports |
| `cep-eval` | benchmark cases + answer keys per CEP-0008/CEP-0009 | machine-checkable evals |
| `ai-learning` | all enabled projections via one recipe | one-command build |

## CLI

```bash
domainforge project --format ai-llm      model.sea out/llm_dataset/
domainforge project --format ai-graph-ml model.sea out/graph_dataset/
domainforge project --format cep-eval    model.sea out/cep_eval/
domainforge project --format ai-learning --recipe recipes/ai_learning.json model.sea out/ai_learning/
```

Flags for the ai-* formats:

- `--recipe <FILE>` — JSON recipe (required for `ai-learning`, optional otherwise)
- `--seed <u64>` — split/sampling seed (overrides the recipe seed, default 0)
- `--families <a,b,c>` — family subset (`ai-llm`, `cep-eval`)
- `--authority-config <FILE>` — AuthorityEnvironmentConfig JSON (overrides the recipe)
- `--created-at <RFC3339>` — fixed timestamp; with it, output is byte-identical across runs

The positional output is a directory; every file written under it passes the
same path-traversal validation as protobuf `--multi-file`.

## The authority environment is the label source

The authority resolver — not templates — computes every authorization label
and every answer key. The resolver consumes an `AuthorityEnvironmentConfig`
JSON (the same format as `domainforge authority`), supplied via
`--authority-config` or the recipe's `authority_config` field (resolved
relative to the recipe file). The projection enumerates
(role, operation, resource-type) tuples — roles and resources from the parsed
graph, operations from the pack's `action` vocabulary — and resolves each
with empty facts:

- `Allow` / `Deny` / `Escalate` map to their labels directly.
- `NotApplicable` (no applicable policy) is labeled **`unknown`**: the model
  does not contain enough authority information, and generated answers must
  say so rather than guess.
- `Escalate` typically arises from unknown-handling on policies whose fact
  requirements cannot be satisfied from declared state alone.

The validator re-resolves every authorization row/answer key against a fresh
resolver run; any disagreement is a rejected artifact and a nonzero
`resolver_disagreement_count`.

## Recipe (JSON, v0)

See `fixtures/ai_learning/manufacturing_quality/recipes/ai_learning.json`:

```json
{
  "name": "manufacturing_quality_ai_learning_v0",
  "authority_config": "../authority/environment.json",
  "projections": {
    "llm_dataset": { "enabled": true, "format": "chatml_jsonl",
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

Recipes are strictly validated (`deny_unknown_fields`): unknown keys, ratios
not summing to 1.0, or unknown family/task/format names are hard errors.
YAML recipes are future work behind the same struct.

## Determinism, provenance, splits, dedup

- **Provenance** is embedded in every record and manifest: `source_model_ref`,
  `source_model_hash` (canonical-JSON SHA-256 of the model's semantic content
  — flow UUIDs are excluded because they are regenerated on every parse),
  `source_refs` (ConceptIds and policy ids), `generation_method`,
  `generator_version`, `recipe_ref`/`recipe_hash`, `seed`, `created_at`,
  `validation_status`, `review_status`.
- **Splits** are a pure function: `bucket(stable_hash(record_id ‖
  source_model_hash ‖ seed))` mapped to the ratios — no RNG state, no shuffle
  order. Every non-empty dataset places at least one record in train;
  validation/test may be empty and the reports say so.
- **Dedup** is exact: two records are duplicates iff their normalized tuples
  `(family, template_id, sorted slot values, sorted source_refs)` are equal;
  the later one is rejected and counted as `duplicate_exact`.
- **review_status** defaults to `unreviewed` everywhere. The one
  `not_required` exception: `entity_lookup` rows, which are verbatim graph
  renderings with no inference.

## Validation philosophy

Reject bad artifacts rather than emit pretty unsupported data. Every
projection emits `validation_report.json` (status, rejected artifacts with
reasons, counters), `coverage_report.json` (distributions plus a computed
`known_gaps` list — never hand-written), and `generation_report.json`.

## CEP Evaluation Dataset

Conforms to CEP-0008 (semantic envelope) and CEP-0009 (benchmark model),
pinned under `docs/specs/cep/` (see `VENDORED.md`); the executable form of
their MUST-field lists ships as JSON Schemas under `schemas/cep_eval/`.
`cep_version: 0.1.0`, `cep_eval_schema_version: 0.1.0`.

- **AuthorityBench** — *Was this transformation permissible under applicable
  authority constraints?* One case per enumerated tuple; the answer key is
  the resolver's decision plus its policy refs (`answer_key_type:
  programmatic`). Critical metric: `false_allow_rate`.
- **ConformanceBench** — *Does this artifact satisfy the published cep_eval
  schema requirements?* One valid envelope plus twelve deterministic invalid
  cases, each corrupting exactly one requirement drawn from the CEP-0008 §53
  invalidity conditions; each answer key names the exact expected failure.

Cases and answer keys live in separate files so a runner can serve cases
without leaking keys. Anti-collapse rules (CEP-0008 §55 / CEP-0009 §44) are
enforced where checkable (benchmark ≠ prompt, answer ≠ score,
generated ≠ reviewed) and asserted as `invalidity_conditions` otherwise.

### Deferred benchmark families (roadmap)

| Family | Status | Unblocking input |
| --- | --- | --- |
| RealityGapBench | **First follow-up** | None beyond implementation: CEP-0009 §31.2 sanctions `mutation_generation` — a controlled mutation of declared state *is* the synthetic observed state and fully determines the answer key. |
| SettlementBench | Deferred | Completion claims / consequence records / evidence streams (§9.3). |
| AffordanceBench | Deferred | Capability/payment/current-state context (§11.3). |
| CapabilityBench | Deferred | Settlement history under variation (§10.3). |

These unlock when DomainForge accepts an observed-state or settlement-trace
input alongside the `.sea` model. Their generators are not stubbed; the
dataset's `known_limitations` lists them.

## Language bindings (v1)

All three bindings expose the same surface on `Graph`: string in, JSON string
out, no filesystem (wasm-safe). The returned JSON object maps relative
artifact paths — the exact `--format ai-learning` layout — to file contents,
so callers can write them to disk, an object store, or keep them in memory.

```python
# Python
graph = domainforge.Graph.parse(source)
artifacts = json.loads(graph.export_ai_learning(
    recipe_json=recipe_json,               # optional; defaults enable everything
    authority_config_json=authority_json,  # required for resolver-grounded families
    created_at="2026-07-02T00:00:00+00:00" # optional; fixes timestamps for reproducibility
))
artifacts["llm_dataset/train.jsonl"]
```

```ts
// TypeScript (napi) — camelCase: graph.exportAiLearning(recipeJson?, authorityConfigJson?, modelRef?, seed?, createdAt?)
// WASM — graph.exportAiLearning(...) with the same signature.
const artifacts = JSON.parse(graph.exportAiLearning(recipeJson, authorityJson));
```

Omitting the authority config is fine for families that need no resolver
(`entity_lookup`, `node_classification`, `link_prediction`); resolver-grounded
families fail loudly instead of guessing. The binding path is covered by
`tests/test_ai_learning.py` (Python), `typescript-tests/ai_learning.test.ts`
(TS), and the Rust parity test `in_memory_api_matches_cli_output`, which
asserts the in-memory output is byte-identical to the CLI's.

## Out of scope (v0)

LoRA/QLoRA/PEFT/HF training, model downloads or inference, LLM-based
synthetic generation, model-provider API calls, GNN training, benchmark
runner execution, leaderboards, human review UI, YAML recipes,
PyG/NetworkX/DGL/GraphML/Parquet emitters, alpaca/sharegpt/dpo/tool-use
formats (the internal record shapes keep them additive). Binding surface was
deferred in v0 and shipped in v1 (section above).

## Downstream examples

Load the JSONL for SFT (any chatml-aware trainer):

```python
import json
rows = [json.loads(l) for l in open("out/llm_dataset/train.jsonl")]
for r in rows:
    messages = r["messages"]          # system/user/assistant
    provenance = r["metadata"]["provenance"]  # trace every row to the model
```

Run an eval from cases + answer keys:

```python
import json, pathlib
root = pathlib.Path("out/cep_eval")
for case_file in sorted((root / "cases").glob("*.benchmark_case.json")):
    case = json.loads(case_file.read_text())
    prediction = my_system(case["case_question_ref"], case["input_representations"])
    key = json.loads((root / case["answer_key_ref"]).read_text())
    score(prediction, key["expected_values"])   # keys served separately from cases
```

## Fixture

`fixtures/ai_learning/manufacturing_quality/` proves all three projections:
a quality-audit domain (`domain/model.sea`), an authority environment
(`authority/environment.json`, regenerable via
`cargo test --test gen_ai_learning_fixture -- --ignored`) that yields all
four decisions, and the recipe above. Known language gap recorded there:
relation subjects/objects must both be roles, so role→resource authorization
links live in the authority pack, not the `.sea` relations.
