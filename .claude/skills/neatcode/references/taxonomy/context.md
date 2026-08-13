# Context failures

Failures of situation. Each change is defensible read on its own and wrong read in place.
This is the signature failure of a system that generates from a prompt plus a few open files,
and the reason the change envelope exists.

---

### Local plausibility, global inconsistency

**Is** — Code that is correct, idiomatic, and reasonable in isolation, and contradicts how the
rest of the system works.

**Signals** — A new error style in a repository with one established style. A second date
handling approach. A hand-rolled retry beside a shared retry utility. A different naming
grammar for the same concept. A module that returns `null` where every sibling throws.

**Cause** — Optimizing the visible window. The prompt and the open file were the whole world.

**Why agents** — Context is finite and the surrounding repository is usually not in it. The
generated code is the average of a million repositories, not a continuation of this one.

**Risk** — Every reader now has to hold two models of how the system works.

**Trajectory** — This is how a codebase acquires three of everything. Each addition is
individually defensible; the aggregate is unlearnable.

**Exception** — A deliberate, documented migration to a new approach, where the new code is
the target state. Check for an ADR or a migration note before reporting.

**False positive** — The "inconsistency" is a genuine boundary — the module really does have
different constraints (a public SDK surface, a legacy adapter, a vendored fork).

**Fix** — Adopt the local idiom. If the local idiom is genuinely worse, that is one finding
about the idiom, not a licence to start an island.

**Proof** — The new code is indistinguishable in style and structure from its neighbours.

---

### Repository conventions not inspected

**Is** — Writing code without reading the instructions, conventions, or established patterns
that govern the area.

**Signals** — A change that violates something plainly stated in `AGENTS.md`,
`CONTRIBUTING.md`, or an ADR. A new dependency where the repository has a documented
preference. A test in a shape the project does not use. Files placed by general convention
rather than this project's convention.

**Cause** — Skipping orientation. The single highest-yield step, skipped because the task
looked small.

**Why agents** — Reading instructions costs tokens and produces no visible output; generating
code produces visible output immediately.

**Risk** — Rework, review friction, and a reviewer who now distrusts the whole change.

**Trajectory** — Conventions that are routinely violated stop being enforced, and the
instruction files become decoration.

**Exception** — The instructions are stale and contradicted by the code itself. Then the
contradiction is the finding.

**False positive** — A nested instruction file governs the changed directory and says
something different from the root one. Nested rules win; check for them.

**Fix** — Read the governing instructions, then conform. Cite them when reporting.

**Proof** — Quote the rule and the line that now satisfies it.

---

### Duplicate implementation

**Is** — Writing a capability the repository already has.

**Signals** — A new `formatCurrency`, `slugify`, `deepMerge`, `parseDuration`, `retry`, or
`isValidEmail` alongside an existing one. A second HTTP client wrapper. A hand-written
pagination loop next to a shared paginator. A private copy of a shared constant.

**Cause** — Not searching before writing. It is faster to produce a function than to find one.

**Why agents** — Generation is the default action. Search is a deliberate choice that must be
made before the generation reflex fires.

**Risk** — Two behaviours diverge on the first edge case — timezone handling, unicode,
rounding — and the difference is invisible until it produces a wrong number.

**Trajectory** — Bug fixes get applied to one copy. The other copy becomes a trap.

**Exception** — The existing implementation genuinely does not fit and adapting it would
couple unrelated modules. Say why, and prefer extending it over duplicating it.

**False positive** — Superficial name similarity with different semantics. Read both before
reporting.

**Fix** — Delete the new one and call the existing one, or extend the existing one. If the
existing one is in the wrong place, moving it is a separate change.

**Proof** — One implementation, one test suite, all call sites through it.

---

### Canonical path bypassed

**Is** — Reaching past the module that owns a capability and going direct.

**Signals** — A raw SQL query where a repository layer exists. `fetch()` where a configured
client exists. Direct mutation of a struct where a transition function exists. Reading
`process.env` where a validated config module exists. Constructing a domain object without
its factory.

**Cause** — The direct route is shorter and visible; the canonical route requires knowing it
exists.

**Why agents** — The canonical path is a fact about *this* repository. The direct path is a
fact about programming in general, which is what the model actually knows.

**Risk** — Every guarantee the canonical path provides — validation, connection pooling,
retry policy, auditing, cache invalidation, permission checks — is skipped exactly once, and
that once is the bug.

**Trajectory** — The canonical path stops being canonical. It becomes one of the ways.

**Exception** — A genuine escape hatch — a migration script, a performance-critical path with
a measurement, a test fixture. It should be visibly exceptional and preferably commented with
its reason.

**False positive** — The "canonical path" is aspirational, and most existing code already
bypasses it. That is a repository-level S3 finding, not a finding against this change.

**Fix** — Route through the owner. If the owner lacks a needed capability, add it there.

**Proof** — The guarantee the canonical path provides is demonstrably applied — a test that
fails when the bypass is reintroduced is ideal.

---

### Callers and integration points unexamined

**Is** — Changing something without looking at what depends on it.

**Signals** — A signature change with no call-site updates in the diff. A returned type
widened or narrowed. A default changed. A field renamed in a serialized structure. A thrown
error type altered. An exported symbol removed.

**Cause** — Treating the edited file as the unit of work.

**Why agents** — The open file is in context; its callers are not, and finding them requires
a deliberate search that generation does not prompt.

**Risk** — Compile errors if you are lucky, silent behaviour change if you are not — and
serialized formats fail at the worst possible distance from the change.

**Trajectory** — Downstream repositories, deployed clients, and persisted data carry the
break long after the diff is forgotten.

**Exception** — A genuinely private symbol with a compiler-enforced boundary.

**False positive** — Callers exist but are updated in a sibling commit the user has named.

**Fix** — Enumerate callers, update them, and say how you searched — including what the
search could not see (dynamic dispatch, reflection, string-constructed names, other repos,
persisted data, external clients).

**Proof** — Build and tests across the whole workspace, plus an explicit statement of the
search's limits.
