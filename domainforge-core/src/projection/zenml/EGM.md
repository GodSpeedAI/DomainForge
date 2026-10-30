# ZenML Learning Pipeline Projection

## Overview
Compiles a `.sea` model plus its authority environment into a runnable ZenML learning-pipeline package: `@step` functions (load dataset → train → evaluate → register) wired into a `@pipeline` DAG, with artifacts named by deterministic ids and the run's model version keyed to the model hash.

## Interface
- `[mod.rs:emit]` — builds `LearningPipelineIR` from the context and writes `pipeline.py`, `steps.py`, `run.py`, `requirements.txt`, `zenml.config.json`, `README.md` into an `ArtifactSink`.
- `[mod.rs:project_zenml_in_memory]` — binding surface: graph + recipe JSON + authority-config JSON in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:ZENML_FAMILY]` — the `element_id` family tag, `zenml`.
- `[ir.rs:ZENML_TARGET_VERSION]` — the pinned ZenML target version, `0.68`.
- `[ir.rs:ZENML_PIP_SPEC]` — the emitted pip requirement spec, `zenml>=0.68,<0.69`.
- `[ir.rs:LLM_DATASET_SUBDIR]` — the sibling ai-learning dataset directory, `llm_dataset`.
- `[ir.rs:SOURCE_FAMILY]` — the ai-learning family the examples are filtered to, `authorization_qa`.
- `[ir.rs:METRIC_NAME]` — the decision-agreement metric name, `decision_agreement`.
- `[ir.rs:LearningPipelineIR::build]` — lowers an `AiLearningContext` into the pipeline IR.
- `[ir.rs:LearningPipelineIR::dag_edges]` — the derived `(from, to)` pipeline edges.
- `[ir.rs:DatasetRef]` / `[ir.rs:StepSpec]` — dataset path references and one step definition.
- `[render.rs:steps]` / `[render.rs:pipeline]` / `[render.rs:run]` / `[render.rs:requirements]` / `[render.rs:config]` / `[render.rs:readme]` — the six renderers.

## Invariants
- `[ir.rs:LearningPipelineIR::build]` — requires a resolver-grounded authority environment (`AiLearningContext::require_authority`); the pipeline and its labeled examples are grounded in the authority pack's decisions.
- `[ir.rs:DatasetRef]` — train/dev inputs are the ai-learning LLM dataset's own splits referenced by path, never copied rows, so the pipeline inputs and the dataset can never diverge.
- `[mod.rs]` (module doc) — the projection emits the pipeline definition only; it never runs training.
- `[ir.rs:LearningPipelineIR]` — the IR is built in sorted order (BTreeMap/BTreeSet only) and every id is minted through `projection::ids`, so output is byte-deterministic for a fixed `created_at`.
- Exactly the six listed artifacts are emitted.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
