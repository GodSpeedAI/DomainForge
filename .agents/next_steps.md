# Next Steps

## 1. Write the Milestone 0 language code plan

Execute Task 3 from
`.agents/plans/2026-07-18-conversational-application-generator.md` and create
`.agents/plans/2026-07-18-conversational-application-generator-m0-language.md`.
Expected outcome: a codebase-grounded, test-first plan that a lesser coding
agent can execute without choosing grammar, identity, validation, hashing, or
binding semantics.

## 2. Adversarially review the Milestone 0 plan

Cross-check every task and test against accepted ADR-013, the normative
reference, actual repository paths/APIs, and the master plan's Milestone 0 stop
gate. Expected outcome: no unresolved design choice or stale path remains in
the coding plan.

## 3. Begin implementation only from the reviewed plan

Use the repository's worktree, incremental implementation, and test-driven
development gates. Expected outcome: implementation starts with the first
failing Milestone 0 test and preserves cross-binding compatibility.
