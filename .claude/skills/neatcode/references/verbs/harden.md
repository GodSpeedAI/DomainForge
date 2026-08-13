# `neatcode harden`

Take code that works on the happy path and make it credible in production.

Structure is preserved. **Completeness is added.** This is the complement of
[`restructure.md`](restructure.md): that verb changes *how* the code is built while keeping
what it does; this one keeps how it is built and completes what it does under real conditions.

This edits code. State the file list first; deletions need confirmation.

---

## When this is the right verb

The target is plausible, structurally sound, and incomplete against reality. Typical origin: an
agent implemented a feature, the tests pass, and nobody has asked what happens when the network
is slow, the message is delivered twice, or the user closes the tab mid-request.

**Not the right verb when** the implementation strategy itself is wrong. Hardening an
implementation that should be replaced is wasted work. Say so and recommend `restructure`
first, then harden.

**Not the right verb when** the code is simply unfinished — stubs and unwired paths are
completion failures, and finishing them is the default build flow, not hardening.

---

## The hardening dimensions

Work through these against the target's archetype
([`../archetypes.md`](../archetypes.md)). A pure transformation needs edge cases and little
else; a workflow orchestration needs nearly all of them. **Do not apply every dimension to
every target** — unearned hardening is unearned complexity wearing a serious face.

### Edge cases
Empty, zero, negative, single-element, maximum, overflow, duplicate, absent, malformed,
unicode and combining characters, timezone and DST boundaries, clock skew, very large inputs.
For each: handle it, make it unrepresentable, or state why it cannot occur.

### Idempotency
Can this run twice? It probably will — at-least-once delivery, client retries, operator
re-runs. Give it an idempotency key, a natural unique constraint, or a conditional write. Make
the *effect* idempotent, not just an early return.

### Concurrency and ordering
What happens when two of these run at once? Find check-then-act, read-modify-write, and
unguarded shared state. Prefer database-level atomicity to application locks. Name the ordering
guarantee the code relies on and confirm it exists.

### Cancellation
Propagate the cancellation signal — token, `AbortSignal`, `context.Context` — and check it at
await points and loop iterations. Release resources in `finally`/`defer`/`Drop`. Ensure a
cancelled operation does not still write.

### Recovery and rollback
For each failure point: what state is left behind? Make the operation atomic, or add explicit
compensation, or document the reconciliation path. Order steps so the recoverable failure
happens first.

### Timeouts and resource bounds
Every external call gets a timeout. Every buffer, queue, batch, and connection pool gets a
bound. Every retry gets an attempt limit, jitter, and — where the dependency can be overwhelmed
— a circuit breaker. Retry at one layer, not three.

### Observability
Every failure path produces a signal. Every degraded path increments a counter. Logs carry the
subject's identifier and a correlation id. Each metric answers a stated operational question.
See [`../taxonomy/observability.md`](../taxonomy/observability.md).

### Security boundaries
Where does untrusted input enter, and where is it validated — before or after the unsafe use?
Is authorization checked on the resource, not just the session? Are defaults closed? Are
secrets absent from logs and error paths? See
[`../taxonomy/security.md`](../taxonomy/security.md).

### Migrations
Expand-then-contract. A rollback path that has been exercised. A mixed-version window where old
code reads new data and new code reads old data. Say which phase this change is and what the
next one is.

### Production wiring
Registered, routed, exported, injected, flagged, configured, deployed. Is the configuration
read? Is the flag reachable? Does the feature work through its real entry point, not just
through a direct call in a test?

---

## Method

### 1 · Establish the baseline
Run the existing tests. Record the result. Hardening must not change happy-path behaviour, and
without a baseline you cannot demonstrate that it did not.

### 2 · Enumerate the gaps
Walk the dimensions and produce a **list before touching anything**, ordered by consequence.
Each entry names the condition, the current behaviour, and the required behaviour.

```markdown
**Hardening gaps** · `src/billing/resume.ts` · archetype: boundary adapter

1. S1 · Capture is retried without an idempotency key → duplicate charge on timeout
2. S2 · No timeout on the Stripe call → a slow dependency exhausts the request pool
3. S2 · Failure path logs nothing → duplicate charges would be invisible
4. S3 · Cancellation is not propagated → work continues after the client disconnects
5. S4 · No bound on the retry attempts → retry storm during a partial outage
```

### 3 · Confirm the scope
Present the list. Ask which items to address if the list is long or if any item changes
observable behaviour. Hardening frequently *does* change behaviour — a previously-silent
failure now returns an error — and that must be agreed, not assumed.

### 4 · Implement
Smallest change per gap. Prefer mechanisms the repository already has: its retry helper, its
logger, its metric names, its error types. Introducing a new resilience library to add one
timeout is unearned.

### 5 · Prove it
Each hardened dimension gets a test that would fail without the hardening:

| Dimension | Test |
| --- | --- |
| Idempotency | Deliver twice; assert one effect |
| Concurrency | Run concurrently; assert the invariant holds |
| Cancellation | Cancel mid-flight; assert cleanup and no write |
| Timeout | Stub a slow dependency; assert it gives up |
| Recovery | Fail between steps; assert a consistent end state |
| Observability | Inject the failure; assert the signal appears |
| Security | Send the hostile input; assert rejection before any effect |

A hardening change with no new tests has not hardened anything. It has added code that looks
careful.

### 6 · Report

```markdown
**NeatCode · harden** · `src/billing/resume.ts` · archetype: boundary adapter
addressed: idempotency, timeout, failure observability · deferred: cancellation (item 4, by request)
behaviour change: capture failure now returns 502 rather than silently succeeding — agreed in scope
evidence: npm test ✓ (211 passed, 7 new) · duplicate-delivery test ✓ · timeout test ✓
critique: correctness 4 · fit 5 · semantics 4 · restraint 4 · operations 5 · evidence 5
open: retry-storm bound (item 5) not addressed — named as debt
```

---

## Failure modes of this verb

- **Hardening everything.** A pure transformation does not need a circuit breaker. Unearned
  hardening is unearned complexity, and it is harder to remove because it looks responsible.
- **Adding defence for impossible conditions.** A null check on a value the type system
  guarantees is noise. Harden against what can occur.
- **Silent behaviour change.** Making a previously-silent failure loud is usually correct and
  is always a behaviour change. Say so.
- **Reaching for a library.** Most of these dimensions are a few lines against what the
  repository already has.
- **Hardening what should be replaced.** Check the structure first.
