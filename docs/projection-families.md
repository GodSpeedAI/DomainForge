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
</content>
</invoke>
