# Next Steps

## 1. Close the RDF completeness gaps: `ResourceInstance` identity and `to_graph()` reconstruction

`ResourceInstance` is the one declared concept still missing from RDF output —
its identity is `Uuid::new_v4()`, so projecting it today would break the
byte-identical output guarantee. Separately, `KnowledgeGraph::to_graph()`
reconstructs `Entity`/`Resource`/`Flow`/`Relation` from Turtle but not the
`sea:EntityInstance`/`sea:Policy` triples `to_turtle()` now emits, so a
`to_turtle` -> `from_turtle` -> `to_graph` round trip silently drops instances
and policies (documented at the `to_graph()` call site). Expected outcome: a
content-derived `ConceptId` for `ResourceInstance`, plus either a `to_graph()`
that reconstructs instances/policies or one that errors instead of silently
dropping them.

## 2. Decide whether policies should lower into SHACL

Policies currently project as individuals carrying their normalized expression;
they are not translated into SHACL constraints, because a partial translation
would silently change what a policy means. Expected outcome: either an explicit
decision to keep policies stated-only, or a specification of the exact expression
subset SHACL can carry faithfully plus the rejection behavior for the rest.

## 3. Resume the ADR-014 contract gate

Ratify `docs/specs/ADR-014-application-review-and-approval-contract.md`, then
proceed with the Milestone 1 plan (Domain/Application IR, inspection, domain
review, semantic diff, semantic approval capture) as one independently testable
packet at a time. Expected outcome: fixed, human-approved contracts with no
implementation-agent choices left open, and inspect/review/diff/approval
workflows satisfying an adversarial Milestone 1 human gate before Milestone 2.
