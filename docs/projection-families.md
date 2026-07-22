# Projection Families

DomainForge projects its canonical semantic graph into mature external
ecosystems while preserving deterministic identity, semantic traceability, and
projection consistency. Each target is an **operator family** — a class of
external tool that operates over a projection of the model — following the
architecture named in
[ADR-011: Operator-Backed Projection Families](specs/ADR-011-operator-backed-projection-families.md).

Every family shares one shape: one IR module, one renderer, one
`emit(graph, …, &mut ArtifactSink)`, one `project_<x>_in_memory` binding
surface, one proving fixture, and one external-toolchain CI `verify-<family>`
job. See the [CLI reference](reference/cli-commands.md#project) for full flag
details, or run `domainforge project --help` — the format doc comments print as
this same operator table.

## The nine operator families

| Operator family | Projects to | CLI format | Reference | CLI behavior |
| --- | --- | --- | --- | --- |
| Semantic graph | RDF / OWL (Turtle + JSON-LD + OWL) | `--format rdf` | [RDF/OWL Projection](rdf-projections.md) | [RDF-specific behavior](reference/cli-commands.md#rdf-specific-behavior) |
| Ordered process | BPMN 2.0 XML | `--format bpmn` | [BPMN Projection](bpmn-projections.md) | [BPMN-specific behavior](reference/cli-commands.md#bpmn-specific-behavior) |
| Adaptive work / case | CMMN 1.1 XML | `--format cmmn` | [CMMN Projection](cmmn-projections.md) | [CMMN-specific behavior](reference/cli-commands.md#cmmn-specific-behavior) |
| Enterprise architecture | ArchiMate 3.0 Model Exchange File | `--format archimate` | [ArchiMate Projection](archimate-projections.md) | [ArchiMate-specific behavior](reference/cli-commands.md#archimate-specific-behavior) |
| Runtime observation | OpenTelemetry SemConv registry + constants | `--format otel-semconv` | [OTel SemConv Projection](otel-projections.md) | [OTel SemConv-specific behavior](reference/cli-commands.md#otel-semconv-specific-behavior) |
| AI representation | BAML capability (`.baml`) | `--format baml` | [BAML Projection](baml-projections.md) | [BAML-specific behavior](reference/cli-commands.md#baml-specific-behavior) |
| AI optimization | DSPy program (Python) | `--format dspy` | [DSPy Projection](dspy-projections.md) | [DSPy-specific behavior](reference/cli-commands.md#dspy-specific-behavior) |
| Learning loop | ZenML pipeline (Python) | `--format zenml` | [ZenML Projection](zenml-projections.md) | [ZenML-specific behavior](reference/cli-commands.md#zenml-specific-behavior) |
| Formal verification | Lean 4 Lake package | `--format lean` | [Lean 4 Projection](lean-projections.md) | [Lean-specific behavior](reference/cli-commands.md#lean-specific-behavior) |

## Related learning-loop datasets

The learning-loop family also emits the AI-learning datasets that DSPy and
ZenML reference (by path, never copied): LLM JSONL, graph-ML, and CEP-evaluation
datasets via `--format ai-llm` / `ai-graph-ml` / `cep-eval` / `ai-learning`. See
[AI Learning Projections](ai-learning-projections.md).

## Determinism and identity

All minted identifiers route through the shared `projection::ids` module
(content hashes of a U+0001-joined tuple, xxh64 seed 42). A fixed `--created-at`
yields byte-identical output, enforced by a `verify-<family>` CI job and a
byte-determinism test per family. See ADR-011 for the full convention.

## Event, authority, verification, and activation targets

Eight additional targets share the same kernel shape. See
[Projection Target Implementation Status](projection-target-implementation-status.md)
for gate/toolchain details.

| Operator family | Projects to | CLI format | Notes |
| --- | --- | --- | --- |
| Event | CloudEvents 1.0 stream | `--format cloudevents` | One envelope per Flow |
| Event-API | AsyncAPI 3.0 YAML | `--format asyncapi` | Channels/operations/messages per Flow; spec-validated |
| Activation | Devbox manifest | `--format devbox` | Domain-aware dev shell |
| Activation | Dagger module | `--format dagger` | One `@dagger.function` per Flow |
| Activation | Cell environment | `--format cell` | Devbox + Mise + dependency-set + sandbox/network + authority + evidence + `cell.lock` for a `Cell` declaration (see `docs/cell-environment-projections.md`) |
| Authority | Cedar schema + policies | `--format cedar` | **Permissive baseline** (see below) |
| Verification | Gauge spec | `--format gauge` | One scenario per Flow |
| Verification | Alloy model | `--format alloy` | Sigs + facts per Flow |
| Verification | TLA+ spec | `--format tla` | State-machine; SANY+TLC verified |

### Cedar authority scope

The Cedar `policies.cedar` file is a **permissive baseline**: one `permit`
per Action, scoped to the flow's source entity type and resource type. It
does not project SEA `policy` expressions as Cedar `forbid`/`when` clauses.
A Cedar engine loaded with this baseline authorizes the model's declared
flows, not its full obligation set. Projecting SEA `policy` obligations into
Cedar policy clauses is future work.

## Code targets (DDD/CQRS domain layer)

Three code targets share one language-neutral Domain IR
(`projection/domain/ir.rs`) and three pure renderers (Python, TypeScript,
Rust). All DDD semantics are decided once in the IR; each renderer only
translates IR → language syntax. Output is a complete, ready-to-use package
up to the **port / abstract-base boundary** (no infrastructure adapters).
See [Project Domain Code](how-tos/project-domain-code.md) for the normative
SEA→DDD mapping and the `@cqrs` annotation switch.

| Operator family | Projects to | CLI format | Notes |
| --- | --- | --- | --- |
| Code | Python DDD/CQRS package | `--format domain-python` | stdlib only; `compileall` + `mypy --strict` + `unittest` verified |
| Code | TypeScript DDD/CQRS package | `--format domain-typescript` | zero runtime deps; `tsc --noEmit` (strict) verified |
| Code | Rust DDD/CQRS crate | `--format domain-rust` | zero deps; `cargo check` + `cargo test` verified |
</content>
</invoke>
