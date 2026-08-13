# Change-discipline failures

Failures of restraint in the diff itself. The code may be fine; the *change* is not. Load this
family on every `review`.

The underlying rule: **a change should be the smallest coherent unit that accomplishes its
stated purpose, and nothing else.** Everything here is a violation of that, and every one of
them makes the change harder to review, harder to revert, and harder to bisect.

---

### Unrelated refactor

**Is** — Improvements bundled into a change that did not need them.

**Signals** — A bug fix that also renames variables across the file. A feature that also
reorganizes imports, extracts helpers, or converts callbacks to promises. Files touched that
have nothing to do with the stated task.

**Cause** — Fixing what is in front of you while you are there. A genuinely good instinct,
applied at the wrong granularity.

**Why agents** — Regenerating a region is easier than editing it precisely, so surrounding
code gets rewritten as a side effect of touching one line.

**Risk** — The real change hides among the noise, and a reviewer approving 300 lines reviews
none of them.

**Trajectory** — Reverting the fix reverts the refactor and vice versa; `git bisect` and
`git blame` both degrade.

**Exception** — A refactor genuinely required to make the fix possible — say so explicitly and
keep it minimal.

**False positive** — Automated formatting the repository applies on save, consistently.

**Fix** — Separate commits or separate pull requests. Say which is which.

**Proof** — The diff contains only what the stated purpose requires.

---

### Oversized diff

**Is** — A change too large to review meaningfully.

**Signals** — Hundreds of changed lines with no natural seams. Many unrelated concerns in one
change. A "phase 1" that implements phases 1 through 3. A rewrite presented as a fix.

**Cause** — Not decomposing.

**Why agents** — Generation is cheap; deciding where to stop is not, and a complete-looking
implementation is a strong attractor.

**Risk** — Review becomes rubber-stamping. Defects pass because attention does not scale.

**Trajectory** — Large changes normalize; review quality falls across the project.

**Exception** — Genuinely atomic changes — a mechanical rename that must be complete to
compile, a generated file, a vendored dependency. Say which, and keep them alone in their
change.

**False positive** — Most lines are generated or vendored. Report the reviewable subset.

**Fix** — Split by seam: interface first, then implementation, then callers. Behaviour-
preserving refactor first, behaviour change second.

**Proof** — Each part builds and passes tests on its own.

---

### Broad reformatting

**Is** — Whitespace, quote, or import churn mixed into a substantive change.

**Signals** — Every line of a file changed with no semantic difference. Line-ending changes.
A different formatter's output. Import reordering across many files.

**Cause** — A different local tool configuration, or a formatter run over a whole file.

**Why agents** — Regenerated code carries the model's default formatting rather than the file's.

**Risk** — The substantive change is invisible; `git blame` is destroyed for the whole file.

**Trajectory** — File history becomes useless, which removes the main tool for understanding
why code is the way it is.

**Exception** — A deliberate, isolated formatting change, alone in its commit.

**False positive** — The repository's own formatter ran as a pre-commit hook.

**Fix** — Revert the formatting noise. Match the file's existing style. Run the repository's
formatter, not yours.

**Proof** — The diff shows only semantic changes.

---

### Gratuitous rename

**Is** — Renaming things the task did not require renaming.

**Signals** — A variable renamed to your preferred convention. A file moved for tidiness. A
function renamed "for clarity" with all call sites updated. A parameter renamed in a public
signature.

**Cause** — Preference expressed as improvement.

**Why agents** — Generated code uses the model's naming conventions rather than the file's.

**Risk** — Diff noise, merge conflicts for anyone else in the file, and — if public — a
breaking change disguised as a cleanup.

**Trajectory** — Naming churn without convergence, since the next agent has different
preferences.

**Exception** — The old name is actively wrong or misleading. Then it is its own change, with
a reason.

**False positive** — The rename is required — a genuine conflict, a corrected typo in a
misleading name.

