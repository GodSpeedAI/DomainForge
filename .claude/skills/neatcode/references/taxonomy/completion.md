# Completion failures

Failures of finishing. The change has the shape of a finished feature and does not work end
to end. This family is why "done" is a claim requiring evidence rather than a feeling.

---

### Stub or placeholder left behind

**Is** — A code path that returns a hard-coded value, does nothing, or raises "not
implemented," inside work presented as complete.

**Signals** — `throw new Error('Not implemented')`, `pass`, `return []`, `return true`,
`todo!()`, an empty method body, a branch whose body is a comment, a function whose entire
body is a `console.log`.

**Cause** — Producing the full shape of a solution, then filling in the parts the model was
confident about.

**Why agents** — The structure of a complete implementation is far more predictable than its
hard parts, so the structure gets generated and the hard parts get deferred.

**Risk** — The path is exercised in production and returns a lie.

**Trajectory** — Stubs are the hardest debt to find later because they look like working code
and pass type checks.

**Exception** — Explicitly staged work the user asked for, with the incomplete part named in
the output.

**False positive** — An intentional no-op with a stated reason (a null object, a default
strategy, a disabled-by-design hook).

**Fix** — Implement it, or remove the path, or state clearly and prominently which part is
unfinished. Never all three quietly.

**Proof** — No unexplained placeholder remains; the report names any that do.

---

### TODO debt

**Is** — Deferred work recorded in a comment instead of in the task, the issue tracker, or
the report.

**Signals** — `TODO`, `FIXME`, `HACK`, `XXX`, `// handle this later`, `// needs error
handling` — introduced by this change.

**Cause** — Discharging an obligation by naming it.

**Why agents** — A TODO reads as diligence and costs nothing to produce.

**Risk** — Low individually. The risk is the aggregate: nobody reads them.

**Trajectory** — A repository with two hundred TODOs has zero, because the signal is gone.

**Exception** — A TODO that references a tracked issue with a date or a ticket, added by
policy. Some repositories require exactly this; check the conventions.

**False positive** — Pre-existing TODOs in a touched file. Not this change's fault.

**Fix** — Do it, or surface it in the report as named debt, or file it where the team tracks
work.

**Proof** — The completion block lists the deferred item explicitly.

---

### Missing registration or wiring

**Is** — Code that exists and is never reached.

**Signals** — A handler never added to the route table. A provider never registered in the
container. A migration file never added to the manifest. A new module never exported from
its index. A CLI subcommand never attached. An event listener never subscribed. A plugin
never listed. A React component never rendered.

**Cause** — Implementing the unit and not the connection. The unit is the interesting part.

**Why agents** — Wiring is repository-specific, unglamorous, and lives in a file the agent
never opened.

**Risk** — The feature does not exist, while every test of the unit passes.

**Trajectory** — Discovered by a user rather than by CI, which is the most expensive way.

**Exception** — Framework auto-discovery by file convention. Verify it actually applies here
rather than assuming.

**False positive** — Registration happens in a generated or scanned location.

**Fix** — Wire it. Point at the registration by path and line.

**Proof** — A test that exercises the feature through its real entry point — the route, the
CLI, the queue — not through a direct call to the unit.

---

### Configuration declared but unused

**Is** — A setting, flag, or environment variable that is defined, documented, and never read.

**Signals** — A new key in a config schema with no consumer. An env var in `.env.example`
with no `process.env` read. A feature flag checked nowhere. A constructor option ignored in
the body. A CLI flag parsed and dropped.

**Cause** — Building the interface of configurability without the behaviour.

**Why agents** — Configuration is a strong pattern-completion attractor; a config block is
what "professional" code looks like.

**Risk** — Operators set it and nothing happens. Trust in the whole config surface drops.

**Trajectory** — Nobody dares remove it, because nobody can prove it is unused.

**Exception** — Reserved for a documented, imminent rollout.

**False positive** — Read dynamically by key, by a framework, or by an external system. Search
for the string, not just the identifier.

**Fix** — Wire it or delete it.

**Proof** — A test where the setting changes observable behaviour.

---

### Migration without rollback or compatibility thinking

**Is** — A schema or data change with no reverse path and no mixed-version story.

**Signals** — A destructive migration (drop column, drop table, narrow a type) with no
reverse. A `NOT NULL` added without a backfill. A rename in one step. An index built without
`CONCURRENTLY` on a live table. A migration coupled to a code change that must deploy
simultaneously.

**Cause** — Reasoning about the final state instead of the transition.

**Why agents** — The migration file expresses the destination; the deploy window is invisible
in source.

**Risk** — Downtime, data loss, or an unrecoverable partial deploy.

**Trajectory** — Recovery requires a manual, high-pressure intervention at the worst time.

**Exception** — A single-node system with a maintenance window, stated as such.

**False positive** — The framework generates reversible migrations and this one is genuinely
covered.

**Fix** — Expand-then-contract: add nullable, backfill, dual-write, switch reads, then remove.
Say which phase this change is and what the next one is.

**Proof** — The rollback exists and has been exercised, at least on a copy. The mixed-version
window is described.

---

### Tests pass while the feature is disconnected

**Is** — Green CI on code that does not work in the running system.

**Signals** — Every collaborator mocked, including the boundary under test. A test importing
the unit directly while nothing imports it in production. Fixtures constructed by hand where
production builds them differently. An "integration" test with no real integration.

**Cause** — Testing the unit rather than the behaviour, in a system where the unit was never
attached.

**Why agents** — A test file next to a source file is a very strong pattern. What that test
should *exercise* is a judgment call.

**Risk** — CI reports success on a broken feature — the most dangerous possible false signal.

**Trajectory** — The team calibrates on CI, and CI is lying.

**Exception** — A genuinely pure unit whose wiring is covered elsewhere. Point at the elsewhere.

**False positive** — A test pyramid where integration coverage lives in a separate suite.
Check whether that suite covers this path.

**Fix** — Add at least one test that reaches the feature through its real entry point.

**Proof** — Break the wiring deliberately and confirm a test goes red. If none does, the
coverage is theatre.
