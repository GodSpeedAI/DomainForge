# DSPy Projection (`--format dspy`)

`domainforge project --format dspy --recipe recipe.json model.sea out/` compiles
a `.sea` model **plus its authority environment** into a runnable
[DSPy](https://dspy.ai/) optimization program: an authority-decision signature
whose prompt carries the governing policies, a metric measuring decision-label
agreement, and an optimizer script with a declared regression gate.

The projection is deterministic: a fixed `--created-at` yields byte-identical
output, and every id is minted through the shared `projection::ids` kernel.

## AI-optimization operator

DSPy is the AI-optimization operator of the projection family. It is part of the
AI family (with BAML and ZenML) and reuses the ai-learning `AiLearningContext` —
the same resolver-grounded `AuthorityCase`s, `PolicyInfo`, and **deterministic
train/dev split** the ai-learning datasets are built from. DSPy never re-derives
authority semantics and never copies dataset rows: its train/dev examples are
*references by path* into the LLM dataset the ai-learning projection emits for
the same model, so DSPy examples and the LLM dataset can never diverge.

## Scope boundary (read this first)

**DomainForge emits a program plus config; it does not run optimization.** The
baseline-vs-optimized comparison and the regression-threshold enforcement happen
inside the *emitted* `optimize.py`, which is the user's compute (it needs a
configured LM and a chosen vendor) — **not** inside DomainForge. Running the
optimizer, comparing the optimized program against the baseline, and enforcing
the threshold are all the user's responsibility once they configure an LM.

## What gets generated

Output is a directory:

| File | Contents |
| --- | --- |
| `program.py` | the `AuthorityDecision` `dspy.Signature` (the capability; its docstring lists the governing policies) and the `AuthorityDecisionProgram` `dspy.Module` wrapping it |
| `metric.py` | `decision_match`, decision-label agreement between the prediction and the resolver-grounded gold label |
| `optimize.py` | loads the referenced ai-learning splits, evaluates a baseline, compiles an optimized program, and enforces the regression gate — run by **you** |
| `dspy.config.json` | signature spec, dataset references, metric, optimizer config + regression threshold, and provenance |
| `README.md` | layout, the dataset-generation step, and how to make it runnable |

Generated Python imports only `dspy` and the Python standard library, so it is
import-clean and `ast.parse`-valid with just `pip install dspy`.

### Model → DSPy mapping

| Model element | DSPy construct |
| --- | --- |
| Policy-decision capability | `AuthorityDecision` signature (input = authorization question, output = decision label) |
| Resolver decision set | the closed `DECISION_LABELS` the output ranges over |
| Authority policies | signature-docstring governing-policy list |
| ai-learning `authorization_qa` rows | train/dev `dspy.Example`s (referenced, not copied) |
| Deterministic ai-learning split | the train/dev file references in `dspy.config.json` |
| Recipe regression threshold | the gate enforced by `optimize.py` |

By default only decisive cases (allow/deny/escalate/reject) inform the labels and
examples; set the recipe's `dspy.include_unknown` to also include tuples the
resolver labels `unknown`.

### Signature scope (v1 narrowing)

v1 emits **one** signature — the authority-decision capability — because the
fixture proves that path end-to-end. Richer per-capability prompt contracts are
deliberately deferred (see Non-goals); this narrowing is stated rather than done
silently, per the plan's redesign trigger.

## Recipe

DSPy reuses the one ai-learning recipe system — there is no second recipe format.
Its section is minimal:

```json
{
  "name": "my_dspy_v0",
  "authority_config": "../authority/environment.json",
  "projections": {
    "dspy": {
      "enabled": true,
      "signature_name": "AuthorityDecision",
      "module_strategy": "predict",
      "optimizer": "BootstrapFewShot",
      "max_bootstrapped_demos": 4,
      "max_labeled_demos": 16,
      "regression_threshold": 0.0,
      "include_unknown": false
    }
  }
}
```

- `signature_name` must be a valid Python identifier (validated).
- `module_strategy` is `predict` (default, → `dspy.Predict`) or `chain_of_thought`
  (→ `dspy.ChainOfThought`).
