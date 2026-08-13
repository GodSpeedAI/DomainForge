# Authority failures

Failures of ownership. More than one place decides the same thing. Each site is correct today;
they will not stay correct together. This family produces the highest-interest debt in the
catalogue, because the cost is paid on every future change rather than once.

The rule underneath all of it: **one invariant, one owner.**

---

### Duplicated validation

**Is** — The same rule enforced in more than one place.

**Signals** — Email format checked in a controller and in a domain constructor. A length limit
in a schema, a DTO, and a database constraint with three different limits. A permission
checked in a route guard and again in a service, with slightly different conditions. Range
checks in both a client and a server that disagree about the boundary.

**Cause** — Adding a check where the bug was observed rather than where the rule belongs.

**Why agents** — Adding a guard at the point of failure is the locally obvious fix, and it
works.

**Risk** — The copies diverge. Then behaviour depends on which entry path the request took,
which is invisible in any single file.

**Trajectory** — Nobody dares remove any copy, because nobody can prove which one is load-
bearing. The rule becomes uneditable.

**Exception** — Defence in depth across a trust boundary is legitimate and often required:
client-side validation for UX, server-side for security, database constraint for integrity.
The rule is that the *authoritative* one is identified and the others are explicitly
secondary — and that they cannot disagree.

**False positive** — Different rules that look similar. Read both conditions carefully; a
`>=` and a `>` is a different rule, not a copy.

**Fix** — Name the owner. Other sites call it or are removed. Where defence in depth applies,
derive the secondary checks from the same constant or schema.

**Proof** — One test suite for the rule; changing the rule in one place changes every path.

---

### Duplicated transition logic

**Is** — More than one place that moves an entity between states.

**Signals** — Status set directly by an assignment in one path and by a transition function in
another. Two code paths that both cancel an order. A state machine plus a scattering of
`if (status === 'x') status = 'y'`. An admin path that bypasses the normal lifecycle.

**Cause** — Adding a new entry point without routing it through the existing lifecycle.

**Why agents** — The transition function is a repository fact; direct assignment is a
programming fact.

**Risk** — Illegal transitions become reachable through exactly one path. Side effects
attached to the transition — events, audit records, notifications, timers — fire on some
paths and not others.

**Trajectory** — The state machine becomes descriptive rather than enforcing, and the real
rules exist only in whoever remembers them.

**Exception** — A migration or repair script, visibly exceptional and documented.

**False positive** — One path is a projection or a read model, not the authority.

**Fix** — One transition function per entity. All mutation through it. Make the state field
private or read-only where the language allows.

**Proof** — A test enumerating illegal transitions and asserting rejection, exercised through
every entry point.

---

### Duplicated normalization

**Is** — The same canonicalization performed in more than one place.

**Signals** — Trimming and lower-casing an identifier in three places. Two date parsers. Two
currency-rounding helpers. Path normalization in a util and inline at a call site. Unicode
normalization applied inconsistently.

**Cause** — Normalizing at the point of use rather than at the point of entry.

**Why agents** — Local normalization always works locally and is invisible from elsewhere.

**Risk** — Two representations of the same value coexist. Lookups miss. Deduplication fails.
Comparisons return false for equal things.

**Trajectory** — Defensive re-normalization spreads until every function normalizes its
inputs "just in case," and the actual rule is unknowable.

**Exception** — Normalizing at a genuine trust boundary, where inputs arrive unnormalized from
outside.

**False positive** — Different normalizations for different purposes (display vs comparison
vs storage). That is legitimate — but each should have exactly one owner.

**Fix** — Normalize once, at the boundary. Use a distinct type for the normalized form where
the language supports it, so the compiler enforces it.

**Proof** — A test that a value normalized at the boundary is never re-normalized downstream.

---

### Multiple owners for one invariant

**Is** — Two or more modules that each believe they are responsible for keeping something true.

**Signals** — A balance maintained by both an account module and a ledger module. A cache
invalidated by three different callers. A counter incremented in a handler and recomputed in a
job. A "denormalized" field updated from several places.

**Cause** — No explicit ownership decision, so ownership accrued by accident.

**Why agents** — Ownership is a design decision that lives nowhere in the code; each addition
looks like a local fix.

**Risk** — The invariant holds only if every owner is correct, in every order, under
concurrency. That conjunction fails.

**Trajectory** — Reconciliation jobs appear. Then reconciliation jobs for the reconciliation.

**Exception** — Genuinely partitioned responsibility, where each owner covers a disjoint
subset and the partition is documented.

**False positive** — One of them is a read-only observer.

**Fix** — Assign one owner. Others request changes through it. Record the decision in
`engineering.md` § Authority map.

**Proof** — Only the owner mutates; a test or a type prevents the others.

---

### Cross-module mutation

**Is** — A module reaching into another module's state and changing it.

**Signals** — Direct field assignment on another module's struct or object. Mutating a
collection returned by a getter. A global or singleton written from several modules. A
returned array modified in place by its caller. Shared configuration mutated at runtime.

**Cause** — Data structures exposed without an ownership rule.

**Why agents** — If it is reachable and mutable, mutating it is the shortest path, and nothing
in the code objects.

**Risk** — Invariants cannot be maintained by the owner, because the owner is not in the call
stack when the change happens.

**Trajectory** — Debugging requires reading every module, since any of them might be the
mutator.

**Exception** — An explicitly shared mutable structure with a documented protocol — a builder,
an accumulator passed by design.

**False positive** — The language enforces value semantics and the "mutation" is on a copy.

**Fix** — Return copies or immutable views. Provide mutation methods on the owner. Where the
language has visibility control, use it.

**Proof** — External mutation no longer compiles, or a test detects it.

---

### Shared module as authority sink

**Is** — A generic `shared/`, `common/`, `core/`, `utils/`, or `lib/` module that has quietly
become where important behaviour lives.

**Signals** — A `utils` module imported by everything, containing business rules. A `common`
package with domain types in it. A `core` module that depends on three feature modules — the
dependency arrow pointing the wrong way is the giveaway. A "helpers" file over 500 lines.

**Cause** — "Shared" is the default answer to "where does this go?" when nobody wants to
decide.

**Why agents** — Placing a function in `utils` is never wrong locally and requires no
knowledge of the domain.

**Risk** — The module becomes a dependency hub. It cannot be changed safely and cannot be
split. It also becomes a cycle magnet.

**Trajectory** — Terminal. A shared module that has accreted domain logic for two years is
effectively unmovable.

**Exception** — Genuinely generic, domain-free utilities with no upward dependencies —
string padding, a typed event emitter, a result type.

**False positive** — A deliberate shared kernel in a DDD system, with documented contents and
an enforced dependency rule.

**Fix** — Move domain behaviour to its domain. Keep `shared` for things with no knowledge of
the domain and no dependency on it. The test is directional: shared may not import features.

**Proof** — A dependency test asserting the shared module imports nothing from feature
modules.
