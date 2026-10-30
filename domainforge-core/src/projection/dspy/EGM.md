# DSPy Optimization Projection

## Overview
Compiles a `.sea` model plus its authority environment into a runnable DSPy optimization program: an authority-decision signature, a decision-label-agreement metric, and an optimizer script with a declared regression gate. It emits program + config and never runs optimization.

## Interface
- `[mod.rs:emit]` — builds `AIOptimizationIR` from the context and writes `program.py`, `metric.py`, `optimize.py`, `dspy.config.json`, `README.md` into an `ArtifactSink`.
- `[mod.rs:project_dspy_in_memory]` — binding surface: graph + recipe JSON + authority-config JSON in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:DSPY_FAMILY]` — the `element_id` family tag, `dspy`.
- `[ir.rs:DSPY_TARGET_VERSION]` — the pinned DSPy target version, `2.5`.
- `[ir.rs:LLM_DATASET_SUBDIR]` — the sibling ai-learning dataset directory the pipeline references, `llm_dataset`.
- `[ir.rs:SOURCE_FAMILY]` — the ai-learning family the examples are filtered to, `authorization_qa`.
- `[ir.rs:AIOptimizationIR::build]` — lowers an `AiLearningContext` into the optimization IR.
- `[ir.rs:AIOptimizationIR::dspy_module_call]` — the DSPy module call name for the configured strategy.
- `[ir.rs:DatasetRef]` — paths (never copied rows) to the ai-learning train/dev splits.
- `[ir.rs:OptimizerSpec]` — optimizer config plus the tolerated `regression_threshold`.
- `[render.rs:program]` / `[render.rs:metric]` / `[render.rs:optimize]` / `[render.rs:config]` / `[render.rs:readme]` — the five renderers.

## Invariants
- `[ir.rs:AIOptimizationIR::build]` — requires a resolver-grounded authority environment (`AiLearningContext::require_authority`); without one the projection fails.
- `[ir.rs:DatasetRef]` — dataset references are paths, never copied data, so DSPy examples and the ai-learning LLM dataset can never diverge.
- `[mod.rs]` (module doc) — the projection never runs optimization; baseline-vs-optimized comparison and regression-threshold enforcement live in the emitted `optimize.py`.
- `[ir.rs:AIOptimizationIR]` — the IR is built in sorted order (BTreeMap/BTreeSet only) and every id is minted through `projection::ids`, so output is byte-deterministic for a fixed `created_at`.
- Exactly the five listed artifacts are emitted.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