- `optimizer` is `BootstrapFewShot` (default) or `LabeledFewShot`.
- `regression_threshold` (0.0–1.0) is the tolerated drop of the optimized dev
  score below the baseline; `optimize.py` fails if
  `optimized < baseline - regression_threshold`. Default `0.0` = no regression
  tolerated.

The authority environment may instead be supplied on the CLI with
`--authority-config`; one of the two is **required** because the signature and
its examples are resolver-grounded.

## Examples are referenced, never copied

`dspy.config.json` records the dataset paths under `dataset.root` (default
`../ai_learning`): `llm_dataset/train.jsonl` (train) and
`llm_dataset/validation.jsonl` (dev). These are the exact relative paths
`domainforge project --format ai-learning` emits for the same model. Generate the
dataset first:

```bash
domainforge project --format ai-learning --recipe recipe.json model.sea ../ai_learning
```

`optimize.py` then loads those files, filters to the `authorization_qa` family,
and builds `dspy.Example(question=…, decision=…)` rows — the DSPy examples are
the LLM dataset's own rows, single-producer.

## No vendor lock-in

`optimize.py` refuses to run until you configure a DSPy LM
(`dspy.configure(lm=dspy.LM(...))`). No provider or key is baked into the
generated code; DomainForge stays vendor-neutral, exactly like the BAML client
placeholder.

## Determinism

- The decision labels, policy lines, and dataset references are built in sorted /
  `BTreeMap` order.
- No wall-clock or randomness enters file contents — only the caller-supplied
  `created_at`.
- The capability id is minted through `projection::ids::element_id` under the
  `dspy` family tag, so renaming a model element changes exactly the ids derived
  from it.

## Validation

The pinned target DSPy API line is recorded in `DSPY_TARGET_VERSION`
(`domainforge-core/src/projection/dspy/ir.rs`). Bumping it is a constant change
plus a green `verify-dspy` CI run — the same policy as `LEAN_TOOLCHAIN`.

**What is live-validated:** every emitted `.py` file parses
(`python -c "import ast; ast.parse(...)"`, the plan's accepted floor, needing no
third-party install), and the DSPy example refs resolve to files the ai-learning
projection actually emits for the same fixture (cross-projection consistency).

**What is NOT live-validated (and why):** DSPy's optimizer is not run in CI.
Running it needs a configured LM and a chosen vendor, and — per the scope
boundary above — the baseline-vs-optimized comparison and the threshold gate are
the user's compute inside `optimize.py`, not DomainForge's. This is the same
substitution the OTel and BAML projections make for heavyweight external tools.

To validate locally:

```bash
domainforge project --format dspy --recipe fixtures/dspy/basic/recipes/dspy.json \
  --created-at 2026-07-02T00:00:00+00:00 fixtures/dspy/basic/domain/model.sea out/
python3 -c "import ast, glob; [ast.parse(open(f).read()) for f in glob.glob('out/**/*.py', recursive=True)]"
```

## Non-goals (v1)

- **No optimization run.** The projection emits the program and config only; the
  baseline-vs-optimized comparison and regression enforcement live in the emitted
  `optimize.py`, not in DomainForge.
- **No LM/provider selection.** `optimize.py` requires you to configure an LM.
- **No richer per-capability signatures.** v1 emits the single authority-decision
  signature; broader prompt contracts are deferred.
- **No new DSL syntax.** DSPy consumes the graph and authority environment as-is.

## Implementation

- IR: `domainforge-core/src/projection/dspy/ir.rs` (`AIOptimizationIR`).
- Renderer: `domainforge-core/src/projection/dspy/render.rs`.
- `emit` / `project_dspy_in_memory`: `domainforge-core/src/projection/dspy/mod.rs`.
- Recipe section: `dspy` in `domainforge-core/src/projection/ai_learning/recipe.rs`.
- Split reuse: `domainforge-core/src/projection/ai_learning/split.rs`
  (`TRAIN`/`VALIDATION` constants drive the referenced file names).
- Fixture: `fixtures/dspy/basic/` (reuses the manufacturing-quality authority
  environment).
- Tests: `domainforge-core/tests/dspy_projection_tests.rs`, `tests/test_dspy.py`,
  `typescript-tests/dspy.test.ts`.
- CI: the `verify-dspy` job in `.github/workflows/ci.yml`.
