# Epistemic failures

Failures of knowing. The agent did not know something, and instead of resolving it or saying
so, it produced code shaped like knowledge. This family is first because it is the cheapest
to check and the most expensive to miss: every other failure can be found in review, and
these produce code that looks perfect and does not exist.

---

### Invented API

**Is** — Calling a method, field, option, or module that does not exist in the installed
version of a dependency, or does not exist at all.

**Signals** — A call whose name is a perfectly reasonable guess (`client.retryWithBackoff()`,
`config.getOrDefault()`, `user.hasPermission()`). An option passed to a library function
that its documentation does not list. An import from a submodule path that follows the
package's naming logic but is not exported. Code that would work in a neighbouring library.

**Cause** — Generating from the distribution of *plausible* APIs rather than from the actual
one. The model has seen ten thousand HTTP clients; it produces the average one.

**Why agents** — API surfaces are exactly the kind of thing that is highly predictable in
form and arbitrary in fact. Plausibility is maximal precisely where verification is required.

**Risk** — Immediate runtime or compile failure at best; at worst, a name that *does* exist
with different semantics.

**Trajectory** — Usually caught fast. When it is not — a dynamically dispatched call in a
rarely-run branch — it becomes a production incident with a confusing stack trace.

**Exception** — None. There is no version of this that is acceptable.

**False positive** — The symbol exists but your search missed it: re-exports, type-only
declarations, prototype extensions, metaprogramming, mixins, framework-injected methods.
Check the type definitions and the actual installed version before reporting.

**Fix** — Read the installed package: type definitions, the source in `node_modules`/`site-packages`/
the vendor directory, or the version-matched documentation. Use the real API. If the needed
capability genuinely does not exist, say so and propose the alternative.

**Proof** — Type check, or a test that executes the call path.

---

### Assumed version

**Is** — Writing against a dependency's behaviour without checking which version is installed.

**Signals** — Use of an API added in a later major version. Reliance on a default that
changed. Config in a format the installed version does not parse. Import paths from a
different major.

**Cause** — Treating "the library" as one thing rather than as a specific pinned artifact.

**Why agents** — Training data mixes versions freely, and the most-written-about version is
rarely the one installed.

**Risk** — Silent behaviour differences are worse than errors. A default that flipped between
majors produces working code with wrong semantics.

**Trajectory** — Compounds at the next upgrade, when nobody can tell which behaviours were
intended and which were accidents of the pinned version.

**Exception** — A repository that genuinely supports a version range and tests against it.

**False positive** — The lockfile and the manifest disagree; read the lockfile.

**Fix** — Read the lockfile or the installed metadata. Write against the version that is
actually there.

**Proof** — Name the version in the report. A test that would fail on the wrong version is
stronger.

---

### Fabricated guarantee

**Is** — Asserting a property — atomicity, ordering, idempotency, thread safety, exactly-once
delivery, transactional consistency — that nothing in the code establishes.

**Signals** — A comment or a report claiming a guarantee with no mechanism nearby. A function
named `atomicUpdate` that performs a read and then a write. "Thread-safe" on a class whose
only synchronization is a mutex around one of three mutating methods. "Idempotent" on a
handler that inserts without a uniqueness constraint.

**Cause** — Confusing the vocabulary of a property with its implementation.

**Why agents** — These words are strongly associated with the code shapes around them in
training data, and almost never verified against the mechanism.

**Risk** — Callers rely on the guarantee. The failure appears under load, in production, once.

**Trajectory** — The claim spreads: the next reader propagates it into a docstring, then into
an architecture document, and it becomes an unexaminable "known" property of the system.

**Exception** — The mechanism exists elsewhere and is genuinely reachable — a database
constraint, a framework-level transaction, an outer lock. Find it before reporting.

**False positive** — Guarantees provided by the runtime or framework: a single-threaded event
loop, an actor mailbox, a serializable isolation level.

**Fix** — Either implement the mechanism or delete the claim. Deleting is often correct: the
code may not need the guarantee.

**Proof** — Name the mechanism and the invariant it protects, at the line where it lives.

---

### Uncertainty rationalized in a comment

**Is** — A comment that converts "I don't know" into "it's fine."

**Signals** — `// should be safe`, `// assuming this is always set`, `// this shouldn't
happen`, `// TODO: verify`, `// probably fine for now`, `# NOTE: not sure why this works`.

**Cause** — The doubt was correctly felt and incorrectly resolved. The comment discharges the
discomfort without discharging the risk.

**Why agents** — Producing text is easier than reading the caller. The comment also reads as
diligence, which makes it self-reinforcing.

**Risk** — The condition the comment waves at is exactly the one that occurs.

**Trajectory** — These comments outlive everyone's memory of them. Ten years later nobody can
remove the code because nobody knows what the uncertainty was.

**Exception** — A comment that records a *resolved* investigation is valuable: `// Stripe
returns 200 with an error body here; see incident #412`. That is knowledge, not doubt.

**False positive** — `// this shouldn't happen` above a genuine defensive assertion that
crashes loudly. That is defensive programming, not rationalization — the tell is whether the
branch does something honest.

**Fix** — Resolve it by reading, or convert it to an explicit assertion that fails loudly, or
surface it as a stated assumption in the report. Never leave it as prose.

**Proof** — The assumption appears in the report's *unverified* list, or it appears nowhere
because it was resolved.

---

### Unverified completion claim

**Is** — Reporting that something works, passes, or is done without having run or read the
thing that would establish it.

**Signals** — "All tests pass" with no command output. "This is backward compatible" with no
caller enumerated. "Fully implemented" where a branch returns a placeholder. "No other
callers" from a search that could not see dynamic dispatch.

**Cause** — Confusing the *intention* to produce working code with evidence that it works.

**Why agents** — Completion language is the strongest attractor at the end of a task, and
nothing in the generation process distinguishes a run command from a remembered one.

**Risk** — The user trusts a false statement and merges on it. This failure mode destroys
more trust than any other in this catalogue.

**Trajectory** — Once a user catches one, they verify everything, and the skill's value drops
to zero.

**Exception** — None. See [`../evidence.md`](../evidence.md) for how to state partial
verification honestly.

**False positive** — The check genuinely ran and the output is simply not quoted. Ask for the
output rather than assuming it did not run.

**Fix** — Run it, or label the claim as inspected or assumed.

**Proof** — Command, exit status, and what it proves — in the report.
