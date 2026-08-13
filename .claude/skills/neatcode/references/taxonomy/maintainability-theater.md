# Maintainability theater

Failures of appearance. Code that performs the rituals of professionalism without their
benefits. Every entry here looks like care and costs like carelessness.

The family is easy to over-report. Nothing here is an S1, and a review consisting mostly of
these findings is a review that missed the real problems. Check the severe families first.

---

### Excessive ceremony

**Is** — Structure and process around a simple thing, out of proportion to it.

**Signals** — A builder for a two-field struct. A dedicated exception type for a condition
handled in one place. A five-layer call chain to read a value. A factory, an interface, an
implementation, and a registration for one function's worth of behaviour. Configuration
objects for a function with two parameters.

**Cause** — Applying the full form of a pattern regardless of scale.

**Why agents** — Patterns appear in training data at their fully-elaborated form, since that is
how they are taught.

**Risk** — Reading cost with no corresponding benefit; the real logic is diluted.

**Trajectory** — The ceremony becomes the local convention and every subsequent addition
matches it.

**Exception** — Framework requirements. A published API where the ceremony *is* the contract.

**False positive** — The ceremony pays for something you did not see — a serialization
requirement, a DI constraint, a testing seam.

**Fix** — Collapse to the direct form.

**Proof** — Fewer constructs, same behaviour, same tests.

---

### Generic utility proliferation

**Is** — A growing collection of small, weakly-related helpers.

**Signals** — `utils.ts`, `helpers.py`, `common.go`, `misc.rb`. Functions with no shared theme
in one file. A helper used once. Several files whose names are all synonyms for "assorted."

**Cause** — "Where does this go?" answered with "somewhere generic."

**Why agents** — Placing a function in `utils` is never locally wrong.

**Risk** — Discoverability collapses. Duplicates appear because nobody finds the existing one.

**Trajectory** — See [`authority.md`](authority.md) § Shared module as authority sink. This is
its early stage.

**Exception** — A small, genuinely generic, well-named module with a stated theme.

**False positive** — A cohesive module whose name happens to be generic.

**Fix** — Move each helper next to the thing it serves. Name modules for their theme, not for
their genericity.

**Proof** — No module's name is a synonym for "assorted."

---

### Taxonomic sprawl

**Is** — A proliferation of role-suffixed types whose distinctions are not real.

**Signals** — `UserService`, `UserManager`, `UserHandler`, `UserProcessor`, `UserHelper`,
`UserProvider`, `UserFactory` in one codebase, with no stated rule for which is which. A new
suffix invented for each addition. Types whose names describe a grammatical role rather than a
responsibility.

**Cause** — Naming by convention-shaped noise rather than by responsibility.

**Why agents** — These suffixes are enormously frequent in training data and carry the *feel*
of architecture without any semantic commitment.

**Risk** — Nobody can predict where anything lives. Every lookup is a search.

**Trajectory** — Placement becomes arbitrary, which makes duplication near-certain.

**Exception** — A repository with a documented, consistently applied vocabulary — where
`Service` and `Repository` have defined meanings. That is a real taxonomy; honour it.

**False positive** — A framework convention (Rails controllers, Spring services, NestJS
providers).

**Fix** — Name for what it does. Where a vocabulary exists, document and apply it. Where it
does not, prefer names that state a responsibility.

**Proof** — Each type's responsibility is stateable in one sentence without "and."

---

### Documentation that restates code

**Is** — Prose that repeats the implementation instead of recording the decision.

**Signals** — A docstring listing parameters that adds nothing to their names and types. A
README describing the folder structure. A comment above `getUserById` saying "gets a user by
id." An architecture document that is a class inventory.

**Cause** — Documenting what is visible, because it is easy, rather than what is invisible.

**Why agents** — Generating a description of visible code is trivial; recording a rationale
requires knowing one.

**Risk** — Documentation rots and misleads, because it duplicates something that changes.

**Trajectory** — Nobody reads the docs; nobody updates the docs; the docs become actively
false.

**Exception** — Generated API reference, where completeness is the point.

