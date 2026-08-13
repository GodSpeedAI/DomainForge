# `neatcode audit`

Judge an **existing** file, module, subsystem, or repository. No diff. **No edits.**

The question is *what is the state of this code* — not *what did this patch do*, which is
[`review.md`](review.md). Both can run over the same files and produce different, both-correct
answers.

---

## 1 · Scope

Take the target the user named. If they named the repository, do not audit all of it at line
level — that produces a document nobody reads. Audit **structurally** first, then go deep on
the two or three areas the structure identifies as most consequential, and say which you chose
and why.

```bash
neatcode envelope --paths src/billing --verb audit
neatcode envelope --repo --verb audit
```

State the scope and the depth in the header. An audit that quietly examined 10% of what its
title implies is worse than one with an honest boundary.

## 2 · Read the morphology first

Before opening a source file, read the shape. This is the code analogue of looking at a page
before reading it, and it is where a repository audit gets most of its value.

- Top-level organization: by technical layer, by feature, by domain, by deployment unit?
- Where has code landed recently (`git log --oneline -30`)? That is the current gravity, which
  is often not the documented structure.
- Test placement and volume relative to source.
- File-size distribution: god modules at one end, one-function sprawl at the other.
- Generic modules — `utils`, `common`, `shared`, `core`, `lib` — and their size and direction of
  dependency.
- Naming grammar: is one concept named one way?
- Public surface: designed, or accidental?

## 3 · Architecture conformance

Run [`../architecture/phenotype.md`](../architecture/phenotype.md). Collect the claim, read the
morphology, read the dependency direction, trace two representative behaviours, and return a
verdict: **conformant · partially conformant · nominal · contradictory · unverifiable ·
coherent emergent alternative.**

Give this section a place near the top of the report. It is usually the most consequential
thing an audit produces, and it reframes every finding below it.

Remember the sixth verdict. Code that has diverged from its README into something coherent is
not decayed; the documentation is wrong, and that is the cheaper fix.

## 4 · Implementation quality

Load the taxonomy families the target implicates ([`../taxonomy.md`](../taxonomy.md)). For a
repository audit, the usual set is `abstraction`, `authority`, `boundary`,
`maintainability-theater`, and `tests`. Add `security` for anything handling input or identity,
and `state-and-concurrency` for anything async or persistent.

Trace at least one behaviour end to end. An audit conducted entirely by pattern-matching finds
smells and misses defects.

## 5 · Operational readiness

For anything that runs continuously or handles production traffic:

- What happens when each dependency is slow, wrong, or absent?
- Are failure paths observable? ([`../taxonomy/observability.md`](../taxonomy/observability.md))
- Are resources bounded — connections, memory, queue depth, concurrency?
- Is shutdown graceful? Is in-flight work drained?
- Are degraded modes represented, or silent?

## 6 · Evidence and debt

- What does this repository accept as proof? Does the proof cover the risky parts?
- Which areas have no tests at all, and which of those matter?
- What debt is real, named, and worth a maintainer knowing before they touch the area?
- What was tried and abandoned — evident from dead code, stale flags, or half-migrations?
  Half-finished migrations are among the highest-value findings an audit produces, because
  nobody currently owns them.

## 7 · Report

Structural picture first, line-level findings second. A reader who stops after the first
paragraph should still have the most important thing.

```markdown
**NeatCode · audit** · `src/billing` (34 files, 6.2k lines) · depth: deep

**Summary** · The billing module works and is structurally sound at the edges, but subscription
state has three owners. Any change to the lifecycle currently requires three coordinated edits,
and two of the three paths skip the audit log.

### Architecture conformance
**Claim** · "Billing owns all subscription state transitions" — `docs/adr/0004.md:12`
**Observed** · `SubscriptionState.transition()` (src/billing/state.ts:40) is the intended owner.
`src/api/admin/subscriptions.ts:88` and `src/jobs/expiry.ts:31` both assign `status` directly.
**Verdict** · **Nominal.** The authority exists and is bypassed by two of three write paths.
**Consequence** · Audit records and expiry timers attach to the transition function; the two
bypassing paths produce subscriptions with no audit trail.
**Correction** · Route both through `transition()`; make `status` non-assignable from outside
the module. Add a test asserting no external assignment compiles.
**Confidence** · confirmed

### Findings

#### S1 — correctness, security, data integrity
<blocks>

#### S2 — architectural and operational risk
<blocks>

#### S3 — compounding debt
<blocks>

#### S4–S5 — maintainability and hygiene
<one line each>

### What is working
- Boundary adapters confine Stripe types to `src/billing/gateway/` — no external type appears
  in the domain modules (verified by import inspection).
- The period-rounding rule has exactly one owner and is covered by table-driven tests.

### Evidence
Examined: all 34 files at structure level; `state.ts`, `resume.ts`, `gateway/stripe.ts` traced
end to end. `npm test -- billing` ✓ (61 passed). Not examined: the `reports/` subtree.

`1 S1 · 2 S2 · 4 S3 · 6 S4 · 2 S5`
```

**Include "what is working."** Not for balance — an audit that cannot identify the load-bearing
correct parts has not understood the system, and the reader has no way to tell which of your
criticisms to weight.

## Failure modes of this verb

- **Auditing everything shallowly.** A ranked, honest partial audit beats a complete superficial
  one. Say what you did not examine.
- **Reporting the same defect once per occurrence.** A convention used badly in forty places is
  one finding about the convention, with a count.
- **Imposing an architecture.** Judge against what the repository claims or consistently does,
  not against your preferred school.
- **Style inventory.** If a linter runs in CI, the linter owns it.
- **Recommending a rewrite.** Almost never the correct output. Name the two changes that would
  most reduce future cost.
