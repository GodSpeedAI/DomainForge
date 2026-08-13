# Contract failures

Failures of agreement. Something other code relies on changed, and nothing said so. These are
the findings that survive review and surface as incidents.

---

### Accidental behaviour change

**Is** — Behaviour the task did not ask to change, changed anyway.

**Signals** — A refactor that also alters an ordering, a default, a rounding mode, a
null-handling rule, a truthiness check, an inclusive bound. A "cleanup" that removes a
seemingly redundant condition. A loop rewritten as a map that no longer short-circuits. A
`==` become `===`.

**Cause** — Rewriting from intent rather than transforming from the original. The rewritten
version expresses what the code *appeared* to do.

**Why agents** — Regenerating a function is easier than editing it, and the regenerated
version is the canonical form rather than this one.

**Risk** — The removed condition was load-bearing for a case nobody remembers.

**Trajectory** — The behaviour is restored later as a special case with a confusing comment,
and the cycle repeats.

**Exception** — The change was requested, or the old behaviour was the bug.

**False positive** — The behaviours are genuinely equivalent. Verify the edges before
reporting: empty input, single element, duplicate keys, `null` vs `undefined`, NaN, negative
zero, unicode.

**Fix** — Restore the original behaviour, or state the change explicitly and get it approved.

**Proof** — A test that captures the original behaviour, passing both before and after — or
failing before, if the change was intended.

---

### Dropped invariant

**Is** — A property the previous code maintained is no longer maintained.

**Signals** — A validation removed during a refactor. A sort whose stability mattered. A
uniqueness check dropped when the code moved. A clamp, a bound, a normalization step gone. A
sequence of operations reordered where the order was the guarantee.

**Cause** — The invariant was implicit — enforced by the *shape* of the old code rather than
stated anywhere.

**Why agents** — Invariants that live in structure rather than in an assertion are invisible
to a reader who did not write them, and there is nothing in the code that says "this line is
load-bearing."

**Risk** — Illegal states become representable. The failure appears somewhere else entirely.

**Trajectory** — Downstream code grows defensive checks for states that should be impossible,
and the real invariant is never restored.

**Exception** — The invariant genuinely moved somewhere better. Find where.

**False positive** — The invariant is now enforced by a type, a database constraint, or a
schema. Look there before reporting.

**Fix** — Restore it at its owner and make it explicit — a type, an assertion, a constraint,
or a single guarded constructor.

**Proof** — A test that constructs the illegal state and expects rejection.

---

### Edge case ignored

**Is** — A reachable input or state the code does not handle and does not acknowledge.

**Signals** — No handling for: empty collection, single element, zero, negative, maximum,
overflow, duplicate, `null`/`None`/`nil`, absent optional, malformed input, unicode and
combining characters, timezone and DST boundaries, leap seconds, concurrent arrival, partial
failure, an empty string that is not the same as absent.

**Cause** — Writing for the example in the task description.

**Why agents** — Training data is dominated by illustrative code, which handles the
illustrative case.

**Risk** — Depends entirely on the domain. In a money or auth path this is S1.

**Trajectory** — Special cases accumulate as patches at call sites rather than at the owner.

**Exception** — The case is genuinely unreachable, guaranteed by a type or an upstream
validation you can point at. Point at it.

**False positive** — Handled by the language or framework — a nullable type that will not
compile, a schema validator at the boundary, a database constraint.

**Fix** — Handle it, or make it unrepresentable, or document why it cannot occur. All three
are acceptable; silence is not.

**Proof** — A test per handled edge, or a type that makes the case impossible.

---

### Public interface changed without justification

**Is** — An externally visible contract altered without a compatibility story.

**Signals** — A changed exported signature, a removed field, a renamed route, an altered
response shape, a new required parameter, a changed status code, a modified event payload, a
narrowed accepted input, a changed default.

**Cause** — Treating "public" as a language keyword rather than as a promise to someone.

**Why agents** — The consumers are not in context. Nothing about the code says who is
depending on it.

**Risk** — Downstream breakage at a distance, including in systems you cannot see or deploy.

**Trajectory** — Ad-hoc compatibility shims accumulate at every consumer.

**Exception** — Pre-1.0, an explicitly unstable surface, or an authorized breaking change
with a migration note.

**False positive** — The surface is internal despite being exported by the language. Check
whether it is actually consumed.

**Fix** — Additive change, deprecation with a window, or an explicitly versioned break with a
migration path.

**Proof** — Contract or golden tests against the previous shape; the enumerated consumers.

---

### Compatibility assumed

**Is** — Believing old callers, old data, or old clients still work, without checking.

**Signals** — A serialization format changed with no migration. A new required field on a
persisted structure. A protocol change with no version negotiation. A schema change deployed
in one step. A widened enum consumed by an exhaustive match elsewhere.

**Cause** — Reasoning only about the new code path, at one instant in time.

**Why agents** — Deployment is invisible in a diff. Mixed-version windows have no
representation in source.

**Risk** — Failure during the deploy window, which is precisely when the system is least
observable.

**Trajectory** — Teams learn to fear deployments and stop doing them frequently, which makes
each one riskier.

**Exception** — A single-deployment-unit system with no persisted state and no external
clients. Rare; verify rather than assume.

**False positive** — The framework handles it — a schema-versioned serializer, a migration
tool that runs in the right order.

**Fix** — Expand-then-contract: add the new form, write both, migrate readers, then remove
the old. Say which phase this change is.

**Proof** — A test that reads old-format data with new code, and vice versa.

---

### Error semantics altered silently

**Is** — How failure is communicated changed, without the callers being updated or told.

**Signals** — A thrown exception became a returned `null`. A specific error type widened to a
generic one. A failure became a default value. An error message that callers parse changed
shape. A `Result::Err` became a logged warning and a success. An HTTP 409 became a 400.

**Cause** — Treating errors as an afterthought rather than as part of the interface.

**Why agents** — The happy path is the task; error handling is boilerplate to be produced in
the most common shape.

**Risk** — Callers that branched on the specific failure now take the wrong branch — often
the *success* branch.

**Trajectory** — Errors lose their semantics one refactor at a time until every failure is
"something went wrong" and nothing can be handled programmatically.

**Exception** — A deliberate, documented error-model migration.

**False positive** — A wrapper preserves the original as a cause and callers inspect the
chain. Check for it.

**Fix** — Preserve the discriminable failure. If the model must change, update every caller
that branches on it in the same change.

**Proof** — Tests asserting the specific error type or code, not merely that something failed.
