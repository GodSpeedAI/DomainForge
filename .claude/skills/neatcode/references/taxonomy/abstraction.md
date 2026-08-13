# Abstraction failures

Failures of proportion. Structure was spent where no constraint earned it. Every entry here is
an application of the earnedness test in [`../restraint.md`](../restraint.md); read that file
first, and check its exceptions list before writing any finding from this family.

This is the family most likely to produce false positives, because the cost is invisible and
the justification is always available in the abstract. Require a *concrete* constraint.

---

### Premature abstraction

**Is** — Generalizing before a second case exists.

**Signals** — A type parameter with one instantiation. A strategy interface with one strategy.
A `BaseX` with one subclass. A config-driven dispatch table with one entry. A plugin system
with one plugin. A callback parameter every caller passes the same value for.

**Cause** — Designing for the imagined future instead of the observed present.

**Why agents** — "Extensible" code is over-represented in training data because tutorials
teach patterns using minimal examples, and the minimal example of a pattern is exactly this
shape.

**Risk** — Every reader traverses machinery that does nothing. The generalization is usually
wrong for the real second case when it arrives, because it was shaped by one example.

**Trajectory** — The abstraction ossifies. The second case gets bolted on awkwardly rather
than reshaping it, because reshaping now means touching the first case.

**Exception** — A published extension point that is the product. A framework contract. A
second implementation in the same change. A stated near-term requirement the user named.

**False positive** — A test double is a real second implementation *if* the seam is needed for
testing and no simpler seam exists. Check whether the code is testable without it first.

**Fix** — Collapse to the concrete case. Add the abstraction with the second implementation —
it will be a smaller change than it feels, and it will be shaped by two real cases.

**Proof** — The direct version passes the same tests with less code.

---

### Pass-through wrapper

**Is** — A type whose methods forward to a field, adding nothing.

**Signals** — Every method body is `return this.inner.sameName(...args)`. A "service" that
delegates each call to a repository one-to-one. A client wrapper that only re-exports. A
`*Manager` around a registry that adds no policy.

**Cause** — Adding a layer because the architecture diagram has one there.

**Why agents** — The layered shape is overwhelmingly common in training data, and reproducing
it reads as competence.

**Risk** — Low individually; the cost is navigational. Three of these and no one can find
where anything happens.

**Trajectory** — The wrapper eventually acquires one piece of real logic, which is now in the
wrong place, and removing the wrapper becomes a real refactor.

**Exception** — The wrapper isolates an external vocabulary at a boundary (that *is* policy),
implements a published interface, or provides a lifecycle the inner type lacks. Also: a
deliberately thin facade over a volatile third-party surface, when the volatility is real.

**False positive** — One of the methods adds something and you skimmed. Read every method.

**Fix** — Delete it; call the inner type. If a boundary is genuinely wanted, give it a real
job — translation, validation, policy — or drop it.

**Proof** — Inlining shortens call sites and changes nothing else.

---

### Interface without demonstrated variation

**Is** — A contract extracted from exactly one thing.

**Signals** — `interface UserRepository` with `PostgresUserRepository` as the only
implementation, and no second database on the roadmap. An `IEmailSender` with one sender.
A protocol whose only conformer is defined immediately below it.

**Cause** — Applying dependency inversion as a rule rather than as a response to a force.

**Why agents** — "Depend on abstractions" is stated as a universal principle in most of the
material the model learned from, without the conditions that make it pay.

**Risk** — Indirection with no payoff, and — more expensively — the interface tends to be a
*mirror* of the one implementation, so it constrains nothing and abstracts nothing.

**Trajectory** — The second implementation, when it arrives, does not fit, and the interface
grows optional methods and capability checks.

**Exception** — Test seams where no simpler one exists. Published SDK contracts. A framework
that requires registration by interface. A real second implementation, present or imminent.

**False positive** — The interface is the module's public API and the implementation is
internal. That is a legitimate encapsulation boundary, not a speculative abstraction.

**Fix** — Use the concrete type. Introduce the interface when the second implementation does.

**Proof** — The concrete version compiles and tests unchanged.

---

### Speculative generality

**Is** — Parameters, options, hooks, and branches for cases that do not exist.

**Signals** — An options object where every caller passes `{}`. A `format` parameter with one
accepted value. A `version` field always `1`. Dead branches for a mode nothing selects. A
callback nothing supplies. A generic container used with one type.

**Cause** — Anticipating requirements rather than eliciting them.

**Why agents** — Adding a parameter is cheap at generation time and reads as thoughtful.

**Risk** — Untested code paths that will be wrong when first used, and a signature that lies
about what the function can do.

**Trajectory** — Callers begin passing the unused options with plausible-looking values that
nothing honours.

**Exception** — Public API stability where the parameter is documented as reserved.

**False positive** — Used by a caller you did not search — a test, a script, another package
in the workspace.

**Fix** — Remove unused parameters and branches. Narrow types to what is used.

**Proof** — A search showing no caller supplies them; the build after removal.

---

### Architecture cosplay

**Is** — Directory names and vocabulary from a named architecture, without any of its
functional properties.

**Signals** — `domain/`, `application/`, `infrastructure/` folders where imports run in both
directions. `ports/` and `adapters/` where production code calls the adapter directly.
A "modular monolith" where every module imports every other module's internals. A "vertical
slice" architecture where each feature change touches five horizontal layers. "Event-driven"
where events are synchronous function calls in disguise.

**Cause** — Copying the visible artifact of an architecture — its folder tree — instead of its
constraint, which is the dependency rule.

**Why agents** — The folder structure is the most visible, most copyable, most frequently
illustrated part. The constraint is invisible in a file listing.

**Risk** — The team believes it has isolation it does not have, and reasons about change cost
using a model that does not describe the system.

**Trajectory** — The gap widens until the vocabulary becomes actively misleading, and new
contributors are systematically misled by the directory names.

**Exception** — A migration in progress with a stated target and a tracked plan.

**False positive** — Conformance is enforced somewhere you did not look — an architecture
test, a lint rule, a module system, a build-level boundary. Look for it before reporting.

**Fix** — Either enforce the constraint (dependency-direction test, lint rule, module
boundary) or rename the folders to describe what they are. Both are honest; the current state
is not. Run the conformance protocol in
[`../architecture/phenotype.md`](../architecture/phenotype.md) before writing this finding.

**Proof** — A test or lint rule that fails when the dependency rule is violated.

---

### Unnecessary fragmentation

**Is** — One cohesive operation split across files that must be read together to understand
any of them.

**Signals** — A directory of one-function files. A three-line "helper" imported once. Types,
constants, validation, and logic for one concept in four files. A `types.ts`, `constants.ts`,
`utils.ts`, `helpers.ts` quartet per feature. Following a single behaviour requires opening
five files.

**Cause** — Confusing file count with modularity. Small files are not the same as small
modules.

**Why agents** — "Single responsibility" is widely stated and widely misread as
"one thing per file."

**Risk** — Comprehension cost per change rises sharply; the useful unit of reasoning is
scattered.

**Trajectory** — The import graph becomes dense and circular-ish; moving anything touches
everything.

**Exception** — Genuine reuse from multiple modules. A repository whose consistent convention
is fine-grained files — consistency wins.

**False positive** — Language or framework convention (Go's package-per-directory, Rails'
conventions, a component-per-file UI framework).

**Fix** — Colocate what changes together. One module per concept, not one file per function.

**Proof** — The behaviour is readable in one place; the import graph is shallower.
