# Boundary failures

Failures of separation. Something crossed an edge it was supposed to stop at. The edge is
usually still there in the folder names; only the enforcement is missing.

Before writing any finding here, establish that the boundary is *real* — claimed and enforced,
or at least consistently honoured. A boundary that exists only in a README is a different
finding: see architecture cosplay in [`abstraction.md`](abstraction.md) and the conformance
protocol in [`../architecture/phenotype.md`](../architecture/phenotype.md).

---

### Infrastructure leaking into domain logic

**Is** — Business logic that knows about storage, transport, or framework mechanics.

**Signals** — SQL inside a domain entity. HTTP status codes chosen in a business rule. A
domain function taking a `Request` or a database connection. Serialization annotations on
domain types. `res.status(404)` inside a pricing calculation. A domain method that awaits a
network call it did not need.

**Cause** — Writing top-down from the entry point; whatever the handler had is what the
callee receives.

**Why agents** — The handler is where the task starts, and passing what is in scope is the
path of least resistance.

**Risk** — The rules cannot be tested or reused without the infrastructure. The business logic
becomes a hostage to the framework's lifecycle.

**Trajectory** — Every framework upgrade is a business-logic migration.

**Exception** — Deliberately transaction-script-shaped code, consistently applied. A small
service with no layering claim. Do not impose hexagonal architecture on a repository that
never claimed it.

**False positive** — A shared type that merely resembles a framework type.

**Fix** — Pass the values the rule needs, not the carriers. Return decisions, not responses;
let the boundary translate.

**Proof** — The domain function is callable and testable with plain values and no framework.

---

### Framework types crossing an intended boundary

**Is** — A framework or library type used as the currency of an inner layer.

**Signals** — Express `Request`/`Response`, Django `HttpRequest`, Spring `ResponseEntity`,
`sqlx::Row`, `mongoose.Document`, a Protobuf message, a Kafka `ConsumerRecord`, or a
`context.Context` carrying business values — appearing in modules that claim independence.

**Cause** — Not translating at the edge, because translation looks like pointless copying.

**Why agents** — Translation code is boilerplate; skipping it makes the diff smaller and the
code "DRYer."

**Risk** — The whole inner layer inherits the framework's version, its lifecycle, and its
breaking changes.

**Trajectory** — The framework becomes unswappable and untestable, which is precisely the
property the boundary existed to prevent.

**Exception** — A pure adapter layer whose job *is* the framework type. A stable,
generational library the project has explicitly adopted as its vocabulary (a decimal type, a
date library) — that is a chosen dependency, not a leak.

**False positive** — The type is the project's own, in a package that happens to be named
like infrastructure.

**Fix** — Translate at the boundary into domain types. Yes, that means mapping code; that is
the cost the boundary buys.

**Proof** — The inner module's imports contain no framework packages — assert it with a test.

---

### ORM entity as universal domain model

**Is** — A persistence-mapped class used as the domain model, the API contract, and the
internal data structure at once.

**Signals** — An `@Entity` class returned directly from an HTTP handler. Lazy-loading errors
outside a session. Serialization annotations and business methods on the same class. A domain
invariant that cannot be enforced because the ORM needs a no-arg constructor and public
setters. Migrations forced by an API change.

**Cause** — One shape is fewer shapes. It is genuinely convenient right up until it is not.

**Why agents** — Overwhelmingly common in tutorials and starter templates.

**Risk** — Persistence concerns dictate the domain model's shape. API changes force schema
changes. Internal fields leak into responses — including ones that should never be public.

**Trajectory** — The model accretes fields for every consumer; nobody can tell which are
persisted, which are computed, and which are API-only.

**Exception** — A small CRUD service where the entity genuinely *is* the model, stated as a
deliberate choice. This is a legitimate architecture, not a failure — do not report it in a
system that made that choice on purpose.

**False positive** — A DTO layer exists and you were looking at the wrong file.

**Fix** — Separate the shapes that have separate reasons to change: persistence model, domain
model, API contract. Start with the API contract; it is where the leak costs most.

**Proof** — Adding a persisted field does not change any API response.

---

### Business logic in transport or persistence layers

**Is** — Rules living in controllers, resolvers, repositories, or migrations.

**Signals** — Pricing arithmetic in a route handler. Eligibility rules in a SQL `WHERE`
clause that nothing else can reach. A discount computed in a GraphQL resolver. State
transitions inside a repository `save`. Business defaults applied in a database trigger.

**Cause** — Implementing where the request arrives.

**Why agents** — The handler is the file that was open.

**Risk** — The rule is unreachable from other entry points — a job, a CLI, a second API
version — so it gets reimplemented.

**Trajectory** — Straight into duplicated authority; see [`authority.md`](authority.md).

**Exception** — Genuinely transport-specific concerns: pagination limits, content negotiation,
auth token parsing, rate limiting.

**False positive** — A thin handler that calls a domain function and formats the result. That
is correct.

**Fix** — Move the rule to its owner; let the handler adapt and delegate.

**Proof** — A second entry point exercises the same rule without duplicating it.

---

### Dependency direction violated

**Is** — An import that points the wrong way against a stated or consistently observed rule.

**Signals** — `domain/` importing from `infrastructure/`. A shared kernel importing a feature.
A lower layer importing an upper one. Two feature modules importing each other's internals. A
package cycle.

**Cause** — Needing a symbol and importing it, without asking which way the arrow is allowed
to point.

**Why agents** — The import resolves and the code compiles. Nothing objects.

**Risk** — The claimed isolation is fictional. Cycles make modules inseparable, untestable in
isolation, and often unbuildable in parallel.

**Trajectory** — Direction violations are individually trivial and collectively terminal. Once
there are enough, restoring the rule is a rewrite.

**Exception** — Dependency inversion done properly: the inner layer defines the interface, the
outer implements it, and the *import* still points inward. Check what is imported, not what is
called.

**False positive** — A type-only import erased at compile time. Legitimate in some languages,
still a coupling signal in others. Say which.

**Fix** — Invert with an interface owned by the inner layer, move the shared type down, or
move the behaviour up. Then enforce it with a test or lint rule — an unenforced rule will be
violated again next week.

**Proof** — An architecture test that fails on the violating import.

---

### Nominal ports and adapters

**Is** — A ports-and-adapters structure that production code routinely bypasses.

**Signals** — A `ports/` directory whose interfaces are implemented once and used only in
tests, while production constructs the adapter directly. An interface injected in one place
and imported concretely in five. A "repository interface" alongside direct query calls.

**Cause** — Adopting the structure without the enforcement.

**Why agents** — The structure is visible and copyable; the discipline is not.

**Risk** — Every claimed benefit — swappability, testability, isolation — is absent while the
cost is fully paid.

**Trajectory** — The ports become documentation of an intention nobody follows, and new
contributors reasonably conclude the pattern is optional.

**Exception** — A migration in progress with a stated target.

**False positive** — Composition happens in a wiring module you did not open. Look for it.

**Fix** — Either route production through the port and enforce it, or delete the port and use
the concrete type honestly.

**Proof** — No production module imports an adapter directly; a test asserts it.