**False positive** — A docstring documenting non-obvious constraints, units, ownership, or
error behaviour. That is valuable.

**Fix** — Document why: the constraint, the rejected alternative, the invariant, the incident
that produced this. Delete the rest.

**Proof** — Each retained sentence says something the code cannot.

---

### Comments describing syntax

**Is** — A comment narrating what the next line does.

**Signals** — `// increment the counter`, `// loop over users`, `// return the result`,
`# set the value`. A comment on every line. Section banners in a 20-line function.

**Cause** — Comment density mistaken for care.

**Why agents** — Instructional code — which is heavily represented in training data — is
commented line by line, because it is teaching the language.

**Risk** — Real comments become invisible in the noise, and comments drift out of sync with
the code they narrate.

**Trajectory** — Readers stop reading comments entirely, including the load-bearing ones.

**Exception** — Genuinely non-obvious operations: bit manipulation, a numerical stability
trick, a workaround for a specific library bug with a link.

**False positive** — A comment explaining *why* that superficially looks like *what*.

**Fix** — Delete narration. Keep intent, constraint, and rationale.

**Proof** — Every comment says something the code does not.

---

### One-file-per-gesture fragmentation

Cross-listed with [`abstraction.md`](abstraction.md) § Unnecessary fragmentation. Same defect
seen from the file-layout side rather than the module-design side. Report it once.

**Signals** — A directory of single-function files. A `types.ts` + `constants.ts` +
`utils.ts` + `index.ts` quartet per feature, each under 20 lines. Understanding one behaviour
requires opening five files.

**Fix** — Colocate what changes together.

---

### False configurability

**Is** — Options that appear to change behaviour and do not.

**Signals** — A setting read once at startup and ignored. An option with one supported value.
A flag whose branches are identical. A `strategy` parameter with one strategy. Environment
variables documented in a README and read nowhere.

**Cause** — Building the interface of flexibility without the mechanism.

**Why agents** — Configuration blocks are a strong pattern and read as maturity.

**Risk** — Operators change a value and nothing happens; trust in the whole configuration
surface drops.

**Trajectory** — Nobody can determine which settings are real. Removal becomes impossible.

**Exception** — Documented, reserved-for-future settings — with a stated date.

**False positive** — Read dynamically, or by an external system. Search for the string.

**Fix** — Wire it or delete it. Deleting is usually correct.

**Proof** — A test where the setting changes observable behaviour.

---

### False extensibility

**Is** — Extension points nothing can actually extend.

**Signals** — A plugin interface with no loading mechanism. A hook array never populated. An
`onX` callback nothing supplies. A "pluggable" backend selected by a hard-coded constant. An
abstract method with one override and no registration.

**Cause** — Building the shape of extension rather than the mechanism.

**Why agents** — Extension-point vocabulary is highly patterned; the loading, registration, and
lifecycle are project-specific.

**Risk** — The next person who tries to extend it discovers there is nothing to extend, after
building against it.

**Trajectory** — Two extension mechanisms — the fake one and the real one someone eventually
adds.

**Exception** — A published extension API with documentation and at least one external
consumer.

**False positive** — The mechanism exists in a wiring module you did not open.

**Fix** — Delete it, or complete it — registration, discovery, lifecycle, and a second
implementation proving it works.

**Proof** — A second implementation registers and runs through the mechanism.

---

### Architecture terminology unsupported by enforcement

**Is** — Naming an architecture that nothing maintains.

Cross-listed with [`abstraction.md`](abstraction.md) § Architecture cosplay and covered fully
by [`../architecture/phenotype.md`](../architecture/phenotype.md). Report it once, from the
phenotype protocol, with the conformance verdict attached — that is the version of this
finding with evidence behind it.

**Signals** — `domain`/`application`/`infrastructure` with bidirectional imports. "Bounded
contexts" sharing internal models. A "modular monolith" with no controlled public surfaces.
A "hexagonal" system whose production code constructs adapters directly.

**Fix** — Enforce the constraint with a test or lint rule, or rename to describe what exists.

**Proof** — A dependency test that fails when the rule is violated.
