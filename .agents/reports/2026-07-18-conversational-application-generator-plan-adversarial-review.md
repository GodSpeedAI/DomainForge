# Adversarial Review: Conversational Application Generator Plan

Date: 2026-07-18

Reviewed artifact:
`.agents/plans/2026-07-18-conversational-application-generator.md`

## Outcome

The original 17-task plan was not safe to execute. It attempted to plan six
implementation milestones before the SEA application language, semantic
ownership, stable interfaces, and acceptance fixture had been decided. The
replacement is a three-task Gate 0 plan: establish evidence and decisions,
ratify ADR-013 plus the normative contract, then write a separate grammar-first
Milestone 0 code plan. All later implementation remains blocked until those
interfaces are real.

## Findings and dispositions

| Severity | Finding | Disposition in revised plan |
|---|---|---|
| Critical | Downstream code was planned before the language contract and ownership model existed. | Removed speculative implementation tasks. Added D1-D10 decision records and two human stop gates. |
| Critical | The flagship acceptance case used placeholders and had no preregistered behavior. | D9 now requires an exact command, query, policy decision, failure paths, persistence, rollback/restart, projections, deterministic outputs, and outcomes. |
| Critical | Large tasks lacked executable RED/GREEN packets and exact interfaces. | Gate 0 is documentation-only. Task 3 requires the follow-on code plan to specify exact tests, interfaces, commands, and small independently reviewable changes. |
| Critical | Several shell commands were invalid for the actual CLI. | The plan documents and removes multi-file parse/format globs, bad positional ordering, and ambiguous test filtering. Commands use `just` or `devbox run -- cargo` because direct `cargo` is not available in the reviewed shell. |
| Critical | Grammar work omitted AST schema conversion, printers, module resolution, fixtures, and binding parity. | Task 3 explicitly requires all of these in the Milestone 0 continuation plan. |
| Critical | Provider eligibility required integration proof before integration existed. | Provider implementation is deferred; the circular maturity rule is listed as a mandatory downstream blocker. |
| Critical | Approval capture lacked a settled CLI and state-machine contract. | Approval CLI, persistence, resume, expiry, and non-interactive behavior must be settled before an approval implementation plan. |
| Critical | The JSON status vocabulary conflicted with the documented generated exit state. | Status enum and exit semantics must be reconciled before CLI planning. |
| Critical | Reusing canonicalization risked changing existing semantic-pack hashes and signatures. | D10 separates typed envelope normalization from the existing pack canonical JSON primitive and requires compatibility tests. |
| Required | Milestone and skill ownership in the plan differed from the governing spec. | The downstream sequence is aligned to M0-M5, and M2 skill ownership must be reconciled before implementation. |
| Required | Draft ADR numbering, status, and ratification gates were vague. | ADR-013 is exact, starts Proposed, becomes Accepted only after an identified human and date, and cannot be silently renumbered or overwritten. |
| Required | Wrapping `ArtifactSink` could not guarantee duplicate detection because existing inner writes bypassed the wrapper. | Future planning must define and migrate the exact sink signature and all call sites before parallel emitters. |
| Required | Filesystem and subprocess security contracts were underdefined. | Future plans must settle root confinement, link policy, atomic publication, environment, timeouts, output bounds, and cleanup before execution features. |
| Required | Tasks were too large for reliable review or recovery. | Gate 0 has only three bounded tasks. Each later milestone gets its own plan with small changes and proof boundaries. |
| Required | Optional branches, generated tree shape, dependency policy, and profile limits were unspecified. | These are named downstream blockers and cannot be delegated to emitter or provider defaults. |
| Required | Fixed `/tmp` paths permitted collision and destructive cleanup. | Future execution contracts require unique temporary directories and explicit ownership/cleanup rules. |

## Repository grounding checked

- `ParseArgs` and `FormatArgs` each accept one input path.
- The CLI has no `application` or `provider` command group today.
- `domainforge-core/Cargo.toml` does not directly declare the proposed web and
  database runtime stack.
- `DomainIr` and current renderers do not preserve typed application contracts.
- ADR-012 is Proposed and records its direct-AST path as an unresolved
  departure from ADR-011, so it is not a safe precedent.
- The SEA language evolution policy requires grammar-first work plus AST,
  schema, formatter, fixtures, tests, and binding compatibility.
- ADR-013 was not present during review.
- `.gitignore` has an unrelated existing modification removing `.agents/`, so
  `.agents/` is currently untracked. This review does not modify that file.

## Review method

The plan was checked against the governing spec, repository instructions,
language policy, ADR template, current CLI types, package manifest, compiler
surfaces, and task-runner commands. A fresh-context adversarial reviewer
independently challenged sequencing, commands, interfaces, safety, and proof.
All substantive findings were accepted. A cross-model review was offered but
not run.
