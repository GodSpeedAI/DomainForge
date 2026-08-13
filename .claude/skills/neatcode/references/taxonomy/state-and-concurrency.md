# State and concurrency failures

Failures of time and sharing. The code is correct when one thing happens at a time and
everything succeeds. Load this family whenever the change touches shared state, async work,
retries, caches, queues, or transactions.

Findings here are frequently S1. They are also the hardest to confirm from a diff — say what
you traced and what you did not.

---

### Race condition

**Is** — Correctness depending on an interleaving that is not guaranteed.

**Signals** — Check-then-act on shared state (`if (!exists) create()`). Read-modify-write
without a lock, a transaction, or an atomic primitive. A `SELECT` followed by an `UPDATE`
computed from it. Two async operations mutating one structure. Lazy initialization without
synchronization. A counter incremented from concurrent handlers.

**Cause** — Reasoning about a single execution.

**Why agents** — Sequential reasoning is the default; concurrency is invisible in the source
of any one function.

**Risk** — Lost updates, duplicate records, double charges, corrupted state — intermittent and
load-dependent, which means production-only.

**Trajectory** — Reproduced rarely, "fixed" with a retry or a sleep, and never actually fixed.

**Exception** — A genuinely single-threaded context with no concurrent entry — verify rather
than assume; a single-threaded runtime still interleaves at `await`.

**False positive** — Protection exists at a level you did not look at: a database constraint,
a serializable transaction, a queue with per-key ordering, an actor mailbox, an outer lock.
Find it before reporting.

**Fix** — Make it atomic: a conditional update, a unique constraint, a compare-and-swap, a
transaction with the right isolation, or a lock with a stated scope. Prefer database-level
atomicity to application-level locking.

**Proof** — A concurrent test, or the database constraint that makes the bad state
unrepresentable.

---

### Non-atomic multi-step operation

**Is** — A logical operation composed of steps that can partially complete.

**Signals** — Two writes with no transaction. A database write followed by an external API
call, or the reverse. A file write plus an index update. Several updates in a loop. A write
followed by an event publish, with no outbox.

**Cause** — Thinking in steps rather than in outcomes.

**Why agents** — The steps are the implementation; atomicity is a property that lives outside
any of them.

**Risk** — Partial state after a crash, a timeout, or a deploy. The classic case: money moves
and the record does not.

**Trajectory** — Inconsistency accumulates silently; reconciliation jobs appear later to clean
up after it.

**Exception** — Deliberate eventual consistency with a stated reconciliation mechanism, or a
saga with compensation. Both are fine when they exist and are named.

**False positive** — An ambient transaction supplied by the framework or a decorator. Check.

**Fix** — One transaction where possible. Where it is not — external effects cannot join a
database transaction — use the outbox pattern, idempotency keys, or explicit compensation.
Order the steps so the recoverable failure happens first.

**Proof** — A test that injects failure between steps and asserts a consistent end state.

---

### Missing idempotency

**Is** — An operation that produces a different result when it runs twice, in a context where
it can.

**Signals** — A webhook handler with no deduplication. A queue consumer that inserts without a
unique key. A retried POST that creates. An email sent on every delivery attempt. A payment
capture without an idempotency key. A migration that is not safe to re-run.

**Cause** — Assuming at-most-once delivery. Almost every real transport is at-least-once.

**Why agents** — The retry lives in infrastructure the code never sees.

**Risk** — Duplicate charges, duplicate records, duplicate notifications.

**Trajectory** — Discovered by customers, and each occurrence needs manual remediation.

**Exception** — A naturally idempotent operation — setting a value rather than incrementing.

**False positive** — Idempotency provided by a unique constraint or an upstream dedupe layer.

**Fix** — Idempotency key with a stored result, a natural unique constraint, or a
conditional write. Make the *effect* idempotent, not just the handler's early return.

**Proof** — A test that delivers the same message twice and asserts one effect.

---

### Unsafe retry

**Is** — Retrying an operation whose repetition is not safe.

**Signals** — A retry wrapper around a function with side effects. Exponential backoff around
a payment call with no idempotency key. Retrying a partially-applied batch. Retrying on
timeout, where the timeout may mean *succeeded but the response was lost*.

**Cause** — Treating retry as a generic reliability decoration.

**Why agents** — Retry-with-backoff is an extremely common code shape, and it looks like
robustness.

