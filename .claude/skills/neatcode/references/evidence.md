# Evidence

Loaded at Step 7 of the default flow, and whenever completion is claimed.

The rule this file exists to enforce:

> **A claim about the code is only as good as the thing that supports it.**
> If nothing supports it, it is an assumption, and it gets written down as one.

## The three states of a claim

| State | Means | How to write it |
| --- | --- | --- |
| **Verified** | A command ran, you saw the result, and the result speaks to the claim. | `npm test` ✓ (204 passed) — covers the resume-from-paused path |
| **Inspected** | You read the code and traced the path, but nothing executed. Legitimate and often sufficient. | Traced `applyProration()` callers; all three pass a non-null cycle. Not executed. |
| **Assumed** | Neither. You believe it because it is plausible. | Assumed: the `stripe-node` v14 `subscriptions.update` accepts `proration_behavior`. Not verified against installed version. |

All three are acceptable in a report. Only one of them is acceptable **unlabelled**, and it
is *verified*. The failure mode is not assuming — it is assuming silently.

## What tests actually prove

The question that matters more than coverage percentage:

> **Could this test have failed before the change?**

- If you fixed a bug and the new test passes against the *old* code, the test does not cover
  the bug. Say so and fix the test.
- If you wrote the test after the implementation by reading the implementation, it asserts
  that the code does what the code does. That is a tautology with a green check mark.
- If the assertion is `expect(result).toBeDefined()` or `assert response.status == 200`, it
  will survive almost any regression. Weak assertions are worse than no test, because they
  buy false confidence.

For a bug fix, the strongest available evidence is: **the test fails on the parent commit and
passes on this one.** When you can produce that, say it explicitly — it is the single most
persuasive line in any report.

## Coverage that means something

Ask, in order:

1. **Does a test exercise the changed line at all?** If not, everything below is moot.
2. **Does it assert the behaviour, or the shape?** Behaviour: the resumed subscription is not
   charged twice. Shape: the function returns an object with four keys.
3. **Is the failure path covered?** Happy-path-only tests are the most common test failure
   in AI-generated code. What happens on a timeout, a conflict, a malformed input, an empty
   collection, a duplicate request?
4. **Does it run against production wiring?** A test that mocks the thing under test proves
   the mock works. An "integration test" whose every collaborator is a stub is a unit test
   with a misleading filename.
5. **Would it catch the regression that motivated it?** If the answer is "probably", it is
   not covered.

## Verification commands

Run what the repository considers proof, not what you would have chosen. `neatcode checks`
lists what it declares; `package.json` scripts, `Makefile` targets, `Cargo.toml`,
`pyproject.toml`, and CI configuration are where the answer lives.

Record, for each:

```text
command · exit status · what it proves · whether it actually ran
```

**"Not run" is a legitimate result.** Reasons a check might not run — no network, a missing
service, a long build, a sandbox restriction — are all fine to state. What is never fine is
reporting a pass for a command you did not execute, or inferring "tests pass" from "the code
looks right." That single behaviour destroys more trust than every other failure mode in this
skill combined.

If a check fails for reasons unrelated to the change, say that, show the failure, and say why
you believe it is unrelated. Do not silently drop it.

## Evidence for claims other than correctness

| Claim | What actually supports it |
| --- | --- |
| "This is faster" | A benchmark, before and after, with the input size stated. Not reasoning about big-O. |
| "This is thread-safe" | The lock or ownership discipline named, and the invariant it protects. Not "I used a mutex." |
| "This is backward compatible" | The old callers enumerated, or a contract test against the previous version. Not "I kept the parameter." |
| "The migration is safe" | The rollback path described and the mixed-version window considered — old code reading new data, new code reading old data. |
| "This is secure" | The trust boundary named, the input path traced from its untrusted origin, the check located *before* the unsafe use. |
| "It's wired up" | The registration, route, export, or container binding pointed at by path and line. |
| "Nothing else uses this" | The search you ran, including its limits — a grep for a symbol misses dynamic dispatch, reflection, string-built names, and other repositories. |

That last row deserves particular care. "No other callers" is one of the most confidently
wrong statements an agent makes. State the search you ran and what it could not see.

## Evidence in a review

You are judging someone else's evidence. Check:

- Were tests added? If not, could the defect recur undetected?
- Do the added tests fail without the change? If you can cheaply check by reverting the
  source hunk in a scratch copy, do.
- Are the assertions specific enough to catch a regression?
- Does the change claim anything — in its description, its comments, or its commit message —
  that nothing in the diff supports?
- Was anything disabled to make the suite pass? A skipped test, a loosened assertion, a
  widened type, a raised timeout, a `# noqa`. These are S2 findings when they are how the
  green check was obtained.

That last check finds real problems more often than any other item in this file.

## The honest close

Every completion block ends with what remains unproven. Not as hedging — as a handover.

```text
evidence: npm test ✓ (204 passed) · npm run typecheck ✓ · npm run lint — not run (no network)
unverified: annual-plan proration path has no test; behaviour inferred from the shared
            code path, not executed.
```

A report that claims less and proves it is worth more than one that claims everything.
