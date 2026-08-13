# Observability failures

Failures of visibility. The code runs; nobody can tell what it did. Load this family for
long-running services, background work, and any production-facing failure path.

The test for every entry: **name the operational question the signal answers.** A log line
that answers no question is noise with a timestamp.

---

### Meaningless log

**Is** — A log statement that conveys nothing beyond the fact that a line executed.

**Signals** — `log.info('starting')`, `log.debug('here')`, `console.log('done')`,
`log.info('processing')`, a log at the top of every function. Emoji-decorated progress lines.
Logging inside a tight loop.

**Cause** — Logging as a marker of activity rather than as a record of a decision.

**Why agents** — Log statements are a strong shape and read as operational maturity.

**Risk** — Volume without signal, real cost, and — at scale — the useful lines are crowded out.

**Trajectory** — Log level gets raised to suppress the noise, and the useful lines go with it.

**Exception** — Structured trace spans, which are the right tool for "this ran."

**False positive** — A debug-level line behind a flag, off in production.

**Fix** — Log decisions and outcomes with their subject: what happened, to what, with which
result. Delete the rest.

**Proof** — Each retained line answers a stated operational question.

---

### Missing correlation

**Is** — Log lines that cannot be tied to a request, a job, a tenant, or an entity.

**Signals** — No request id, trace id, job id, user id, or entity id. Logs from concurrent
work interleaved with nothing to separate them. A logger constructed without context. A
correlation id received at the edge and never propagated.

**Cause** — Logging locally without a plan for reading globally.

**Why agents** — The correlation identifier is not in scope at the log site, and plumbing it
is work.

**Risk** — In production, an interleaved log without correlation is unreadable. Debugging a
single failing request becomes impossible.

**Trajectory** — Investigations get abandoned; incidents close as "could not reproduce."

**Exception** — A single-threaded batch process with one unit of work.

**False positive** — Correlation injected by middleware or by an async-context mechanism.

**Fix** — Propagate a correlation id from the edge — via context, an async local, or a scoped
logger. Include the subject's identifier in every line about it.

**Proof** — One identifier retrieves the complete story of one request.

---

### Log without actionable state

**Is** — A message that reports a problem without the information needed to act on it.

**Signals** — `log.error('failed to save')`. `log.warn('invalid input')`. `catch (e) {
log.error('error', e.message) }`. A validation failure logged without the value or the field.
A retry logged without the attempt count or the reason.

**Cause** — Writing for a reader who already has the context — which is you, now, and nobody
later.

**Why agents** — The message is generated from the surrounding code and never read back from
an operator's perspective.

**Risk** — The signal exists and is useless. Worse, it is *reassuring* — someone will see it
and think the failure is handled.

**Trajectory** — Alerts fire on it and nobody can do anything, so the alert gets muted.

**Exception** — None. If it is worth logging, it is worth logging usefully.

**False positive** — Context is attached structurally by the logging framework.

**Fix** — Include the subject, the operation, the outcome, and the cause. Redact values; keep
references.

**Proof** — Read the line cold and answer: what failed, for whom, why, and what should be done.

---

### Silent degradation

**Is** — Working in a reduced mode with no signal that it happened.

**Signals** — A fallback with no counter or log. A cache miss path that silently queries a
slower source with no metric. A dependency timeout handled by returning partial results with
no indication. A feature flag defaulting off because its service is unreachable.

**Cause** — Treating degradation as a success rather than as a distinct state.

**Why agents** — The fallback branch is written as the recovery, and recovery reads as
completion.

**Risk** — The system runs degraded for weeks. Metrics look healthy because the degraded path
returns 200.

**Trajectory** — The degraded mode becomes the normal mode and nobody notices until the
primary path is discovered to have been broken all along.

**Exception** — Explicitly accepted, monitored degradation.

**False positive** — A metric exists that you did not find.

**Fix** — Count every entry into a degraded path and expose it. Reflect it in the response
where consumers need to know.

**Proof** — A dashboard or query that shows degraded-mode rate; an alert threshold on it.

---

### Metric disconnected from an operational question

**Is** — Instrumentation nobody can act on.

**Signals** — A counter with no alert and no dashboard. A gauge of an internal implementation
detail. Timing on a function nobody would tune. Metrics with unbounded label cardinality —
user id, request id, path with parameters. Metrics added because "we should have metrics."

**Cause** — Instrumenting what is easy rather than what is asked.

**Why agents** — Adding a counter is a strong shape and needs no operational knowledge.

**Risk** — Cost, cardinality explosions that take down the metrics backend, and dashboards
nobody trusts.

**Trajectory** — Metric sprawl; the real signals become unfindable.

**Exception** — Deliberate exploratory instrumentation with a stated removal date.

**False positive** — Used in a dashboard or alert you did not check.

**Fix** — For each metric, state the question and the decision it informs. Delete the ones with
no answer. Bound label cardinality.

**Proof** — Every metric maps to a dashboard panel or an alert.

---

### Failure path invisible in production

**Is** — An error branch that produces no observable signal.

**Signals** — A `catch` with no log, no metric, and no re-throw. A dropped message with no
dead-letter record. A validation rejection returning a 400 with nothing recorded. A background
job that fails and is never retried or reported.

**Cause** — The error path is written last and least, and it is the one path no test exercises.

**Why agents** — Instrumentation on the failure branch is the least-generated code in any
handler.

**Risk** — Failures are invisible until a user reports them, which is the slowest possible
detection channel.

**Trajectory** — Error rate is unknown, so error budgets and SLOs are fiction.

**Exception** — Expected, high-volume, benign rejections — where an aggregate counter is
correct and per-event logging is not.

**False positive** — Handled by a global error reporter.

**Fix** — Every failure path increments something. Unexpected failures log with context.
Dropped work goes to a dead-letter queue.

**Proof** — Inject the failure and confirm a signal appears where an operator would look.
