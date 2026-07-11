# ZenML Projection (`--format zenml`)

`domainforge project --format zenml --recipe recipe.json model.sea out/` compiles
a `.sea` model **plus its authority environment** into a runnable
[ZenML](https://zenml.io/) learning-pipeline package: `@step` functions (load
dataset → train → evaluate → register) wired into a `@pipeline` DAG, with
artifacts named by deterministic ids and the run's model version keyed to the
model hash.

The projection is deterministic: a fixed `--created-at` yields byte-identical
output, and every id is minted through the shared `projection::ids` kernel.

## Learning-loop operator

ZenML is the learning-loop operator of the projection family. It **closes the
loop** the ai-learning datasets open: dataset emission (`--format ai-learning`)
→ a pipeline that *consumes* that dataset (this projection) → metrics carried
back into the run's model version. It is part of the AI family (with BAML and
DSPy) and reuses the ai-learning `AiLearningContext` — the same resolver-grounded
`AuthorityCase`s, `PolicyInfo`, and **deterministic train/dev split** the
ai-learning datasets are built from. ZenML never re-derives authority semantics
and never copies dataset rows: the pipeline's train/dev inputs are *references by
path* into the LLM dataset the ai-learning projection emits for the same model,
so the pipeline's inputs and the LLM dataset can never diverge.

## Scope boundary (read this first)

**DomainForge emits the pipeline definition (and a runnable baseline); it does
not run training.** Training, evaluation on your data, and the promotion decision
all happen when **you** run the emitted pipeline on **your** ZenML stack — **not**
inside DomainForge. The emitted `train_model` step is a deterministic
majority-class baseline you replace with real training; the promotion gate is
enforced inside the emitted `register_model` step at run time. This is the same
posture as the DSPy projection's `optimize.py`.

## What gets generated

Output is a directory:

| File | Contents |
| --- | --- |
| `pipeline.py` | the `authority_learning_pipeline` `@pipeline` wiring `load_dataset -> train_model -> evaluate_model -> register_model`, versioned under a ZenML `Model` whose version is keyed to the model hash |
| `steps.py` | the four `@step` functions; `train_model` is a deterministic majority-class baseline you replace with real training |
| `run.py` | `python run.py --dry-run` constructs the pipeline without executing it; `python run.py` runs it on your configured stack |
| `requirements.txt` | pins ZenML to the targeted API line |
| `zenml.config.json` | pipeline id, dataset references, steps + DAG edges, metric, promotion gate, model-version key, and provenance |
| `README.md` | layout, the dataset-generation step, and the scope boundary |

Generated Python imports only `zenml` and the Python standard library, so it is
import-clean and `ast.parse`-valid with just `pip install zenml`.

### Model → ZenML mapping

| Model element | ZenML construct |
| --- | --- |
| Policy-decision capability | the `authority_learning_pipeline` `@pipeline` |
| Learning stages | `load_dataset` / `train_model` / `evaluate_model` / `register_model` `@step`s |
| Step outputs | artifacts named by deterministic ids (`projection::ids`) |
| ai-learning `authorization_qa` rows | the pipeline's train/dev inputs (referenced, not copied) |
| Deterministic ai-learning split | the train/dev file references in `zenml.config.json` |
| Source model hash | the ZenML `Model` version every run groups under |
| Resolver decision agreement | the `decision_agreement` metric the pipeline computes |
| Recipe promotion threshold | the gate enforced by `register_model` |

By default only decisive cases (allow/deny/escalate/reject) inform the labels and
examples; set the recipe's `zenml.include_unknown` to also include tuples the
resolver labels `unknown`.

## Recipe

ZenML reuses the one ai-learning recipe system — there is no second recipe
format. Its section is minimal:

```json
{
  "name": "my_zenml_v0",
  "authority_config": "../authority/environment.json",
  "projections": {
    "zenml": {
      "enabled": true,
      "pipeline_name": "authority_learning_pipeline",
      "model_name": "authority_decider",
      "promotion_threshold": 0.0,
      "include_unknown": false
    }
  }
}
```

- `pipeline_name` and `model_name` must be valid Python identifiers (validated).
- `promotion_threshold` (0.0–1.0) is the minimum `decision_agreement` score
  `register_model` requires before promoting the run's model version. Default
  `0.0` = always promote (no gate).

The authority environment may instead be supplied on the CLI with
`--authority-config`; one of the two is **required** because the pipeline and its
examples are resolver-grounded.