**Risk** — Amplified duplicate effects, and retry storms that convert a slow dependency into
an outage.

**Trajectory** — Retries are added at several layers; the effective multiplier becomes the
product of all of them.

**Exception** — Retrying genuinely idempotent reads, or writes with an idempotency key.

**False positive** — The underlying call is idempotent and you did not check.

**Fix** — Retry only idempotent operations. Add an idempotency key. Bound attempts, add
jitter, and add a circuit breaker where the dependency can be overwhelmed. Retry at one layer,
not three.

**Proof** — A test asserting one effect after a forced retry.

---

### Stale cache assumption

**Is** — Caching without an invalidation rule, or with one that cannot be correct.

**Signals** — A cache populated on read with no invalidation on write. A TTL chosen with no
reasoning about staleness tolerance. A cache keyed on something that does not capture every
input — notably the tenant, the locale, or the permission scope. A memoized function whose
inputs include mutable state.

**Cause** — Treating caching as a performance switch rather than as a consistency decision.

**Why agents** — Adding a cache is a well-known shape; deciding what makes an entry wrong is
domain reasoning.

**Risk** — Users see stale data. In a permission cache, users see *other people's* data —
which makes this S1, not a performance note.

**Trajectory** — Nobody can safely change any write path, because nobody knows what caches it
must invalidate.

**Exception** — Explicitly accepted staleness with a stated bound.

**False positive** — Invalidation exists in a write path you did not read.

**Fix** — Name the invalidation rule and place it with the write. Include every variable that
affects the value in the key — especially the identity of the requester.

**Proof** — A test: write, then read, and assert freshness.

---

### Cancellation leak

**Is** — Work that continues after its requester is gone.

**Signals** — An async operation with no cancellation token, `AbortSignal`, or `context.Context`
plumbed through. A `setTimeout` never cleared. A subscription never unsubscribed. A goroutine
with no exit path. A stream never closed. A cancellation token accepted and never checked.

**Cause** — Modelling the success path only.

**Why agents** — Cancellation plumbing is verbose, easy to omit, and never exercised by a
happy-path test.

**Risk** — Resource exhaustion, work done for nobody, writes landing after a "cancelled"
operation returned.

**Trajectory** — Slow leaks that present as periodic restarts and get attributed to
"memory pressure."

**Exception** — Short, bounded work where cancellation costs more than it saves.

**False positive** — The runtime cancels automatically — a scoped task, a structured
concurrency construct.

**Fix** — Propagate the cancellation signal and check it at each await point and loop
iteration. Clean up in a `finally`, a `defer`, or a `Drop`.

**Proof** — A test that cancels mid-flight and asserts the resource was released.

---

### Inconsistent state transition

**Is** — A state change reachable that the model does not allow, or that leaves related state
inconsistent.

**Signals** — Direct status assignment bypassing the transition function. A transition with
side effects that only some paths trigger. A field updated without its dependent fields — a
status changed without its timestamp, a total without its line items.

**Cause** — Treating state as data rather than as a machine.

**Why agents** — Assignment is simpler than a transition function, and the constraint is not
represented in the type.

**Risk** — Entities in states the rest of the system does not handle, discovered far away from
the cause.

**Trajectory** — Defensive checks for impossible states spread through the codebase.

**Exception** — Documented repair paths.

**False positive** — The transition is legal and you misread the machine.

**Fix** — One transition function; make the state field non-assignable from outside; update
dependent fields inside the transition.

**Proof** — A test enumerating legal transitions and asserting all others are rejected.

---

### Unclear transaction boundary

**Is** — It is not determinable from the code where a transaction starts and ends.

**Signals** — Nested transaction helpers. A transaction opened in a repository method called
from another transactional method. External calls inside a transaction. A long transaction
spanning user interaction. Implicit auto-commit mixed with explicit transactions.

**Cause** — Transactions added per-method rather than per-use-case.

**Why agents** — Each method looks correct in isolation.

**Risk** — Lock contention, unexpected partial commits, deadlocks, and connection pool
exhaustion.

**Trajectory** — Fear of touching data access at all.

**Exception** — A framework with well-defined nesting and savepoint semantics that the team
uses deliberately.

**False positive** — Nesting is handled correctly by the framework's propagation rules.

**Fix** — Open transactions at the use-case boundary; make inner functions transaction-
agnostic. Never make an external call inside one.

**Proof** — A test asserting rollback of the whole use case on a late failure.
