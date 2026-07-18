# Audit Report: Event, Authority, Verification, and Activation Projections Plan

**Date:** 2026-07-08 (revised)
**Plan audited:** `.agents/plans/domainforge_event_authority_verification_activation_projections_plan.md`
**Revision note:** The original version of this audit contained fabricated grounding evidence — file paths and fixture conventions that do not exist in this repo. This revision replaces every citation with a path verified against the working tree on 2026-07-08 and calls out where the original audit was itself wrong, not just the plan.

## Findings Addressed (plan-level, carried over — verified correct)

- The plan cited `/mnt/data/semantic-projection.md`, `/mnt/data/write-in-sea.md`, and `/mnt/data/domainforge-strategic-archive.md` as source material, but those files are absent in this workspace. The plan now states that current repository code and docs are the implementation source of truth.
- The plan treated `ProjectionRegistry` as a CLI target registry. Repo inspection confirms `ProjectionRegistry` (`domainforge-core/src/projection/registry.rs:5`) only looks up SEA `Mapping` / `Projection` contracts (`find_mappings_for_target`), while CLI targets live in `domainforge-core/src/cli/project.rs` as the `ProjectFormat` enum. This distinction is real and the plan's correction stands.
- The plan used `examples/projection-cell/projection-cell.sea`, a path that does not exist. Confirmed — no such file or directory exists anywhere in the repo.
- The Task 1 gate referenced the wrong plan filename; it now checks the correct plan path.
- The plan suggested using SEA `Mapping`/`Projection` syntax for new targets without noting the current grammar only accepts `calm`, `kg`, `sbvr`, `protobuf`, `proto` in `target_format` (`domainforge-core/grammar/sea.pest:33` region has declarations; the `target_format` rule itself is at line 373: `^"calm" | ^"kg" | ^"sbvr" | ^"protobuf" | ^"proto"`). Confirmed correct.

## Findings Addressed (new — errors in the original audit itself)

The original audit's "Grounding Evidence" list cited several paths that **do not exist in this repository**. These were fabricated (or copied from a different codebase/branch) and would have sent an implementing agent looking for files that were never there:

- `domainforge-core/src/projection/otel/mod.rs` and `domainforge-core/src/projection/otel/ir.rs` — **do not exist**. The actual `domainforge-core/src/projection/` directory contains only `buf.rs`, `contracts.rs`, `engine.rs`, `mod.rs`, `protobuf.rs`, `registry.rs`. There is no `otel` submodule, no per-family projection module pattern to "mirror" as claimed.
- `domainforge-core/tests/common/projection_harness.rs` and `domainforge-core/tests/otel_projection_tests.rs` — **do not exist**. There is no `tests/common/` directory at all. `domainforge-core/tests/` has ~80 flat test files (e.g. `protobuf_projection_tests.rs`, `calm_round_trip_tests.rs`, `sbvr_parsing_tests.rs`) but no otel-family harness or shared harness module.
- `docs/projection-families.md` — **does not exist**. `docs/` has no file by this name (confirmed via directory listing of `docs/`).
- `fixtures/otel/basic/domain/model.sea`, `fixtures/bpmn/basic/domain/model.sea`, `fixtures/baml/basic/domain/model.sea` — **none exist**. `fixtures/` contains exactly one subtree: `fixtures/semantic_packs/acme_procurement/{domain,tests,review}/*.sea`. There is no `fixtures/<family>/basic/domain/model.sea` convention anywhere in the repo — this pattern was invented, not observed.
- Stray `.pyc` files (`tests/__pycache__/test_otel.cpython-*.pyc`, `test_bpmn.*.pyc`, `test_baml.*.pyc`) exist with no corresponding `.py` source in the tree. These are compiled-cache leftovers from deleted or never-committed test files, not evidence of a live otel/bpmn/baml projection feature. The original audit likely inferred module existence from these filenames without checking for live source.

