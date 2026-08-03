# Current State: SEA Interaction Interventions I1-I3 Implemented

## I1-I3 execution (2026-08-03)

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
  extension. The binding will be rebuilt non-destructively before final gates.
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

Human Gates A and B and the Milestone 0 human gate are CLOSED. Milestone 1
contract settlement is proposed in ADR-014. Plan finalization and implementation
wait for explicit ADR-014 ratification. Later milestone gates remain closed to
self-approval: the repository maintainer must explicitly accept each reviewed
milestone.