## Examples are referenced, never copied

`zenml.config.json` records the dataset paths under `dataset.root` (default
`../ai_learning`): `llm_dataset/train.jsonl` (train) and
`llm_dataset/validation.jsonl` (dev). These are the exact relative paths
`domainforge project --format ai-learning` emits for the same model. Generate the
dataset first:

```bash
domainforge project --format ai-learning --recipe recipe.json model.sea ../ai_learning
```

`load_dataset` then reads those files, filters to the `authorization_qa` family,
and builds `{question, decision}` rows — the pipeline's examples are the LLM
dataset's own rows, single-producer.

## Model-version grouping

Every run of the same domain model versions its ZenML `Model` under `v-<hash>`,
where `<hash>` is the source model hash. Runs of the same model therefore group
under one model version, and the evaluated metric flows back into the run's
`register_model` artifact — closing the learning loop.

## Determinism

- The decision labels, policy lines, DAG edges, and dataset references are built
  in sorted / `BTreeMap` order.
- No wall-clock or randomness enters file contents — only the caller-supplied
  `created_at`.
- The pipeline id and every step artifact id are minted through
  `projection::ids::element_id` under the `zenml` family tag, so renaming a model
  element changes exactly the ids derived from it.

## Validation

The pinned target ZenML API line is recorded in `ZENML_TARGET_VERSION`
(`domainforge-core/src/projection/zenml/ir.rs`) and pinned in the emitted
`requirements.txt` (`ZENML_PIP_SPEC`). ZenML's decorator API has shifted between
releases, so the projection targets a pinned line, period — bumping it is a
constant change plus a green `verify-zenml` CI run, the same policy as
`LEAN_TOOLCHAIN`.

**What is live-validated:** every emitted `.py` file parses
(`python -c "import ast; ast.parse(...)"`, the plan's accepted floor, needing no
third-party install), and the ZenML dataset refs resolve to files the ai-learning
projection actually emits for the same fixture (cross-projection consistency).

**What is NOT live-validated (and why):** a real `pip install zenml` plus a live
pipeline-construction dry-run is not run in CI. ZenML's dependency tree is very
heavy (SQLAlchemy, pydantic, a web stack, ML extras) and slow/flaky to install on
every run, and — per the scope boundary above — training, evaluation, and
promotion are the user's compute on their ZenML stack, not DomainForge's.
Promoting a pinned `zenml` install to a separate non-blocking job is a documented
follow-up. This is the same substitution the OTel (weaver) and DSPy projections
make for heavyweight external tools.

To validate locally:

```bash
domainforge project --format zenml --recipe fixtures/zenml/basic/recipes/zenml.json \
  --created-at 2026-07-02T00:00:00+00:00 fixtures/zenml/basic/domain/model.sea out/
python3 -c "import ast, glob; [ast.parse(open(f).read()) for f in glob.glob('out/**/*.py', recursive=True)]"
```

## Non-goals (v1)

- **No training run.** The projection emits the pipeline definition and a
  runnable baseline only; real training, evaluation, and promotion happen when
  you run the pipeline on your ZenML stack, not in DomainForge.
- **No stack/orchestrator selection.** `run.py` runs against whatever ZenML stack
  you have configured; DomainForge picks none.
- **No richer per-capability pipelines.** v1 emits the single authority-decision
  learning loop; broader pipelines are deferred.
- **No new DSL syntax.** ZenML consumes the graph and authority environment as-is.

## Implementation

- IR: `domainforge-core/src/projection/zenml/ir.rs` (`LearningPipelineIR`).
- Renderer: `domainforge-core/src/projection/zenml/render.rs`.
- `emit` / `project_zenml_in_memory`: `domainforge-core/src/projection/zenml/mod.rs`.
- Recipe section: `zenml` in `domainforge-core/src/projection/ai_learning/recipe.rs`.
- Split reuse: `domainforge-core/src/projection/ai_learning/split.rs`
  (`TRAIN`/`VALIDATION` constants drive the referenced file names).
- Fixture: `fixtures/zenml/basic/` (reuses the manufacturing-quality authority
  environment).
- Tests: `domainforge-core/tests/zenml_projection_tests.rs`, `tests/test_zenml.py`,
  `typescript-tests/zenml.test.ts`.
- CI: the `verify-zenml` job in `.github/workflows/ci.yml`.