Because the plan's "Key facts already discovered (do not re-derive)" table was built directly from this audit, it inherited every one of the above fabrications verbatim (see plan lines 69, 72–75). An agent executing Task 1 today would fail to find any of these files and could either stall or start inventing structure to match, defeating the point of an audit.

## Additional gap the original audit missed entirely

- `docs/reference/cli-commands.md` — this file **does exist** (correctly cited), but its content is itself stale relative to the code and the original audit did not flag the drift. The docs describe:
  ```
  domainforge project --format <calm|rdf|sbvr|dsl|protobuf> input.sea output.json
  ```
  The actual `ProjectFormat` enum (`domainforge-core/src/cli/project.rs:64-69`) only has four variants: `Calm`, `Kg`, `Protobuf`, `Proto` (alias). There is no `Sbvr` or `Dsl` variant in the CLI dispatcher at all, despite `sbvr` being a valid grammar `target_format` and having dedicated parser test coverage (`sbvr_parsing_tests.rs`, `sbvr_fact_schema_tests.rs`, `sbvr_flow_facts_tests.rs`). `rdf` in the docs corresponds to the `Kg` variant's `--format rdf` extension-based branch (`.rdf`/`.xml` → `to_rdf_xml()`), not a separate enum value — the docs use different names than the code. This means:
  - The CLI target surface is smaller than both the grammar's `target_format` list and the docs imply.
  - Task 1's "Re-check the pinned file:line facts" step needs an explicit sub-step to reconcile `docs/reference/cli-commands.md` against `ProjectFormat` before treating either as ground truth, since they currently disagree with each other and with the code.

## Grounding Evidence (corrected, verified 2026-07-08)

- Grammar declaration and current target list: `domainforge-core/grammar/sea.pest:373` (`target_format` rule: `calm | kg | sbvr | protobuf | proto`)
- CLI target enum and dispatcher: `domainforge-core/src/cli/project.rs:64-69` (`ProjectFormat` enum: `Calm`, `Kg`, `Protobuf`, `Proto` — no `Sbvr`, no `Dsl`), dispatch `match` at `domainforge-core/src/cli/project.rs:111-141`
- SEA contract registry (mapping/projection lookups only, not a CLI target registry): `domainforge-core/src/projection/registry.rs:5,14`
- Actual projection module set to mirror (no per-family submodule exists yet): `domainforge-core/src/projection/{buf,contracts,engine,mod,protobuf,registry}.rs`
- Existing fixture convention (the only one that exists): `fixtures/semantic_packs/acme_procurement/domain/*.sea`, `fixtures/semantic_packs/acme_procurement/tests/*.sea` — flat `.sea` files per concern, not a `<family>/basic/domain/model.sea` tree
- CLI docs (stale relative to code, needs reconciliation before use as ground truth): `docs/reference/cli-commands.md:70-98`
- Confirmed absent: `docs/projection-families.md`, `domainforge-core/tests/common/`, any `otel`/`bpmn`/`baml` Rust module or fixture directory

## Residual Risk

- **Blocking:** The plan's Task 1 ("Re-check the pinned file:line facts above if the branch has changed") is not sufficient cover here — the facts were never accurate on this branch, not stale from drift. Task 1 must independently re-derive the projection module/fixture/test conventions from the corrected Grounding Evidence above rather than trusting the plan's pinned table.
- The plan's per-task steps that say "using the existing target pattern" (Tasks 2–9, each citing an otel-style module to mirror) have no real precedent to mirror. Each of those tasks will need to establish the first per-family projection module convention from scratch, which is materially more work than "add one more target following the pattern."
- No implementation or validation scripts for the eight planned targets exist yet; this was true in the original audit and remains true.
- `docs/reference/cli-commands.md` should be corrected (or the drift explicitly documented) as part of Task 1, since two later tasks depend on extending the same CLI surface docs describe today.
- Existing repo has many untracked/modified files unrelated to this audit. They were not changed or reverted.
