# Restraint — the earnedness test

Loaded on every run. This is the shortest file in the skill and the one that changes the most
output.

## The question

> **What concrete constraint earns this complexity?**

A *concrete constraint* is a fact about the world you can point at:

- a second implementation that exists **today**, in this repository
- a published interface other code already depends on
- a measured performance budget the direct version misses
- a boundary a security or compliance requirement puts there
- a variation the product has already shipped twice
- a test that cannot be written any other way
- an explicit instruction in `AGENTS.md`, an ADR, or the user's request

None of these are concrete constraints:

- "we might need to swap this out later"
- "this makes it more testable" (when the direct version is already testable)
- "it follows the pattern" (which pattern, earning what?)
- "it separates concerns" (which concerns, and what breaks if they touch?)
- "it's more extensible" (extensible along which axis, demanded by whom?)
- "this is the clean way to do it"

If the only available answer comes from the second list, **write the direct version.**

## What the test applies to

Abstractions · interfaces · base classes · layers · modules · packages · dependencies ·
indirection · factories · builders · registries · dependency-injection wiring ·
configuration options · feature flags · compatibility shims · caching · retries · queues ·
extension points · plugin hooks · concurrency machinery · generated artifacts · new files.

New files count. A change that adds six one-function files where the repository keeps
cohesive operations together has spent structure it did not earn.

## The complexity budget

Structure is paid for by every future reader. Spend it where the problem is genuinely hard,
not where it is merely repetitive.

| Problem shape | Proportionate response |
| --- | --- |
| One behaviour, one caller, no variation | A function |
| One behaviour, many callers, no variation | A function, exported |
| Two real implementations today | An interface with two implementations |
| One implementation, one *planned* | One implementation. Add the interface when the second arrives — it will be a smaller change than you fear, and it will have a real second case to be shaped by. |
| Repeated three-line sequence | Often nothing. Duplication is cheaper than the wrong abstraction. |
| Cross-cutting behaviour with a real policy | A wrapper — if it *has* the policy. A wrapper that only forwards is not a wrapper. |

**Under-engineering is also a failure.** Restraint is not minimalism. A safety-critical path
with redundant validation, a boundary adapter with explicit timeout and retry policy, a
concurrency-sensitive operation with a real lock discipline — these are proportionate, and
stripping them is the same error in the other direction. The test is *earned*, not *small*.

## Recognizing unearned structure

Signals, each of which is a hypothesis rather than a verdict:

- An interface, trait, protocol, or abstract base with exactly one implementation, and no
  test double that needed it.
- A class whose every method forwards to one field with no added policy, translation,
  validation, or lifecycle.
- A `*Manager`, `*Service`, `*Handler`, `*Processor`, `*Helper`, `*Util`, or `*Provider`
  whose responsibility cannot be stated in one sentence without the word "and".
- A factory that constructs exactly one type.
- A registry with a fixed, compile-time-known set of entries.
- A configuration option no code path reads, or that only ever holds its default.
- A layer that appears in every call stack and transforms nothing.
- A generic `shared/`, `common/`, `core/`, or `utils/` module that has become where behaviour
  goes when nobody decided where it belongs.
- An event, callback, or hook with exactly one subscriber that is invoked synchronously and
  immediately — a function call wearing a costume.

## The removal test

Before defending a piece of structure, run it in your head:

> Delete it and inline its contents at every call site. Does anything get *worse*?

If the answer is "the code gets shorter, the call sites get clearer, and nothing else
changes," the structure was not earned. Say so plainly and remove it — or, in `review` and
`audit`, report it with the inlined version as the correction.

If the answer names something concrete that gets worse — a boundary is crossed, a test
becomes impossible, a published contract breaks, an invariant loses its owner — the structure
is earned. Record *why* in one line, ideally in `engineering.md`, so the next reader does not
re-litigate it.

## Legitimate exceptions

Do not report unearned structure when any of these hold — check before writing the finding:

- The repository's stated conventions require the shape (`AGENTS.md`, an ADR, a framework's
  contract). Consistency with a mediocre convention beats a lone island of better taste.
- The framework demands it: a DI container needs registrable types, a serialization library
  needs a DTO, a test framework needs a fixture class.
- The abstraction is a **published** interface with external consumers, including other teams
  or downstream repositories. A single implementation is fine when the *contract* is the
  product.
- A second implementation is present in the same change, or in an adjacent open branch the
  user has named.
- The wrapper exists to isolate an external vocabulary at a boundary — that is policy, even
  when the method bodies look like forwarding.
- It is a safety-critical path where redundancy is the point.

When an exception applies, do not silently skip the finding: in `audit` and `study`, record
the earned structure and its constraint. That is how the next run stops re-asking.
