# Lean 4 Projection (`--format lean`)

DomainForge can project a `.sea` model into a self-contained **Lean 4** package
whose compilation *is* formal verification: `lake build` re-checks every
generated theorem against the declared model on every run.

```bash
domainforge project --format lean domain/model.sea out/
cd out && lake build          # proof-checks the model
```

The same artifacts are available from the language bindings as a
path → content map (no filesystem access needed):

```python
artifacts = json.loads(graph.export_lean(created_at="2026-07-02T00:00:00+00:00"))
```

```ts
const artifacts = JSON.parse(graph.exportLean(undefined, '2026-07-02T00:00:00+00:00'));
```

## What gets generated

| File | Content |
| --- | --- |
| `lean-toolchain` | Pinned Lean release (managed by `elan`) |
| `lakefile.toml` | Lake package: `DomainForge` (default target) + `Obligations` |
| `DomainForge/Types.lean` | `inductive Entity / Role / Resource` — the closed domains, with `DecidableEq` |
| `DomainForge/Model.lean` | `flows` and `relations` as concrete data; `quantityScale` |
| `DomainForge/Policies.lean` | One `abbrev … : Prop` per auto-groundable policy plus a `by decide` theorem |
| `Obligations/Stubs.lean` | `sorry` stubs for policies that cannot be auto-grounded |
| `README.md` | How to check and strengthen the package |

The package has **zero external dependencies** — no Mathlib, no Batteries.
`lake build` needs no network access; CI verification is hermetic and fast.

## The two-layer proof discipline

- **`DomainForge` (the checked layer)** always compiles **sorry-free**. Every
  theorem is discharged automatically by `decide` over the finite generated
  model — no proof engineering required. A groundable policy the model
  *violates* still compiles: it is emitted as a `…_violated : ¬ policy` theorem,
  so the build stays green and the violation is stated as a checked fact.
  When two or more checked policies hold, a `checked_policies_consistent`
  theorem witnesses their joint satisfiability.
- **`Obligations` (the proof backlog)** holds one documented `sorry` stub per
  policy that could not be auto-grounded, with the deferral reason and the
  original expression AST in the doc comment. Strengthen these over time;
  they are *not* part of the default build target.

CI gate (see the `verify-lean` job in `.github/workflows/ci.yml`): project the
fixture, `lake build`, and fail if the checked layer reports `uses 'sorry'`.

## What is auto-groundable (v1)

A policy grounds automatically when its expression uses only:

- boolean/numeric/string literals and quantity literals,
- `and` / `or` / `not`, equality, ordering comparisons, `+` / `-`,
- `Flow.quantity` (compiled to `∀ f ∈ flows, …`).

Everything else — quantifiers, aggregations, other member accesses
(e.g. `Entity.name`), temporal operators, `*` / `/`, casts, regex — defers to
an obligation stub. The generator evaluates exactly the semantics it emits, so
an emitted `by decide` proof is guaranteed to check.

## Semantics notes

- **Quantities are exact scaled integers**: every decimal in the model is
  represented as `value × 10^(-quantityScale)` with a single shared scale, so
  comparisons are exact (no floats). Units are recorded in the source model
  but not checked in Lean (v1).
- **Determinism**: identical model + fixed `--created-at` produce
  byte-identical output. Graph collections are emitted in sorted order.
- **Toolchain pin**: `LEAN_TOOLCHAIN` in
  `domainforge-core/src/projection/lean/mod.rs` is the single source of truth.
  Bumping it is a one-line change validated by the `verify-lean` CI job.

## Non-goals (v1)

- No Mathlib/Batteries dependency in generated packages (deliberate: hermetic builds).
- No typed entity attributes (attributes are untyped in the IR today).
- No Lean 3 output.
- No `projection … target lean` contract surface — the target is CLI-format-driven.

## Local end-to-end check

The ignored test runs a real `lake build` when a Lean toolchain is installed:

```bash
cargo test --features cli -p domainforge-core -- --ignored lake_build
```

Install the toolchain with [elan](https://lean-lang.org/documentation/setup/).
