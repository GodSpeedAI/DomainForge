# Next Steps

## 1. Maintain Documentation Synchronization upon ADR-014 Ratification

When `docs/specs/ADR-014-application-review-and-approval-contract.md` is ratified and Milestone 1 implementation begins, update `docs/subsystems/application-contracts.md` and `docs/reference/cli-reference.md` to reflect the settled inspection, review, and semantic approval commands. Expected outcome: documentation remains 100% synchronized with the implementation without drift.

## 2. Close the RDF completeness gaps: `ResourceInstance` identity and `to_graph()` reconstruction

`ResourceInstance` is the one declared concept still missing from RDF output — its identity is `Uuid::new_v4()`, so projecting it today would break the byte-identical output guarantee. Separately, `KnowledgeGraph::to_graph()` reconstructs `Entity`/`Resource`/`Flow`/`Relation` from Turtle but not the `sea:EntityInstance`/`sea:Policy` triples `to_turtle()` now emits. Expected outcome: a content-derived `ConceptId` for `ResourceInstance`, plus either a `to_graph()` that reconstructs instances/policies or one that errors instead of silently dropping them.

## 3. Decide whether policies should lower into SHACL

Policies currently project as individuals carrying their normalized expression; they are not translated into SHACL constraints, because a partial translation would silently change what a policy means. Expected outcome: either an explicit decision to keep policies stated-only, or a specification of the exact expression subset SHACL can carry faithfully plus the rejection behavior for the rest.