**Fix** — Keep the existing names. Propose renames separately.

**Proof** — The diff renames nothing the purpose did not require.

---

### Hidden dependency change

**Is** — Adding, upgrading, or removing a dependency as a side effect of other work.

**Signals** — A lockfile change in a bug-fix diff. A new package used once for something the
standard library covers. A transitive major version bump. A dependency removed because "it
seemed unused."

**Cause** — Installing to solve a local problem without weighing the cost.

**Why agents** — Reaching for a package is the most common solution shape in training data,
and the cost of a dependency is invisible in a diff.

**Risk** — Supply-chain surface, licence exposure, bundle size, transitive breakage, and a
maintenance obligation nobody agreed to.

**Trajectory** — Dependency count grows monotonically; upgrades become impossible.

**Exception** — The dependency is the point of the change, stated up front.

**False positive** — A lockfile refresh from an unrelated install; still worth flagging so it
can be reverted.

**Fix** — Separate the dependency change, justify it, and check whether the repository already
has a way to do this. Most small utility dependencies are not earned — see
[`../restraint.md`](../restraint.md).

**Proof** — The dependency change is its own commit with a stated reason.

---

### Generated-file noise

**Is** — Build output, lockfiles, or generated code entering the diff unintentionally.

**Signals** — Compiled assets committed. A snapshot regenerated wholesale. A schema
regenerated with unrelated drift. A lockfile changed by a different package-manager version.

**Cause** — Committing everything the working tree contains.

**Why agents** — `git add -A` is the reflex.

**Risk** — Enormous diffs, merge conflicts, and real changes hidden in generated churn.

**Trajectory** — Reviewers learn to skip whole file types, including when it matters.

**Exception** — Repositories that intentionally commit generated artifacts — then they must be
regenerated deterministically, from the same tool version.

**False positive** — Regeneration is a required part of the change.

**Fix** — Regenerate deliberately with the pinned tool version, or exclude and add to
`.gitignore`.

**Proof** — Regenerating produces no further diff.

---

### Scope expansion

**Is** — Doing more than was asked.

**Signals** — Extra features "while we're here." Additional configuration nobody requested. A
new abstraction to support a hypothetical case. Fixing an adjacent bug without saying so.

**Cause** — Helpfulness without a boundary.

**Why agents** — Completing a pattern feels like completing the task, and more output reads as
more value.

**Risk** — Unrequested behaviour, unreviewed decisions, and a change whose purpose can no
longer be stated in one sentence.

**Trajectory** — Requirements become undiscoverable, because the code contains things nobody
asked for.

**Exception** — Genuinely trivial and necessary — a one-line import fix required to compile.

**False positive** — The user asked for it earlier in the conversation.

**Fix** — Implement the request. Report the rest as findings and let the user choose.

**Proof** — Every element of the diff traces to the stated request.

---

### Behaviour removed without authorization

**Is** — Deleting functionality that was not agreed to be deleted.

**Signals** — A branch removed because it "looked dead." A parameter dropped. A validation
deleted because it "seemed redundant." An endpoint removed. A test deleted because it failed.

**Cause** — Confusing "I cannot see why this exists" with "this does not need to exist."

**Why agents** — Code whose purpose is not locally visible looks like noise, and simplification
is a rewarded-looking action.

**Risk** — Silent capability loss. The deleted branch was the one that handled the customer
nobody remembers.

**Trajectory** — Discovered months later with no record of the deletion's reasoning.

**Exception** — Provably dead code with a search demonstrating it, and authorization.

**False positive** — Genuinely dead code, correctly identified — say how you established it,
including what your search could not see.

**Fix** — Do not delete. Report it as a finding with evidence and let the user decide.

**Proof** — The deletion is explicitly authorized, or it did not happen.

**A deleted failing test is the sharpest case here.** It is never a fix, and it is an S2
finding at minimum — see [`../evidence.md`](../evidence.md).
