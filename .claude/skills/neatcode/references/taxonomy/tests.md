# Test failures

Failures of proof. Tests exist, run, and pass, and establish very little. The pattern to watch
for: tests written *after* the implementation, *from* the implementation, which can only ever
confirm that the code does what the code does.

The question that resolves most entries here: **could this test have failed before the change?**

---

### Test mirrors the implementation

**Is** — A test that asserts how the code works rather than what it must do.

**Signals** — Assertions on the number of calls to an internal collaborator. Mocks verifying
private method invocation. A test that must change whenever the implementation is refactored
without behaviour changing. Assertions on intermediate structures nobody outside consumes.
A test that reads like a transcription of the function body.

**Cause** — Writing the test by reading the implementation.

**Why agents** — The implementation is in context; the requirement often is not. Generating an
assertion from visible code is trivial.

**Risk** — The test blocks refactoring while catching no regressions — the worst possible
trade.

**Trajectory** — The suite becomes an obstacle. Teams start deleting tests to ship, and lose
the good ones alongside the bad.

**Exception** — Genuine contract tests on a public interface — including one asserting a
collaborator is called when the *call itself* is the observable behaviour (an audit log, a
published event).

**False positive** — Interaction testing is correct when the interaction is the outcome.

**Fix** — Assert observable behaviour: return values, state after, effects at the boundary.

**Proof** — Refactor the implementation without changing behaviour; the test still passes.

---

### Happy path only

**Is** — Tests covering success and nothing else.

**Signals** — Every test supplies valid input. No test for empty, missing, malformed,
duplicate, out-of-range, unauthorized, timed-out, or conflicting. No test of the error branch
the change added.

**Cause** — Testing the example from the task description.

**Why agents** — The success case is the one the implementation was written for and the one
the prompt described.

**Risk** — Every error path in the change is unverified — and error paths are where the
severe bugs are.

**Trajectory** — Error handling rots invisibly, since nothing exercises it.

**Exception** — A pure function with a total domain, where invalid input cannot be
constructed.

**False positive** — Failure cases covered in a separate file or suite. Look before reporting.

**Fix** — One test per named failure mode. Start with the ones the code explicitly handles: if
there is a `catch`, there should be a test that reaches it.

**Proof** — Every error branch in the change has a test that enters it.

---

### Assertion too weak to catch a regression

**Is** — An assertion that would pass through the bug it exists to prevent.

**Signals** — `expect(result).toBeDefined()`. `assert response.status_code == 200` with no body
check. `assert len(items) > 0`. `expect(fn).not.toThrow()`. A snapshot asserting a structure
that does not include the changed field. `assert result is not None`.

**Cause** — Producing an assertion rather than deciding what must be true.

**Why agents** — Weak assertions always pass, which makes them locally successful.

**Risk** — Green CI through a real regression.

**Trajectory** — Coverage numbers rise while defect escape stays flat, and the metric becomes
actively misleading.

**Exception** — A smoke test whose only job is "the thing starts."

**False positive** — The specific assertion lives in another test in the same file.

**Fix** — Assert the value that matters — the computed number, the exact state, the specific
error code.

**Proof** — Mutate the implementation; the test must fail.

---

### Excessive mocking

**Is** — So much replaced that the test exercises the mocks.

**Signals** — Every dependency mocked, including the one under test. A mock returning a value
the test then asserts. Mocks of the language runtime or the standard library. More setup than
assertion. A test that passes when the real implementation is broken.

**Cause** — Isolating for its own sake instead of for a reason — slowness, nondeterminism, an
external effect.

**Why agents** — Mock setup is highly patterned and produces confident-looking test code.

**Risk** — Tests validate the test's own model of the system, and drift from reality silently.

**Trajectory** — Integration bugs land in production while the unit suite is green.

**Exception** — Mocking genuinely external, slow, nondeterministic, or effectful
collaborators: network, payment providers, email, the clock, randomness.

