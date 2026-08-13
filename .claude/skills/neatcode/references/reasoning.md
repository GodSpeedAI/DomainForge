# The reasoning sequence

Loaded for Standard and Deep work. Five steps, in order. The order matters: each step
supplies the context that makes the next one judgeable.

The sequence replaces the thing an agent does by default, which is to read the diff, notice
something familiar, and generate a comment about it. Familiarity is not analysis.

---

## 1 · Intent

**Establish what should be true before looking at what is.**

Determine:

- **Requested outcome** — in the user's words, not your paraphrase of what would be nice.
- **Acceptance criteria** — how anyone would know it is done. If the task does not state
  them, derive them and say you derived them.
- **Protected behaviour** — what must not change. Usually larger than the task admits: public
  interfaces, persisted formats, error contracts, performance characteristics, ordering.
- **Scope** — which files, modules, and surfaces are in play.
- **Constraints** — repository rules, framework demands, deployment realities, deadlines the
  user named.
- **Explicit non-goals** — what you are deliberately not doing. Writing these down is what
  keeps a task from growing.
- **Unresolved uncertainty** — what you do not know and cannot cheaply find out.

In `review`, add the question the reviewer must ask first:

> **Does this change correspond to the task?**

A patch that solves an adjacent problem elegantly is still the wrong patch, and this is the
only step where you will notice.

**Ask sparingly.** State the contract and proceed. Ask one question only when two materially
different implementations hang on the answer — not to be safe, not to seem thorough.
Uncertainty that does not change the work goes in the output as a stated assumption.

---

## 2 · Surface

**Read the shape before the meaning.** This is the direct analogue of looking at a rendered
page: you can learn a great deal from the silhouette before you read a word.

Inspect:

- **Files touched** — how many, and where. Two files in one module reads differently from
  eleven files across five directories.
- **Dispersion** — a change spread across horizontal technical layers in a repository that
  claims vertical slices is a signal before you read any logic.
- **Added and removed abstractions** — new types, interfaces, modules, packages, files.
- **Public surface** — exported symbols, route definitions, schema fields, CLI flags, event
  names, migration files.
- **Dependencies** — anything new in a manifest or lockfile. Note it now; judge it at step 3.
- **Placeholders** — `TODO`, `FIXME`, `NotImplemented`, empty catch blocks, functions
  returning a hard-coded value, commented-out code.
- **Comments** — what they claim, and whether the code supports it.
- **Tests** — present or absent, and what shape.
- **Generated artifacts** — lockfiles, schemas, compiled output, snapshots.
- **Naming and placement** — does the new file's name and location tell you what it does, and
  does it match its neighbours?
- **Size and taxonomy signals** — a new `utils.ts`, a fourth `*Manager`, a 900-line module, a
  directory of one-function files.

Write down what you saw before interpreting it. The interpretation is step 3.

---

## 3 · Structure

**Place the change in the repository's morphology.** This is where most AI-generated code
fails, and where a reviewer who skipped steps 1 and 2 has nothing to say.

Determine:

- **Canonical path** — where does this kind of behaviour already live? Find it before judging
  where the change put it. If you cannot find one, say the repository does not have one; that
  is itself a finding.
- **Authority** — who owns each invariant this touches? One invariant, one owner. If two
  places now enforce the same rule, they will diverge; the only question is when.
- **Dependency direction** — which way do the arrows point here, and which way does the
  repository say they should? An import from `domain/` into `infrastructure/` is a fact you
  can check in seconds and it decides a whole class of findings.
- **Boundaries** — does the change cross a module, package, layer, or bounded-context edge?
  What is supposed to happen at that edge — translation, validation, a port?
- **State ownership** — who may mutate this, and under what guarantee?
- **Source of truth** — for each piece of data, where does it authoritatively live?
- **Declared vs observed** — does the structure you found match the structure the repository
  claims? At Deep depth, run [`architecture/phenotype.md`](architecture/phenotype.md).
- **Earnedness** — is any new structure earned? ([`restraint.md`](restraint.md))

---

## 4 · Semantics

**Trace what the code does, not what it looks like.**

Follow, for the paths the change touches:

- **Data flow** — where each value comes from, what transforms it, where it lands. Trace at
  least one value end to end.
- **Control flow** — branches, early returns, loops, recursion, and which paths are actually
  reachable.
- **Contracts** — preconditions, postconditions, and what happens when a caller violates one.
- **Invariants** — what must remain true, and where it is enforced. An invariant enforced at
  a call site rather than at its owner is enforced nowhere.
- **State transitions** — is the set of legal transitions explicit, or emergent from the
  order of statements? Can an illegal state be represented?
- **Error paths** — what is thrown or returned, what catches it, what the caller can do with
  it, and what information survives the trip.
- **External effects** — network, disk, database, queue, clock, randomness, environment. Each
  is a place the happy path ends.
- **Concurrency** — shared mutable state, ordering assumptions, atomicity, locks, async
  boundaries, cancellation.
- **Compatibility** — old callers, old data, old clients, mixed-version deployments.
- **Security consequences** — trust boundaries crossed, authorization decisions made, inputs
  reaching a sink.

The highest-yield habit here: **pick the single most important value in the change and trace
it from its untrusted origin to its final effect.** Most real defects are found this way and
almost none are found by reading a diff top to bottom.

---

## 5 · Evidence

**Establish what is actually known.** Full protocol in [`evidence.md`](evidence.md).

- Which commands ran, and what did they return?
- Which tests exercise the changed lines?
- Could those tests have failed before the change?
- Are both success and failure behaviour covered?
- Was anything skipped, loosened, or disabled to get to green?
- Which claims remain inferred rather than verified?
- Is the feature connected end to end, or only in the layer that was touched?

Then, and only then, emit the diagnosis.

---

## Anti-sequence — how this goes wrong

Four failure patterns worth naming, because an agent falls into them by default:

**Reading the diff first.** Starting at step 2 with no step 1 produces reviews that are
fluent about what changed and silent about whether it should have.

**Pattern-matching without tracing.** Seeing `catch (e) {}` and reporting "swallowed
exception" without checking whether the caller retries, or whether the surrounding function
already logs. Half of pattern-matched findings are false positives, and false positives are
what teach a user to stop reading.

**Judging the file instead of the change.** Everything in step 3 is context. Only the change
is on trial. ([`findings.md`](findings.md) § Provenance.)

**Stopping at step 3.** Structural criticism is easy and feels expert. Semantics is where the
bugs are. A review that names an architectural smell and misses the off-by-one has failed.
