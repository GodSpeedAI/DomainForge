# Current State: RDF Instance and Policy Projection Implemented

## RDF instance/policy projection (2026-08-06)

Branch: `feat/rdf-instance-policy-projection`. No commits ahead of `main`; the
entire change is uncommitted worktree content.

Problem addressed: `KnowledgeGraph::from_graph` iterated only entities, roles,
resources, patterns, relations, and flows, so every authored `Instance` and
`Policy` was silently dropped from `model.ttl`, `model.jsonld`, and
`ontology.owl.ttl`.

- [x] Instance and policy triples emitted. Evidence: `domainforge-core/src/kg.rs`
  emits each instance as `sea:instance_<name>` typed `sea:EntityInstance`, linked
  by `sea:instanceOf`, with one `sea:<fieldName>` triple per field; each policy
  as `sea:policy_<name>` typed `sea:Policy` carrying expression, modality, kind,
  priority, rationale, and tags. Field keys and tags are sorted before emission
  because `Instance::fields` is a `HashMap` and unsorted iteration would break
  byte-identical output.
- [x] Ontology axioms emitted. Evidence:
  `domainforge-core/src/projection/rdf/ontology.rs` adds the `EntityInstance` and
  `Policy` classes, the `instanceOf` object property, the `expression`/
  `modality`/`priority` data properties, and named individuals — all gated on the
  model actually declaring instances or policies, so nothing appears
  speculatively.
- [x] Test suite added. Evidence:
  `domainforge-core/tests/turtle_instance_export_tests.rs`, 9 tests covering
  Turtle subjects, field datatypes, policies, ontology individuals, JSON-LD typed
  values, determinism across 8 reprojections, the graph-wide instance-name
  uniqueness invariant, distinct instances of different entities, and the
  no-instances model projecting unchanged.
- [x] Documentation updated. Evidence: `docs/specs/SDS-005-knowledge-graph-module.md`
  §4.2–§4.5 and `docs/rdf-projections.md`.
- [x] Corrected a doc/code contradiction left by the prior agent. The docs
  specified `sea:instance_<EntityType>_<name>`; the implementation emits
  `sea:instance_<name>`. The docs were corrected to match the code: the name
  alone is sufficient because `Graph::insert_entity_instance` rejects duplicate
  names graph-wide, and omitting the entity type means retyping an instance does
  not change its identity. The invariant is pinned by
  `instance_names_are_unique_graph_wide`.
- [x] Verified the SDS-005 §4.4 canonicalization claim against the code.
  `is_minted_node` (`domainforge-core/src/projection/rdf/mod.rs:71`) matches only
  `sea:flow_` and `sea:pattern_`, so instance and policy IRIs are never rewritten
  by `canonicalize_node_ids`.

### Deliberate non-goals in this change

- `ResourceInstance` is not projected. Its identity is `Uuid::new_v4()` rather
  than a content-derived `ConceptId`, so emitting it would break determinism. A
  content-derived identity is the prerequisite, not a projection change.
- Policies are stated, not lowered into SHACL. A partial translation would
  silently change what a policy means.
- `FieldType::EntityRef` emits the referenced key value as a literal, not an IRI,
  so an unvalidated model cannot produce dangling references.

### Gate evidence (2026-08-06)

| Gate | Result |
| ---- | ------ |
| `cargo fmt --check` | clean (rustfmt applied to the new code first) |
| `cargo clippy --all-targets --features cli -- -D warnings` | clean |
| `just rust-test` | pass |
| `just ai-validate` | pass |
| `just ts-test` | 197 pass / 0 fail |
| `just python-test` | 241 pass / 5 skipped |

Cross-binding evidence is meaningful only against freshly built artifacts: the
napi binding (`bun run build` in `domainforge-typescript/`) and the maturin
extension (`just python-setup`) were both rebuilt from current source before the
binding suites were run. That rebuild also resolved the stale-native-extension
Python golden mismatch recorded on 2026-08-03.

### Environment change

`bun` was absent, and the `nub` shim refuses npm while `bun.lock` exists, which
blocked the TypeScript gate entirely. bun 1.3.14 is now installed at
`~/.bun/bin/bun` with a guarded PATH block appended to `~/.bashrc`. Repository
dependencies were installed with `bun install --frozen-lockfile`; `bun.lock` and
`package.json` are unchanged.