**False positive** — A test double that is a real in-memory implementation, not a stub. That is
usually better than mocking.

**Fix** — Mock at the boundary only. Use real collaborators inside. Prefer fakes over mocks.

**Proof** — Breaking a real collaborator makes some test fail.

---

### Snapshot substituting for verification

**Is** — Recording current output instead of asserting required output.

**Signals** — Snapshot files regenerated whenever they fail. Large snapshots nobody reads.
Approval tests as the only coverage for logic. A snapshot containing a value that is wrong,
committed because the test "passed."

**Cause** — Snapshots make a test pass without deciding what correct is.

**Why agents** — `toMatchSnapshot()` is one assertion for any output shape.

**Risk** — A bug is recorded as expected on the first run, and the snapshot then *defends* it.

**Trajectory** — Snapshot updates become reflexive; the suite records history rather than
requirements.

**Exception** — Genuinely appropriate for large stable outputs — rendered markup, generated
code, formatter output — alongside targeted behavioural assertions.

**False positive** — The snapshot is small, reviewed, and the behaviour is asserted elsewhere.

**Fix** — Assert the specific properties that must hold. Keep snapshots for shape, not for
correctness.

**Proof** — A deliberate defect fails a named assertion, not just a snapshot diff.

---

### Test does not exercise production wiring

**Is** — Verification that never touches how the code actually runs.

**Signals** — Handlers called directly rather than through the router. A job function invoked
directly rather than through the scheduler. Configuration hand-constructed rather than loaded.
A DI graph bypassed. Middleware skipped.

**Cause** — Testing the unit because the unit is the thing that was written.

**Why agents** — Wiring is repository-specific; the unit is not.

**Risk** — Everything passes while the feature is unreachable. See
[`completion.md`](completion.md) § Missing registration.

**Trajectory** — The team learns that green CI does not mean working software.

**Exception** — A test pyramid where wiring is covered by a separate suite that actually runs.

**False positive** — An end-to-end suite exists elsewhere.

**Fix** — At least one test through the real entry point.

**Proof** — Deliberately break the wiring; a test must go red.

---

### Fake integration test

**Is** — A test named `integration` that integrates nothing.

**Signals** — An "integration" test with every collaborator stubbed. A "database test" against
an in-memory substitute with different semantics — different transactions, different
constraints, different SQL dialect. An "API test" that calls the handler function.

**Cause** — Naming by intent rather than by content.

**Why agents** — The filename is generated from the task description.

**Risk** — False confidence in the exact place where confidence matters most.

**Trajectory** — Real integration failures escape to production while the suite reports
coverage of them.

**Exception** — An in-memory substitute with genuinely equivalent semantics, verified as such.

**False positive** — A real container-backed suite you did not identify.

**Fix** — Integrate against the real thing — a container, a test instance — or rename the test
honestly.

**Proof** — The test fails when the real dependency is misconfigured.

---

### Tests added only to ratify the implementation

**Is** — Tests written after the fact, from the code, to satisfy a coverage expectation.

**Signals** — Test names describing methods rather than behaviours (`test_process_data`).
Every test asserting exactly what the code returns today. A test suite created in the same
commit that would pass against any implementation of the same shape. Coverage exactly at the
threshold.

**Cause** — Treating tests as a deliverable rather than as evidence.

**Why agents** — A test file is expected output; whether it can fail is not checked.

**Risk** — The appearance of verification without the substance, which is worse than no tests
because it stops anyone from adding real ones.

**Trajectory** — The suite grows, the defect rate does not change, and confidence in testing
itself erodes.

**Exception** — Characterization tests written deliberately to pin existing behaviour before a
restructure. Same shape, opposite purpose — and they should say so in a comment.

**False positive** — Good tests written after the implementation. Post-hoc is not
automatically bad; derived-from-the-implementation is.

**Fix** — Write the test from the requirement. For a bug fix, confirm it fails on the parent
commit.

**Proof** — The test fails against the pre-change code. Say so in the report.
