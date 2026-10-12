# Rego projection

## Scope

`--format rego` lowers the policies of a `.sea` model into an OPA/Rego
bundle that evaluates the model's policies over an input document of facts.
The projection covers a narrow policy subset. Policies outside that subset
are refused, not approximated.

The projection is an Authority-family target (see
`docs/projection-families.md`). It emits policy code and a decision contract.
It does not run OPA, does not enforce anything, and does not choose what an
integrator does with an `escalate` or `deny` outcome.

The per-policy semantics are those of DomainForge's own evaluator in its
default three-valued mode (`Policy::evaluate_with_mode`). OPA is the reference
evaluator for the emitted code; the differential tests check the emitted code
against the DomainForge evaluator, not against a reimplementation of it.

## Usage

```bash
domainforge project --format rego \
  --created-at 2026-07-11T00:00:00Z \
  fixtures/cell_env/repair-agent/model.sea out/rego
```

`--created-at` (RFC 3339) is the only value that varies between runs. For a
fixed model and `--created-at`, the output is byte-identical; only
`provenance.json` carries the timestamp.

Generated tree:

```text
out/rego/
├── bundle/
│   ├── .manifest
│   ├── data.json
│   └── domainforge/
│       ├── sea/
│       │   └── lib.rego
│       └── policy/
│           ├── decision.rego
│           └── policies.rego
├── contracts/
│   ├── input.schema.json
│   └── decision.schema.json
├── fixtures/
│   ├── model.json
│   ├── empty.json
│   ├── incomplete.json
│   └── malformed.json
├── tests/
│   └── decision_test.rego
├── fidelity-report.json
└── provenance.json
```

`scripts/verify/projection-targets/rego.sh` runs the two-run `diff -r`, the
OPA toolchain checks, and the differential tests (see "Verification").

## Semantics

### Supported policy constructs

Policies are lowered from the resolved graph into a narrow policy IR. The
following constructs are supported:

| Construct | Form |
|---|---|
| Boolean literals | `true`, `false` |
| Boolean connectives | `and`, `or`, `not` |
| Comparisons between scalar literals, fields of quantifier-bound items, and counts | `=`, `!=`, `<`, `<=`, `>`, `>=` |
| String predicates | `contains`, `startswith`, `endswith` |
| Quantifiers | `forall`, `exists`, `exists_unique` over `entities`, `resources`, `flows`, `instances`, `relations`, `entity_instances` (optionally `of "Type"`) |
| Counts | `count(collection)`, `count(v in collection where ...: v)` |

Anything else fails the whole projection (see "Error codes").

### Per-policy results

Each policy evaluates to one of four results:

| Result | Meaning |
|---|---|
| `true` | the policy holds |
| `false` | the policy is violated |
| `unknown` | the evaluator's NULL |
| `error` | the evaluator returned `Err` |

`unknown` and `error` are represented as explicit result values. Rego
`undefined` is never used to represent either.

### Decision contract

The decision contract is `domainforge.rego.decision/v1`, exposed at
`data.domainforge.policy.decision`. Its document is
`{contract, outcome, results, reasons, input_errors, model}`.

`outcome` is determined in this order:

1. `deny` if the input is malformed, or any error-severity policy is
   violated or fails to evaluate.
2. otherwise `escalate` if any error-severity policy is `unknown`.
3. otherwise `allow`.

Severity follows the modality as in the evaluator: Obligation and Prohibition
are error-severity; Permission is info-severity and never blocks. Priority
orders the entries of `reasons` only. It does not affect `outcome`.

### Fidelity classes

`fidelity-report.json` classifies each part of the output:

| Part | Class | Meaning |
|---|---|---|
| Per-policy results | `isomorphic` | equal to `Policy::evaluate_with_mode` in three-valued mode |
| Aggregate decision (`outcome`) | `adapter` | the evaluator has no aggregate outcome; the rule is defined by this projection |
| Policy metadata | `structural` | names, modalities, and identifiers carried as data |
| Unsupported constructs | `unrepresentable` | the projection fails with `REGO001` |

## Error codes

| Code | Meaning |
|---|---|
| REGO001 | `unrepresentable: policy '<ns>::<name>' uses <construct>: <reason>`. The construct is outside the supported subset. |
| REGO002 | model error (the model cannot be projected) |
| REGO003 | write error (output could not be written) |

Constructs that produce `REGO001`: arithmetic; `matches`; `has_role`;
temporal operators and literals; quantities and casts; `group_by`;
`sum`, `min`, `max`, `avg`; windowed aggregations; role references; free
variables (for example operation-context terms); a bare reference used as a
condition; non-boolean literals in boolean position; operation-bound policies.

## Verification

`scripts/verify/projection-targets/rego.sh` requires `opa` 1.21.1 on `PATH`
and fails if it is missing. It runs:

1. Two projections with the same `--created-at`, compared with `diff -r`.
2. `opa check --strict --v1-compatible bundle tests`
3. `opa fmt --v1-compatible --fail --list bundle`
4. `opa test --v1-compatible bundle tests` (four generated cases)
5. `opa build --bundle bundle --output bundle.tar.gz`
6. `cargo test -p domainforge-core --features cli --test rego_projection_tests -- --include-ignored`

The differential tests (`domainforge-core/tests/rego_projection_tests.rs`,
9 tests, three of them requiring `opa` on `PATH`) run real OPA against 600
generated inputs per model, plus hand-written malformed inputs. For each
input, the full decision document must equal the reference computed by
DomainForge's evaluator. The policy's collections are rebound to the input
facts as literal arrays, so the reference is the evaluator itself.

The tests also require that the inputs reach every per-policy result
(`true`, `false`, `unknown`, `error`) and every outcome (`allow`, `deny`,
`escalate`).

## Known limitations

- **Numeric comparison**: numbers compare exactly in OPA and as `f64` in the
  evaluator, so results can differ for values beyond 2^53.
- **Equality on array- and object-valued fields**: integer and float forms of
  the same number are equal in OPA and different in the evaluator.
- **Evaluation mode**: only the evaluator's default three-valued mode is
  projected.
- **Three-valued logic is not Kleene throughout the evaluator**: see the
  recorded debt in `.agents/OBSERVED_DEBT.md`. The Rego library reproduces the
  current evaluator behaviour exactly, including its NULL handling. It is not
  Kleene logic as described in `docs/`.
- **`regorus` is not compatible**: `regorus` 0.12.0 (default features off; its
  default feature set conflicts with this crate's pinned `uuid`) loads the
  generated modules and data but fails at evaluation on every well-formed
  input (0 of 605 corpus inputs matched; OPA 1.21.1 evaluated all 605). The
  errors were ``var `v1` used before definition below`` on a nested
  comprehension in `policies.rego` and `statements not scheduled in query` on
  the `input_errors` rule in `decision.rego`. The generated Rego was not
  changed to accommodate it. OPA 1.21.1 is the only supported evaluator;
  consumers that need embedded evaluation must run the `opa` binary.
- **Unsupported constructs are refused**: a model that uses one of the
  constructs listed above does not project. There is no partial output.
