# .agents/ — Agent Workbench Guide

Durable operational memory, task state, specifications, and execution evidence for DomainForge agents. Follow root `AGENTS.md` for repository-wide invariants and verification gates.

## 1. Directory Roles & Authority

* **`current_state.md`**: Canonical checklist-backed current execution state. Every checked item requires concrete implementation evidence (changed paths, tests run, commands executed, pass/fail results); documentation references alone are insufficient.
* **`next_steps.md`**: Immediate 2–3 concrete executable steps, each with an expected outcome. Keep concise so a cold agent can resume without rediscovery.
* **`plans/`**: Active and approved execution plans for multi-step or non-trivial tasks. Plans are execution scratch; do not treat plans as specifications.
* **`reports/`**: Read-only audits, investigations, benchmarks, and run summaries.
* **`specs/`**: Feature/architecture contracts and design notes.
* **`lessons/`**: Verified, reusable project-specific failures and improvements. Record only lessons backed by executable evidence, commands, or tests.
* **`OBSERVED_DEBT.md`**: Concrete out-of-scope debt, gaps, risks, or defects noticed while working. Each entry requires evidence, concrete impact, and a suggested next move.

## 2. Workbench Invariants

* **Evidence over intent**: Never mark work complete or check off status items without running verification commands and observing results.
* **One home per fact**: Reference canonical sources; do not copy specifications into scratch notes or duplicate ledgers.
* **Scope bounding**: When discovering adjacent defects or out-of-scope debt, record them in `OBSERVED_DEBT.md` rather than widening the current task.
* **Generalizable lessons**: Record entries in `lessons/` only when the learning is reusable across future sessions in this repository.
* **No conversational transcripts**: Keep memory files concise, structured, and free of chat transcripts or machine-specific absolute paths.

## 3. Handoff Contract

Before completing a task or handing off to another agent:

1. Inspect the final diff (`git status`, `git diff`).
2. Run verification commands proportional to the claim across affected surfaces.
3. Confirm no test, invariant, or canonical source was weakened or bypassed.
4. Update `current_state.md` with actual evidence and test results.
5. Update `next_steps.md` so the next agent can resume cold without rediscovery.
6. Record reusable failures in `lessons/`.
7. Run `graft build` after code changes that affect repository indexing.
8. Leave unresolved debt or uncertainty explicit in `OBSERVED_DEBT.md`.
