# `neatcode restructure`

Preserve the legitimate behavioural intent; replace a weak implementation strategy.

> **Keep the behaviour and the constraints. Replace the implementation fingerprint.**

This edits code. Everything in the safety rails applies: state the file list first, deletions
require confirmation, never expand scope.

**It is not a rewrite.** An agent that hears "restructure" and discards too much is the
failure mode this verb is designed against. The most common correct output is a *smaller* diff
than expected.

---

## What may change, and what may not

**Preserve:**
- Externally observable behaviour
- Public contracts, unless the user explicitly authorized a break
- Domain meaning and vocabulary
- Required integrations
- Data compatibility — persisted formats, wire formats, migrations already applied
- Operational constraints — resource limits, latency budgets, deployment shape

**May be replaced:**
- Module structure and file layout
- Control flow
- State ownership
- Abstraction boundaries
- Dependency direction
- Internal API shape
- Error model, where it is internal
- Test organization

---

## 1 · Characterize first

**Without a behavioural baseline, "preserves behaviour" is a wish.** This step is the whole
verb; skipping it converts a restructure into an unreviewable rewrite.

1. Read the target and write down what it does, including the parts that look accidental.
   Accidental-looking behaviour is where the load-bearing quirks live.
2. Identify contracts and invariants: what callers rely on, what must remain true.
3. **Inspect callers and integration points.** Every one you can find, plus a statement of what
   your search could not see — dynamic dispatch, reflection, other repositories, deployed
   clients, persisted data.
4. Establish the baseline:
   - If tests already cover the behaviour, run them and record the result. That is the baseline.
   - If they do not, **write characterization tests first**, in a separate commit, asserting
     current behaviour including its oddities. Label them as characterization so the next
     reader knows they pin behaviour rather than specify it.
   - If characterization is impossible — a side-effect-heavy path with no seam — say so, and
     restrict the restructure to what can be verified. Do not proceed blind.

State the baseline in the plan. "27 existing tests pass; added 6 characterization tests for the
undocumented empty-input behaviour" is a baseline. "The code looks well understood" is not.

## 2 · Name the structural failure

Say precisely what is wrong. One sentence, from the taxonomy where it fits:

> *"Subscription state has three owners; two write paths bypass the transition function, so
> audit records attach to only one of three."*

If you cannot name the failure crisply, do not restructure. A restructure without a named
defect is churn, and churn in a working system is a net loss.

## 3 · Choose the target shape

- The narrowest change that removes the named failure. Not the ideal architecture.
- Restore canonical authority: one invariant, one owner.
- Remove unearned structure ([`../restraint.md`](../restraint.md)) — but only structure
  implicated in the failure. Deleting an unrelated wrapper because you noticed it is scope
  expansion.
- Fix dependency direction if it is the failure. Do not re-layer a whole module because one
  import points the wrong way.
- Match the repository's idiom. A restructure that introduces a foreign style trades one
  problem for another.

## 4 · Plan

```markdown
**NeatCode · restructure** · `src/billing` subscription state

- **Failure** · three owners for the subscription lifecycle; two paths bypass `transition()`
- **Preserve** · all externally observable transitions, the `SubscriptionStatus` wire values,
  the admin API's response shape
- **Replace** · direct `status` assignment in `admin/subscriptions.ts` and `jobs/expiry.ts`;
  make the field module-private
- **Baseline** · 27 existing tests + 6 new characterization tests for the admin path
- **Files** · modify `src/billing/state.ts`, `src/api/admin/subscriptions.ts`,
  `src/jobs/expiry.ts`; add `src/billing/state.characterization.test.ts`. No deletions.
- **Out of scope** · the `reports/` subtree also reads `status`; read-only, unaffected
```

Wait for confirmation on anything that touches a public surface or deletes a file.

## 5 · Restructure

- Behaviour-preserving moves first, in their own commit where the workflow allows. Then, if the
  task calls for it, behaviour change — clearly separated.
- Keep the diff reviewable. If it grows past what a person will read, split it.
- Do not "improve" adjacent code. Note it as a finding.
- Do not rename beyond what the restructure requires ([`../taxonomy/change-discipline.md`](../taxonomy/change-discipline.md)).

## 6 · Compare behaviour

Run the baseline. Every characterization test must still pass, or every difference must be
**intentional, listed, and justified**.

```markdown
**Behaviour comparison**
- 33 of 33 baseline tests pass unchanged.
- One intentional deviation: the admin path now writes an audit record where it previously did
  not. This was the defect; `docs/adr/0004.md:12` requires it. Called out for approval.
```

An unexplained difference is a bug, not a bonus. Report it as one.

## 7 · Report

Completion block plus the behaviour comparison plus the six-axis critique. Name what you did
**not** restructure and why — that list is how the user knows the boundary held.

---

## When to refuse

Say so plainly rather than doing something adjacent:

- **No baseline is obtainable and the risk is high.** Recommend characterization work first.
- **The target is fine and the user dislikes its style.** Say that the structure is earned,
  name what earns it, and stop. Restructuring to taste is churn.
- **The failure is completeness, not structure.** That is [`harden.md`](harden.md). Hardening
  code you are about to replace is wasted work — and replacing code that only needed hardening
  destroys working behaviour for nothing.
- **A rewrite is genuinely warranted.** Say so explicitly, with the reasoning, and get
  authorization. Do not deliver one under this verb's name.
