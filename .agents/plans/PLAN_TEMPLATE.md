<!-- ============================================================= -->

<!-- PLAN TEMPLATE — fill every {{VARIABLE}} and [PLACEHOLDER]. -->

<!-- Conventions: -->

<!-- {{VAR}} = a value reused across the doc (define once in the table below) -->

<!-- [PLACEHOLDER: x] = fill-in specific to this plan; delete the brackets when done -->

<!-- <!-- … --> = authoring guidance; DELETE these comments in the final plan -->

<!-- ⟨optional⟩ = include only if relevant; otherwise remove the whole line/block -->

<!-- Golden rule this template enforces: every task PROVES or CHANGES something with a -->

<!-- copy-pasteable verification gate that exits 0. No task is "done" on assertion alone. -->

<!-- ============================================================= -->

# Implementation Plan — {{PLAN_TITLE}}

**Created:** {{DATE}}

**Source of truth:** {{SOURCE_DOC_PATH}} <!-- the audit/spec/review this plan executes; quote it, don't paraphrase from memory -->

**Originating context:** [PLACEHOLDER: 1–2 lines — what produced this plan (a review? an audit? a bug?) and on what branch/state]

**Status of the work today:** [PLACEHOLDER: what already exists vs. what this plan closes — be honest about partials]

---

## 0. How to use this plan (agent operating instructions)

<!-- This section is what lets a COLD agent execute without re-deriving context. Keep it. -->

- Execute tasks in the order given. Dependencies: [PLACEHOLDER: e.g. "Task 1→2 dependent (2 reuses X); Tasks 3,4 independent"].
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **{{CORE_PRINCIPLE}}** <!-- the single rule that keeps the work coherent. e.g. "One engine, one producer — never re-implement output in a wrapper/test; extract a shared function." Make it specific to THIS codebase. -->
- Match surrounding code style; locate the relevant idioms here: [PLACEHOLDER: dirs/files that show the pattern to mirror].
- [PLACEHOLDER: any source-of-truth-is-X rule, e.g. "the corpus/fixtures are the spec — change expected files deliberately, in the same commit as the code that justifies them."]

### Global verification gates (must stay green after EVERY task)

<!-- List the cheapest commands that prove the whole system still works. These get re-run constantly. -->

```bash
{{GATE_BUILD_OR_UNIT}}        # [PLACEHOLDER: e.g. cargo test --features cli --workspace]
{{GATE_LANG_A}}               # [PLACEHOLDER: e.g. just ci-test-python]
{{GATE_LANG_B}}               # [PLACEHOLDER: e.g. just ci-test-ts]
{{GATE_LINT}}                 # [PLACEHOLDER: the exact CI lint/format gate]

```

⟨Rebuild steps if you touched {{ARTIFACT}} (bindings/generated code) before running the gates:⟩

```bash
{{REBUILD_CMD_1}}             # [PLACEHOLDER]
{{REBUILD_CMD_2}}             # [PLACEHOLDER]

```

---

## Key facts already discovered (do not re-derive)

<!-- The highest-leverage section. Every file:line you had to hunt for goes here so the next agent doesn't. -->

<!-- Prefer file:line over prose. If a fact isn't pinned to a location, it's probably not actionable. -->

| Thing                                                                                                                   | Location          |
| ----------------------------------------------------------------------------------------------------------------------- | ----------------- |
| [PLACEHOLDER: the producer/owner of the behavior in scope]                                                              | {{PATH}}:{{LINE}} |
| [PLACEHOLDER: the test/oracle that pins current behavior]                                                               | {{PATH}}          |
| [PLACEHOLDER: a non-obvious invariant — e.g. "IDs are content-derived EXCEPT X which is random and must be normalized"] | {{PATH}}          |
| [PLACEHOLDER: relevant API signatures the tasks will call]                                                              | {{PATH}}          |
| [PLACEHOLDER: where the pre-existing/unrelated issue actually lives]                                                    | {{PATH}}          |

---

<!-- ============================================================= -->

<!-- TASK BLOCK — DUPLICATE THIS WHOLE BLOCK ONCE PER TASK. -->

<!-- Order tasks by dependency, then by value. Keep each one -->

<!-- independently verifiable: an agent should be able to stop -->

<!-- after any task with the tree in a green, coherent state. -->

<!-- ============================================================= -->

## Task {{N}} — {{TASK_TITLE}} ⟨(phase/area · priority)⟩

**Goal:** [PLACEHOLDER: one sentence — the observable outcome, not the activity. "X is true and a test proves it."]

**Why this shape:** [PLACEHOLDER: the design constraint that dictates the approach, so the agent doesn't pick a plausible-but-wrong alternative. Omit only if truly self-evident.]

### Steps

1. [PLACEHOLDER: concrete step] — file: `{{PATH}}`. <!-- name the exact file; suggest a signature/name if adding code -->
2. [PLACEHOLDER: concrete step] — file: `{{PATH}}`.
3. [PLACEHOLDER: keep steps small enough that each maps to an edit; reference the "discovered facts" table instead of re-explaining].
4. ⟨Docs/coherence step: update {{DOC}} — but only flip a status claim AFTER its test passes (no claim ahead of proof).⟩

### Gate

```bash
[PLACEHOLDER: exact command(s) that prove THIS task — narrowest scope that's still meaningful]

```

**Done when:** [PLACEHOLDER: observable pass condition + a negative check, e.g. "deliberately breaking X makes the gate fail" — proves the test has teeth].

**Redesign trigger:** [PLACEHOLDER: the discovery that invalidates this approach and what to do instead. Write "none plausible" if genuinely none.]

<!-- END TASK BLOCK — duplicate above as needed -->

---

## Final acceptance checklist (whole plan)

<!-- One checkbox per task's "Done when", plus the cross-cutting ones. The reviewer reads only this. -->

- [ ] [PLACEHOLDER: Task 1 outcome, with its teeth-check] _(Task 1)_
- [ ] [PLACEHOLDER: Task 2 outcome] _(Task 2)_
- [ ] [PLACEHOLDER: Task N outcome] _(Task N)_
- [ ] {{GATE_LINT}} exits 0.
- [ ] [PLACEHOLDER: docs/{{DOC}} reflects reality — no status ahead of a passing test.]
- [ ] All global gates green; {{ARTIFACT}} rebuilt.

## Guardrails (do not violate)

<!-- Pulled from the source-of-truth doc. These stop an eager agent from scope-creeping the system wider. -->

- [PLACEHOLDER: e.g. "Do NOT add new {{SURFACE}} to accomplish a task — every item here proves existing behavior, not extends it."]
- [PLACEHOLDER: project-specific invariant that must survive]
- [PLACEHOLDER: commit hygiene — e.g. "keep the unrelated cleanup (Task N) in its own commit titled …"]
<!-- ============================================================= -->

<!-- AUTHORING CHECKLIST (delete before saving the real plan): -->

<!-- [ ] Every task has a Gate that exits 0 and a teeth-check. -->

<!-- [ ] Every file reference is file:line, not vibes. -->

<!-- [ ] Dependencies between tasks stated; independent ones flagged. -->

<!-- [ ] CORE_PRINCIPLE + Guardrails come from the source doc, quoted. -->

<!-- [ ] Pre-existing/unrelated work is isolated, not smuggled in. -->

<!-- [ ] A cold agent could start at Task 1 with zero prior context. -->

<!-- ============================================================= -->
