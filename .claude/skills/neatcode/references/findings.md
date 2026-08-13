# The finding model

Loaded for `review` and `audit`, and whenever the default flow produces something reportable.

A finding is a claim about a defect, grounded in evidence you can point at. Generic
criticism is not a finding. "Consider refactoring this" is not a finding.

## Anatomy

```text
### <Finding name>            ← from the taxonomy, or a precise phrase
S<n> · <confidence> · <provenance>

Location   <path:line>, <path:line>
Evidence   What is observably true. Quote the code or name the exact structure.
Cause      The reasoning failure underneath — not the symptom.
Consequence What goes wrong, concretely, and when.
Trajectory  What this costs in six months if it stays. Omit for S1.
Correction  The specific change. Name the function, the module, the call site.
Verification What proves the correction worked.
Exception   Where you might be wrong, or the condition under which this is fine. Omit when there is none.
```

Not every field earns its place every time. At **Trace** and **Standard** depth, collapse to
three lines when the finding is simple:

```text
### Duplicated normalization  S3 · confirmed · introduced
`src/api/handler.ts:88` re-implements the trimming and case-folding already in
`src/domain/email.ts:12`. Two owners for one invariant; they will drift on the next change.
→ Call `normalizeEmail()` from the handler. Verified by the existing `email.test.ts` cases
  plus one asserting the handler path produces identical output.
```

Reserve the full block for S1–S2 findings and anything the user is likely to push back on.

## Provenance — relationship to the change

Mandatory in `review`. Omitted in `audit`, where there is no change to be relative to.

| Label | Meaning |
| --- | --- |
| **introduced** | The change created this. It did not exist before. |
| **worsened** | It existed; the change made it more expensive, more widespread, or harder to undo. Adding a third caller to a duplicated path is worsening, not introducing. |
| **exposed** | The change did not touch it, but made it reachable, load-bearing, or visible for the first time. |
| **pre-existing (blocking)** | Untouched by the change, but the change cannot be completed safely while it stands. Say *why* it blocks. |
| **pre-existing (out of scope)** | Real debt, not this patch's business. Report at most a short list of these, at the end, clearly separated. |
| **resolved** | The change fixed something. Say so — a review that only ever subtracts is not trusted, and resolved findings are how a reviewer proves they read the whole diff. |

**The fairness rule.** Do not bill a patch for the file it landed in. A 12-line fix in a
600-line god module is a 12-line fix; the god module is a separate, older finding. The
inverse rule matters just as much: a bad file is not a licence to add to it, and "the code
was already like this" does not convert **introduced** into **pre-existing**.

## Severity

Consequence-oriented. Not aesthetic dislike, not how strongly you feel about it.

| Level | Meaning | Examples |
| --- | --- | --- |
| **S1** | Correctness, security, or data-integrity risk. Wrong results, lost or corrupted data, an authorization hole, an exploitable input path, a migration that cannot roll back. | Off-by-one in a money calculation · missing authorization check · non-atomic read-modify-write on shared state · unbounded deserialization of user input |
| **S2** | Architectural or operational risk. Correct today, fails under production conditions or violates a boundary that will force expensive rework. | Retry of a non-idempotent effect · dependency direction inverted against a stated rule · a failure path with no observability · a public interface changed without a compatibility story |
| **S3** | Compounding debt. Each future change pays interest, and the interest rate rises. | Duplicated authority · parallel source of truth · a second normalization path · a generic module accumulating unrelated behaviour |
| **S4** | Maintainability degradation. Readers pay; the system does not. | Unearned abstraction · pass-through wrapper · fragmentation of one cohesive operation · a test that asserts implementation details |
| **S5** | Clarity and hygiene. Small, real, and cheap to fix. | A misleading name · a comment restating the code · an unused import that will confuse a search |

**Calibrate against context, not against a table.** The same construct is S1 in a payments
path and S4 in a build script. A missing `await` in a logging call is S5; the same missing
`await` in a transaction commit is S1. State the context that set the level when it is not
obvious.

**Do not inflate.** A skill that files everything as critical gets ignored on its second run.
**Do not deflate either** — the pressure to be agreeable is exactly how an S1 becomes "a
minor note."

## Confidence

| Level | Meaning |
| --- | --- |
| **confirmed** | You read the code that makes it true, and traced it. You can quote the lines. |
| **probable** | The evidence strongly implies it, but one link is unverified — a caller you did not open, a framework behaviour you did not confirm. Say which link. |
| **possible** | A pattern-level suspicion worth raising, explicitly unconfirmed. Never report an S1 as *possible* without saying exactly what would confirm it. |

Confidence and severity are independent. A *possible* S1 is worth reporting; it just has to
carry the sentence "here is what would confirm this."

## What is not a finding

Check this list before writing anything down. Every item here, reported as a defect, costs
the skill credibility it does not get back.

- **Formatting and style the repository's own tooling accepts.** Line length, quote style,
  brace placement, import order. If a linter runs in CI, the linter owns it.
- **Naming you would have done differently**, where the existing name is accurate and
  consistent with its neighbours.
- **A pattern you dislike that the repository uses consistently.** Consistency with a
  mediocre convention beats an island of your preference. If the convention is genuinely
  harmful, that is one S3 finding about the convention — not one finding per occurrence.
- **Missing tests for code the change did not touch.**
- **Hypothetical future requirements.** "This won't scale" is a finding only with a named
  load and a named limit.
- **Restating the diff.** "This adds a new function" is not an observation.
- **Speculative security findings with no reachable input path.** Name the path or drop it.

Two of these are worth converting rather than dropping: a widely-used harmful convention
becomes one S3 finding about the convention, and a genuine scaling limit becomes an S2 with
the load and the limit stated.

## Report shapes

### `review`

```markdown
**NeatCode · review** · <scope> · <n> files (+a / −b) · depth: <trace|standard|deep>

**Verdict** · <one sentence: what this change does, and whether it is safe to merge>

**Contract read** · <what the change appears intended to do; flag any mismatch with the stated task>

#### Blocking (S1–S2)
<finding blocks>

#### Debt introduced (S3–S4)
<finding blocks>

#### Resolved by this change
<one line each>

#### Pre-existing, out of scope
<one line each, at most five, clearly not this patch's fault>

**Evidence** · <commands run, results; what remains unverified>
**Critique** · correctness n · fit n · semantics n · restraint n · operations n · evidence n
```

A clean change gets a short report. Say so plainly and stop:

```markdown
**NeatCode · review** · staged · 2 files (+18 / −4) · depth: standard
**Verdict** · Correct, in the canonical path, covered by a test that fails without it. No findings.
**Evidence** · `npm test` ✓ (204 passed) · `npm run typecheck` ✓
```

### `audit`

Same block shape, without provenance labels, grouped by consequence rather than by file, and
opening with the structural picture — the morphology and conformance verdict — before the
line-level findings. A reader who stops after the first paragraph should still know the most
important thing.

Close every report with a count: `2 S1 · 1 S2 · 4 S3 · 3 S4 · 1 S5`.

## Ordering

1. Findings that block safe completion, most severe first.
2. Debt introduced by the change.
3. Structural and architectural observations.
4. Resolved.
5. Pre-existing, out of scope.

Within a tier, order by consequence, not by file order. The reader's attention is the scarce
resource; spend it on the thing that will hurt.
