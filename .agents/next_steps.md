# Next Steps

## 1. Review and integrate the RDF instance/policy projection

Review the uncommitted diff on `feat/rdf-instance-policy-projection`
(`domainforge-core/src/kg.rs`, `domainforge-core/src/projection/rdf/ontology.rs`,
`domainforge-core/tests/turtle_instance_export_tests.rs`,
`docs/specs/SDS-005-knowledge-graph-module.md`, `docs/rdf-projections.md`), then
commit or open a pull request. Every automated gate passes; the only open action
is human review. Expected outcome: the change is integrated without staging or
rewriting the unrelated `.claude/skills/neatcode/` worktree content.

## 2. Give `ResourceInstance` a content-derived identity

`ResourceInstance` is the one declared concept still missing from RDF output. Its
identity is `Uuid::new_v4()`, so projecting it today would break the
byte-identical output guarantee. Expected outcome: a content-derived `ConceptId`
for `ResourceInstance`, after which its projection is a small follow-on change.

## 3. Decide whether policies should lower into SHACL

Policies currently project as individuals carrying their normalized expression;
they are not translated into SHACL constraints, because a partial translation
would silently change what a policy means. Expected outcome: either an explicit
decision to keep policies stated-only, or a specification of the exact expression
subset SHACL can carry faithfully plus the rejection behavior for the rest.

## 4. Restore model-checker-backed TLA evidence when available

Install Java and provide `tla2tools.jar`, then rerun `just prove`. Expected
outcome: TLA evidence upgrades from the recorded structural-only fallback to a
model-checker-backed result. This is an environment limitation, not a failing
test.

## 5. Continue the Milestone 1 contract gate

Resume ADR-014 ratification once the projection change above is reviewed. Do not
combine Milestone 1 implementation with this projection diff.

## Prior milestone next steps (superseded while the above is active)

## 1. Ratify ADR-014

Review and explicitly accept or amend
`docs/specs/ADR-014-application-review-and-approval-contract.md`. Expected
outcome: fixed, human-approved CLI, artifact, status, diff, and approval
contracts with no implementation-agent choices left open.

## 2. Write and adversarially review the Milestone 1 plan

Create
`.agents/plans/2026-07-19-conversational-application-generator-m1-review.md`
from specification Milestone 1 and the remediated Milestone 0 APIs. Expected
outcome: a test-first, codebase-grounded plan for Domain/Application IR,
inspection, domain review, semantic diff, and semantic approval capture.

## 3. Implement and gate Milestone 1

Execute one independently testable packet at a time, preserving canonical
hashing and binding parity. Expected outcome: inspect/review/diff/approval
workflows satisfy an adversarial Milestone 1 human gate without entering
provider, generation, or skill scope; explicit maintainer ratification remains
required before Milestone 2.
