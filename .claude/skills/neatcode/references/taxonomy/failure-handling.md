# Failure-handling failures

Failures of failure. Error handling exists, looks conscientious, and makes things worse. The
common thread: the code treats an error as something to be *quieted* rather than as
information to be routed to whoever can act on it.

---

### Swallowed exception

**Is** — An error caught and discarded.

**Signals** — An empty `catch {}`. `except: pass`. `catch (e) { console.log(e) }` in a path
where the caller needed to know. `_ = err` in Go. `.catch(() => {})` on a promise. A `Result`
discarded with `let _ =`.

**Cause** — Making an error stop, rather than deciding what should happen because of it.

**Why agents** — A `try/catch` around risky code is one of the strongest shapes in the
training distribution, and an empty handler is the shortest way to complete it.

**Risk** — The operation reports success having done nothing. This is worse than crashing:
a crash is observable.

**Trajectory** — Debugging becomes archaeology. The eventual symptom appears arbitrarily far
from the cause.

**Exception** — Genuinely optional work — a best-effort metric, an analytics ping, a cache
warm — where the comment says so and the failure is still counted somewhere.

**False positive** — The error is handled by a control-flow mechanism you did not read: a
framework error boundary, a middleware, an outer handler, a supervisor.

**Fix** — Handle it, propagate it, or explicitly document why it is ignorable. If ignoring, at
minimum count it.

**Proof** — A test that injects the failure and asserts the observable consequence.

---

### Overly broad catch

**Is** — Catching more than the code knows how to handle.

**Signals** — `catch (Exception)` / `except Exception:` / `catch (e)` around a large block.
Catching at a level that cannot distinguish a network timeout from a null dereference. A
retry wrapper that retries programming errors. A catch that also swallows cancellation.

**Cause** — Catching by location instead of by failure mode.

**Why agents** — Narrow catch requires knowing which exceptions the block can raise, which
requires reading the callee.

**Risk** — Bugs are converted into "handled" states. A `TypeError` in the handler becomes a
retried request. Cancellation gets swallowed and the operation keeps going.

**Trajectory** — The failure surface becomes uniform and uninformative; nothing can be
handled specifically because nothing is distinguishable.

**Exception** — A top-level boundary — a request handler, a job runner, a supervisor loop —
whose job is to keep the process alive. It must log with full context and re-raise fatal
kinds.

**False positive** — The language has no narrower type available.

**Fix** — Catch the specific kinds you can act on. Let the rest propagate. Exclude
cancellation explicitly.

**Proof** — A test that an unexpected error type is not silently handled.

---

### Silent fallback

**Is** — Substituting a default when the real value could not be obtained, without saying so.

**Signals** — `catch { return [] }`. A config read failure returning built-in defaults. A
feature flag defaulting to `false` when the flag service is down. A currency conversion
falling back to 1.0. A permission check failing open. A cache miss on error returning stale
or empty data as if fresh.

**Cause** — Optimizing for "it keeps working" over "it tells the truth."

**Why agents** — Fallbacks look resilient and produce code with no visible failure path.

**Risk** — Wrong results presented as right ones. In an authorization path, failing open is
S1. In pricing, a fallback rate is a financial incident.

**Trajectory** — The system develops a degraded mode nobody knows it has, and metrics look
healthy throughout.

**Exception** — A genuine graceful-degradation requirement — with the degradation *visible*:
counted, logged, surfaced to the caller, or reflected in a response field.

**False positive** — The default is the specified behaviour for the empty case, not for the
error case. Read whether the fallback is on the error path.

**Fix** — Distinguish "no data" from "could not get data." Fail closed on anything
security-relevant. Where degradation is intended, make it observable and let the caller know.

**Proof** — A test asserting that a dependency failure is distinguishable from an empty result.

---

### Misleading recovery

**Is** — Handling that claims to recover but leaves the system inconsistent.

**Signals** — A `catch` that logs "recovered" and continues with partially mutated state. A
rollback that only reverses some steps. A compensating action that can itself fail, unhandled.
A cleanup in `finally` that assumes setup succeeded.

**Cause** — Writing recovery as a shape rather than reasoning about the state at the moment of
failure.

**Why agents** — Recovery blocks are highly patterned; what state exists at the catch point is
specific to this code.

**Risk** — The corrupted state is now *blessed* — the system believes it recovered.

**Trajectory** — Inconsistencies with no error trail, which is the most expensive kind.

**Exception** — Documented partial recovery with a reconciliation path.

**False positive** — Recovery is genuinely complete and you did not trace it.

**Fix** — Make failure atomic where possible. Where compensation is required, handle its
failure explicitly and escalate. Verify what `finally` assumes.

**Proof** — A test that fails mid-operation and asserts a consistent end state.

---

### Generic error erasing semantics

**Is** — Replacing a specific failure with an undifferentiated one.

**Signals** — `throw new Error('Something went wrong')`. A 500 for what was a 409. Every
failure mapped to one error type. A stringified error with the cause dropped. `catch (e) {
throw new AppError('failed') }`.

**Cause** — Treating errors as messages for humans rather than as values callers branch on.

**Why agents** — A single generic error type is simpler and reads as tidy.

**Risk** — Callers cannot respond appropriately. A retryable failure and a permanent one look
identical, so either everything is retried or nothing is.

**Trajectory** — Support and on-call cannot triage; every incident starts from zero.

**Exception** — A deliberate boundary that maps internal errors to a public taxonomy — which
is *preserving* semantics in a different vocabulary, not erasing it. It must keep the internal
cause on the inside.

**False positive** — The specific type is preserved in a `cause` chain you did not check.

**Fix** — Preserve the discriminant. Wrap with context and keep the cause. Map to a public
taxonomy at the edge, with distinguishable categories.

**Proof** — A test asserting the specific error type or code survives the boundary.

---

### Retrying non-idempotent effects

**Is** — Automatic retry applied to something that must not repeat.

Cross-listed with [`state-and-concurrency.md`](state-and-concurrency.md) § Unsafe retry,
because it is reached from both directions: as a concurrency defect and as an error-handling
reflex. Treat it as one finding, not two.

**Signals** — A generic retry decorator on a method that writes, charges, sends, or emits. A
retry on timeout without an idempotency key. A queue with automatic redelivery consumed by a
non-idempotent handler.

**Fix** — Make the effect idempotent first, then retry. Never the other way around.

**Proof** — A test that forces a retry and asserts exactly one effect.

---

### Error context discarded

**Is** — Losing the information needed to diagnose the failure.

**Signals** — `throw new Error(e.message)` — dropping the stack and the cause. Logging
`err.message` alone. Catching, logging, and re-throwing a fresh error. An error message with
no identifier: which user, which order, which file, which request.

**Cause** — Treating the message as the error.

**Why agents** — The message is the human-readable part, and human-readable is what generation
optimizes for.

**Risk** — The error is unactionable. On-call has a symptom and no subject.

**Trajectory** — Logs grow while diagnosability falls.

**Exception** — Deliberately redacting sensitive values at a boundary — which should redact,
not discard, and should keep a correlation id.

**False positive** — Context is attached by a logging framework or a middleware.

**Fix** — Wrap with cause preserved (`cause:`, `%w`, `raise ... from e`, `.context()`). Include
the identifiers needed to find the subject. Redact values, keep references.

**Proof** — A test asserting the cause chain survives; an example log line with its identifiers.