## Prior state: SEA Interaction Interventions I1-I3 (2026-08-03)

- [x] Read repository instructions and the governing limitations report.
  Evidence: `AGENTS.md`, `.github/copilot-instructions.md`, and
  `.agents/reports/2026-08-03-sea-interaction-model-limitations-interventions.md`.
- [x] Verified the report against current parser, application contract, graph,
  module resolver, CLI, policy, binding, projection, and compatibility paths.
  Evidence: typed entity bodies are discarded at graph conversion; the
  filesystem CLI converts only the entry AST; policies expose resource
  `instances` but not entity instances. The report overstates existing
  instance-value validation: application resolution validates declarations and
  defaults, not concrete `Instance` values.
- [x] Established baseline. Evidence: `just ai-validate` passed; `just ts-test`
  passed 196/196; `just python-test` had one pre-existing cross-binding golden
  mismatch while all other Python tests passed, consistent with a stale native
  extension. That mismatch is resolved as of the 2026-08-06 rebuild above.
- [x] Recorded executable plan. Evidence:
  `.agents/plans/2026-08-03-sea-interaction-i1-i3.md`.
- [x] I1 typed graph contract and entity-instance validation implemented.
  Evidence: graph-owned entity/enum contracts plus focused validation tests for
  required and optional fields, supported constraints and scalar types, enums,
  typed references, key uniqueness, and dangling references.
- [x] I2 filesystem CLI closure implemented. Evidence: file-based parse,
  validate, and generic project commands use the existing deterministic module
  resolver over the transitive closure; focused tests prove imported
  same-namespace instances resolve.
- [x] I3 entity-instance policy bindings implemented. Evidence:
  `entity_instances` plural quantification and `entity_instance` singular
  bindings are covered by parsed and aggregate policy tests.
- [x] Focused, full, compatibility, projection, and faithful-fixture evidence
  complete. Evidence:
  `.agents/reports/2026-08-03-i1-i3-verification.md` and
  `evidence/latest/proof.{json,md}`. The original SEA Forge interaction model
  validates unchanged; the modular fixture accepts the formerly failing
  imported-instance shape and rejects its intentionally invalid variant.

## Prior state

Branch: `agent/projection-targets`. Accepted ADR-013 baseline: `1320fed`.
Milestone 0 implementation and remediation run through `56895f2`.

## Milestone 0 gate (2026-07-19)

- [x] Implemented the 15-task language plan. Evidence: commits `3550baf`
  through `e3e82a4` and the implementation paths named in
  `.agents/plans/2026-07-18-conversational-application-generator-m0-language.md`.
- [x] Completed adversarial human-gate review. Evidence:
  `.agents/reports/2026-07-19-m0-human-review-gate.md`.
- [x] Remediated the review findings. Evidence: commits `cc6036f` through
  `56895f2` cover patch hygiene, strict schemas, field semantics, shared
  identities, resolved envelope references, canonicalization, persisted
  metadata, diagnostics, resource budgets, compatibility evidence, semantic
  packs, and rebuilt cross-binding parity.
- [x] Human explicitly accepted Milestone 0. Evidence: repository maintainer
  instruction on 2026-07-19: “milestone 0 is done just accept it”. The gate
  disposition is recorded in the review report.
- [x] Drafted the required Milestone 1 public-contract settlement. Evidence:
  `docs/specs/ADR-014-application-review-and-approval-contract.md` fixes exact
  commands, artifact schemas, approval arguments, statement binding, statuses,
  exit codes, and pure-core/binding ownership.
- [ ] ADR-014 is accepted. Proposed pending explicit maintainer ratification;
  the executable plan cannot safely freeze its interfaces before this gate.
- [ ] Milestone 1 executable plan is complete and reviewed. Blocked on ADR-014;
  expected artifact is
  `.agents/plans/2026-07-19-conversational-application-generator-m1-review.md`.
- [ ] Milestone 1 implementation is accepted. Not started; implementation may
  begin only from the completed Milestone 1 plan.

## Gate status

Human Gates A and B and the Milestone 0 human gate are CLOSED. The RDF
instance/policy projection change passes every automated gate but has NOT been
reviewed or committed; maintainer review is the open action. Milestone 1
contract settlement remains proposed in ADR-014, with plan finalization and
implementation waiting on explicit ratification. Later milestone gates remain
closed to self-approval: the repository maintainer must explicitly accept each
reviewed milestone.
