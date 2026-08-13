# Change archetypes

Loaded at Step 2. Name one. The archetype decides **which rules bind hardest** — without it,
the rubric collapses into universal minimalism, which is wrong for a safety-critical path and
wrong for a throwaway script in opposite directions.

Pick the dominant one. A change can touch several; name the one whose failure would matter
most, and mention the second if it changes the scrutiny.

---

### Pure transformation

Deterministic input to output. No state, no effects, no clock, no randomness.

**Bias toward** directness and exhaustive tests. This is the archetype where property-based
and table-driven tests pay most.
**Bind hardest** — contract (edge cases), tests (weak assertions), abstraction (nothing
earns a class here).
**Relax** — observability, concurrency, state.
**Smell** — a class with one method and no state. It is a function.

### Stateful domain operation

Modifies entities under invariants. The heart of most business software.

**Bias toward** explicit transitions, invariants owned once, illegal states unrepresentable.
**Bind hardest** — authority, contract, state-and-concurrency, tests.
**Relax** — nothing much. This archetype earns its scrutiny.
**Smell** — a status field assigned directly; an invariant checked at the call site.

### Boundary adapter

Translates between this system and an external one: an API client, a queue consumer, a
database gateway, a file format reader.

**Bias toward** isolating the foreign vocabulary. Explicit timeouts, retries, and error
translation. Nothing external crosses inward.
**Bind hardest** — boundary, failure-handling, state-and-concurrency (idempotency, retry
safety), security (trust boundary).
**Relax** — domain modelling depth.
**Smell** — the external library's types appearing in domain modules; a client with no
timeout.

### Workflow orchestration

Coordinates several capabilities toward an outcome. Multi-step, often long-running.

**Bias toward** making transitions and compensation visible. Each step's failure has a named
consequence.
**Bind hardest** — state-and-concurrency (atomicity, idempotency), failure-handling
(compensation), observability, completion (partial states).
**Relax** — performance.
**Smell** — a sequence of awaits with no consideration of failure between them.

### Query / read model

Retrieves and shapes information. No mutation.

**Bias toward** correctness of filtering and authorization scope, and predictable performance.
**Bind hardest** — security (tenant and permission scoping), performance under real data
volume, contract (shape stability).
**Relax** — transaction boundaries, idempotency.
**Smell** — a query with no scoping by the requester's authority; N+1 access patterns.

### Infrastructure mechanism

Storage, transport, scheduling, caching, telemetry. Underneath the domain.

**Bias toward** resource bounds, lifecycle, and configuration that is honest.
**Bind hardest** — state-and-concurrency, observability, completion (wiring), boundary
(nothing domain-specific leaks in).
**Relax** — domain vocabulary.
**Smell** — business rules embedded in the mechanism; an unbounded queue or buffer.

### Interactive application flow

User-driven state transitions: UI flows, CLIs, wizards, forms.

**Bias toward** every state being representable and reachable — loading, empty, error,
partial, success — and cancellation being real.
**Bind hardest** — contract (edge states), failure-handling (what the user sees), state
(cancellation, stale responses), accessibility where applicable.
**Relax** — deep architectural layering.
**Smell** — only the success state implemented; a stale async response overwriting newer state.

### Compiler or transformation pipeline

Parse, normalize, validate, lower, emit. Staged transformation with named intermediate forms.

**Bias toward** stage boundaries that hold, diagnostics that accumulate rather than abort, and
deterministic output.
**Bind hardest** — boundary (stage separation), contract (IR stability), tests (fixture-based
round-trips).
**Relax** — runtime observability.
**Smell** — a stage that reaches back into the previous representation; a parser that also
resolves names.

### Concurrency-sensitive operation

Ordering and shared state determine correctness.

**Bias toward** explicit synchronization discipline, named and scoped. Prefer database-level
atomicity over application locks.
**Bind hardest** — state-and-concurrency, all of it. Every entry in that family applies.
**Relax** — nothing. Escalate to **Deep** depth automatically.
**Smell** — check-then-act; a lock that protects some mutations and not others.

### Safety-critical path

Failures require explicit containment and evidence: payments, auth, medical, physical control,
data destruction.

**Bias toward** redundancy that would be unearned elsewhere. Fail closed. Prove it.
**Bind hardest** — security, contract, evidence, state-and-concurrency.
**Relax** — restraint, deliberately. Defence in depth is earned here; the earnedness test's
constraint is the criticality itself.
**Smell** — a single check with no independent verification; a fallback that fails open.

### Repository maintenance

Migrations, cleanup, dependency updates, mechanical changes, formatting, tooling.

**Bias toward** mechanical uniformity, complete application, and a clean separation from
behavioural change.
**Bind hardest** — change-discipline (all of it), completion (did it apply everywhere?),
contract (did any behaviour change?).
**Relax** — architecture, abstraction.
**Smell** — a "mechanical" change containing one hand-edited exception; a dependency bump
bundled with a fix.

---

## Using the archetype

State it in one clause and let it do work:

> *"Boundary adapter — so the Stripe types stop at this module, and the retry needs an
> idempotency key before it is safe."*

Two ways this goes wrong:

- **Picking the flattering archetype.** Labelling a stateful domain operation a "pure
  transformation" because that relaxes the state rules. The archetype describes the change,
  not the scrutiny you would prefer.
- **Applying one archetype's bias universally.** Stripping a safety-critical path's redundancy
  because restraint said so is the same error as adding a factory to a pure function.
