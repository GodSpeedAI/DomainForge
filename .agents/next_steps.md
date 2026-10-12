# Next Steps

## Rego and OpenShell projection follow-up (2026-10-12)

1. `regorus` 0.12.0 was evaluated on 2026-10-12 and cannot evaluate the emitted bundle (0/605 corpus inputs; nested-comprehension and statement-scheduling errors). Re-evaluate on a later `regorus` release before considering embedded evaluation; OPA 1.21.1 stays the reference evaluator.
2. Verify an OpenShell provider mapping for credential-bound flows. Expected outcome: credential-bound flows can project, or the refusal is kept with the verified reason recorded.
3. Decide whether the evaluator's NULL handling becomes consistent (debt D12 in `OBSERVED_DEBT.md`). Expected outcome: one documented three-valued semantics, with `lib.rego` and the differential tests updated in the same change.

The fallible CEP world mint fix is complete: the regression and envelope suites pass (27 tests, serial run), with fmt and CLI Clippy clean. Continue the release follow-up below.

## world_ref follow-up (2026-10-04)

1. `feat/world-ref` is merged (#132, 86aaf6f). Do not cut a release from it alone; wait for #133 (expected release 0.19.0). Expected outcome: `world_ref` is published in `domainforge-core` (crates.io) and `@godspeedai/domainforge` (npm).
2. Merge #133 (world-bound `semantic_snapshot`), then let release-please cut one release carrying both. Expected outcome: `world_ref` and the world-bound snapshot are published together.
3. Bump every consumer (sea-forge, Cognate, RealityTrace and their pinned constants) to that release together. Expected outcome: no version skew, as in Stage 1 of the cross-repo migration.

The 2026-09-26 Turtle IRI delimiter fix has no remaining code or test steps. If this task later enables CodeRabbit review or provides graft, review the current diff and rebuild the index; then continue the repository priorities below.

Current handoff (2026-09-25): `fix/debt-003-domainforge-accessors` merged to `main` and pushed; release-please opens the release PR from the `feat` commit (expect a minor bump from 0.17.0). Watch for the release PR titled `chore: release ...`, merge it to publish `@godspeedai/domainforge` plus the Python/Rust artifacts, then report the new version number for the Cognate pin bump. D8–D10 debt entries carry their fix evidence; fill `[RESOLVED <sha>]` with the merge commit at that point.

## 1. Maintain Documentation Synchronization upon ADR-014 Ratification

When `docs/specs/ADR-014-application-review-and-approval-contract.md` is ratified and Milestone 1 implementation begins, update `docs/subsystems/application-contracts.md` and `docs/reference/cli-reference.md` to reflect the settled inspection, review, and semantic approval commands. Expected outcome: documentation remains 100% synchronized with the implementation without drift.

## 2. Close the RDF completeness gaps: `ResourceInstance` identity and `to_graph()` reconstruction

`ResourceInstance` is the one declared concept still missing from RDF output — its identity is `Uuid::new_v4()`, so projecting it today would break the byte-identical output guarantee. Separately, `KnowledgeGraph::to_graph()` reconstructs `Entity`/`Resource`/`Flow`/`Relation` from Turtle but not the `sea:EntityInstance`/`sea:Policy` triples `to_turtle()` now emits. Expected outcome: a content-derived `ConceptId` for `ResourceInstance`, plus either a `to_graph()` that reconstructs instances/policies or one that errors instead of silently dropping them.

## 3. Decide whether policies should lower into SHACL

Policies currently project as individuals carrying their normalized expression; they are not translated into SHACL constraints, because a partial translation would silently change what a policy means. Expected outcome: either an explicit decision to keep policies stated-only, or a specification of the exact expression subset SHACL can carry faithfully plus the rejection behavior for the rest.
