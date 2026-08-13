# Failure taxonomy — index

Fourteen families of named failure modes. **Read this index, then load only the family files
the change actually implicates.** Loading all fourteen is the largest avoidable token cost in
this skill; a typical run needs two to four.

Each family file gives, per failure mode: a stable name, a definition, observable signals,
the underlying reasoning failure, why AI agents produce it, the immediate risk, the debt
trajectory, legitimate exceptions, likely false positives, a correction strategy, and how to
verify the correction.

**These are heuristics, not lint rules.** Every entry has an exceptions section, and the
exceptions are load-bearing. A finding that ignores its own exception list is a false
positive with a citation.

| Family | Load when | File |
| --- | --- | --- |
| **Epistemic** — coding through uncertainty, invented APIs, fabricated guarantees | Always, for generated code. The cheapest family to check and the most expensive to miss. | [`taxonomy/epistemic.md`](taxonomy/epistemic.md) |
| **Context** — locally plausible, globally wrong; duplicate implementations; bypassed canonical paths | The change adds behaviour to an existing system | [`taxonomy/context.md`](taxonomy/context.md) |
| **Contract** — behaviour changed by accident, invariants dropped, error semantics altered | The change touches existing behaviour or a public surface | [`taxonomy/contract.md`](taxonomy/contract.md) |
| **Completion** — stubs, placeholders, missing wiring, config declared but unread | Any feature work; any change claimed to be finished | [`taxonomy/completion.md`](taxonomy/completion.md) |
| **Abstraction** — premature interfaces, pass-throughs, speculative generality, architecture cosplay | New types, modules, layers, or files appear | [`taxonomy/abstraction.md`](taxonomy/abstraction.md) |
| **Authority** — duplicated validation, transitions, or normalization; multiple owners for one invariant | The change validates, normalizes, or transitions state | [`taxonomy/authority.md`](taxonomy/authority.md) |
| **Boundary** — infrastructure in domain logic, framework types crossing layers, nominal ports | The change crosses a module or layer boundary | [`taxonomy/boundary.md`](taxonomy/boundary.md) |
| **State and concurrency** — races, non-atomic operations, missing idempotency, cancellation leaks | Shared state, async work, retries, caches, or transactions | [`taxonomy/state-and-concurrency.md`](taxonomy/state-and-concurrency.md) |
| **Failure handling** — swallowed exceptions, broad catches, silent fallbacks, misleading recovery | Any `try`, `catch`, `rescue`, `recover`, `except`, or `Result` handling | [`taxonomy/failure-handling.md`](taxonomy/failure-handling.md) |
| **Tests** — implementation mirroring, happy-path-only, weak assertions, fake integration | Tests are added or changed, or should have been | [`taxonomy/tests.md`](taxonomy/tests.md) |
| **Observability** — meaningless logs, missing correlation, silent degradation | Long-running services, background work, or production-facing failure paths | [`taxonomy/observability.md`](taxonomy/observability.md) |
| **Security** — missing authorization, insecure defaults, injection, trust-boundary confusion | Any input crossing a trust boundary, any auth path, any secret, any query construction | [`taxonomy/security.md`](taxonomy/security.md) |
| **Change discipline** — oversized diffs, unrelated refactors, hidden dependency upgrades | Always, in `review` | [`taxonomy/change-discipline.md`](taxonomy/change-discipline.md) |
| **Maintainability theater** — ceremony, taxonomic sprawl, false configurability, documentation that restates code | Naming, file layout, comments, or docs are part of the change | [`taxonomy/maintainability-theater.md`](taxonomy/maintainability-theater.md) |

## Fast routing

| What you are looking at | Load these |
| --- | --- |
| An agent's implementation of a new feature | epistemic · context · abstraction · completion · tests |
| A bug fix | epistemic · contract · tests · failure-handling |
| A refactor or "cleanup" PR | change-discipline · abstraction · authority · contract |
| Anything touching auth, input, or secrets | security · boundary · failure-handling |
| Anything async, queued, retried, or cached | state-and-concurrency · failure-handling · observability |
| A repository audit | abstraction · authority · boundary · maintainability-theater · tests |
| A migration or schema change | contract · completion · state-and-concurrency |

## The two questions underneath all fourteen

Every entry in every family is a specialization of one of these:

1. **Earnedness** — what concrete constraint earns this complexity?
   ([`restraint.md`](restraint.md))
2. **Evidence** — what supports the claim that this is correct and complete?
   ([`evidence.md`](evidence.md))

If a candidate finding cannot be traced back to one of them, it is probably a preference.
Check [`findings.md`](findings.md) § *What is not a finding* before writing it down.
