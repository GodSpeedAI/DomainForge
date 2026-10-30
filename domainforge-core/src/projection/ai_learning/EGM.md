# AI Learning Projections

## Overview
Compiles `.sea` models into machine-learnable and machine-evaluable artifacts (LLM JSONL datasets, graph-ML exports, CEP evaluation cases) under one shared context, recipe, provenance, split, and report layer. No generated artifact may invent domain facts: authorization labels come from the authority resolver.

## Interface
- `[mod.rs:GENERATOR_VERSION]` — the generator version stamped into provenance, from `CARGO_PKG_VERSION`.
- `[mod.rs:AiLearningContext]` — the shared input bundle: graph, model ref/hash, recipe + hash, seed, created_at, authority environment, authority cases, policy info.
- `[mod.rs:AiLearningContext::build]` — constructs the context, computing `model_hash` from a run-stable canonical graph rendering and `recipe_hash` from the recipe.
- `[mod.rs:AiLearningContext::require_authority]` — fails when a projection needs resolver-grounded labels but no authority environment was supplied.
- `[mod.rs:AiLearningContext::provenance]` — mints the shared `Provenance` record.
- `[mod.rs:AuthorityCase]` — one resolver-grounded authorization case (role, operation, resource type, decision, label, reason code, policy lists).
- `[mod.rs:PolicyInfo]` — static description of one authority-pack policy.
- `[mod.rs:decision_label]` — maps `FinalDecision` to `allow` / `deny` / `escalate` / `unknown` / `reject`.
- `[mod.rs:project_ai_learning_in_memory]` — binding surface: graph + recipe JSON + authority-config JSON in, `BTreeMap<path, content>` out, no filesystem.
- `[recipe.rs:Recipe]` / `[recipe.rs:Recipe::from_json]` / `[recipe.rs:Recipe::validate]` — the strictly validated JSON recipe.
- `[recipe.rs:Projections]` — per-projection sections: `llm_dataset`, `graph_ml_dataset`, `cep_eval_dataset`, `baml`, `dspy`, `zenml`.
- `[recipe.rs:LLM_FAMILIES]`, `[recipe.rs:GRAPH_ML_TASKS]`, `[recipe.rs:CEP_FAMILIES]`, `[recipe.rs:LLM_FORMATS]`, `[recipe.rs:DSPY_MODULE_STRATEGIES]`, `[recipe.rs:DSPY_OPTIMIZERS]` — the closed allowed-value sets.
- `[recipe.rs:SplitRatios]` — train/validation/test ratios.
- `[recipe.rs:ZenmlProjection]`, `[recipe.rs:DspyProjection]`, `[recipe.rs:BamlProjection]`, `[recipe.rs:LlmProjection]`, `[recipe.rs:GraphMlProjection]`, `[recipe.rs:CepEvalProjection]` — the per-family recipe sections.
- `[llm.rs:emit]` — emits one `<split>.jsonl` per non-empty split (train/validation/test) plus `dataset_manifest.json`, `validation_report.json`, `coverage_report.json`, and `generation_report.json`.
- `[graph_ml.rs:emit]` — emits the graph-ML export (`graph.json`, `nodes.csv`, `edges.csv`, `labels.json`, `masks.json`, `negative_samples.json`, `dataset_manifest.json`).
- `[cep_eval.rs:emit]` — emits per-case `cases/<id>.benchmark_case.json` and `answer_keys/<id>.answer_key.json` (separate files so a runner can serve cases without leaking keys), `answer_shapes/*.answer_shape.json`, per-metric files, and `dataset.json`.
- `[cep_eval.rs:REQUIRED_ENVELOPE_FIELDS]` / `[cep_eval.rs:REQUIRED_CASE_FIELDS]` / `[cep_eval.rs:REQUIRED_ANSWER_KEY_FIELDS]` — the vendored CEP spec field sets.
- `[cep_eval.rs:CEP_VERSION]` / `[cep_eval.rs:CEP_EVAL_SCHEMA_VERSION]` — the emitted CEP versions (`0.1.0`).
- `[split.rs:assign_split]` / `[split.rs:ensure_train_nonempty]` / `[split.rs:record_id]` — deterministic split assignment and record identity.
- `[split.rs:TRAIN]` / `[split.rs:VALIDATION]` / `[split.rs:TEST]` — the split name constants.
- `[provenance.rs:Provenance]` — the shared provenance record embedded in every record and manifest.
- `[report.rs:GenerationReport]` / `[report.rs:ValidationReport]` / `[report.rs:RejectedArtifact]` / `[report.rs:CoverageReport]` / `[report.rs:counts_to_value]` — the shared report types.

## Invariants
- `[recipe.rs:Recipe]` — every recipe struct is `#[serde(deny_unknown_fields)]`, so unknown keys are hard errors.
- `[recipe.rs:Recipe::validate]` — split ratios must each be in `0.0..=1.0` and sum to `1.0` within `1e-9`; families/tasks/formats/strategies/optimizers must be members of the declared constant sets; `baml.function_name` must be a valid BAML identifier; `dspy.signature_name` must be a valid Python identifier.
- `[split.rs:assign_split]` — split assignment is a pure function of `(record_id ‖ source_model_hash ‖ seed)` via xxh64; there is no RNG state and no shuffle order.
- `[split.rs:ensure_train_nonempty]` — every non-empty dataset places at least one record in train.
- `[split.rs:record_id]` — delegates to `crate::projection::ids::content_hash`, so ai-learning record ids share the projection identity convention.
- `[mod.rs:AiLearningContext::build]` — `model_hash` is computed over a run-stable canonical graph value, not raw Graph serialization, because flow ids are fresh UUIDs per parse.
- `[llm.rs:emit]` — the authorization families (`authorization_qa`, `policy_explanation`, `counterexample_detection`) require the authority environment.
- `[graph_ml.rs:emit]` — the `policy_violation_detection` task requires the authority environment.
- `[cep_eval.rs:emit]` — a non-empty CEP family set requires the authority environment (ConformanceBench's envelopes are derived from a well-formed AuthorityBench case).
- `[cep_eval.rs:emit]` — benchmark cases and answer keys are written to separate trees so a runner can serve cases without leaking keys (answer ≠ score, benchmark ≠ prompt).
- `[provenance.rs:Provenance]` — `created_at` is injectable so golden runs stay byte-identical; `review_status` defaults to `unreviewed` and is set to `not_required` only where a stated rule proves review unnecessary.
- `[cep_eval.rs]` (module doc) — v0 families are `AuthorityBench` and `ConformanceBench`; `RealityGapBench` and the Settlement/Affordance/Capability families are not emitted.

## Side Effects
- `[llm.rs:emit]`, `[graph_ml.rs:emit]`, `[cep_eval.rs:emit]` — write through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
