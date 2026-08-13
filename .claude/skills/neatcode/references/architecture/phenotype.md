# Architectural phenotype — the conformance protocol

Loaded at **Deep** depth, for `audit` and `study`, and whenever a change touches a module
boundary.

A repository has two architectures. The one it **claims** — in its README, its ADRs, its
folder names, its vocabulary — and the one it **expresses** — in its imports, its call graph,
where behaviour actually lives, and which modules must be edited together.

> **Architectural terminology is evidence of intent, not evidence of conformance.**

This protocol compares the two and returns a verdict. It is the code analogue of looking at a
rendered page: the directory tree, module boundaries, dependency direction, naming, tests,
manifests, and public interfaces are the observable phenotype.

---

## Step 1 · Collect the claim (genotype)

Read, and quote:

- `README` — the architecture section, and any diagram's caption.
- `ARCHITECTURE.md`, `DESIGN.md`, `docs/architecture/**`.
- ADRs — `docs/adr/**`, `docs/decisions/**`. These are the highest-quality claims because they
  record a decision and its rejected alternatives.
- `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `CONVENTIONS.md` — often where the *enforced*
  rules live, as opposed to the aspirational ones.
- `engineering.md`, if present.
- Manifests — workspace definitions, module declarations, package boundaries, visibility
  settings.
- Vocabulary in the source itself: folder names, package names, base classes, interface names.

Record each claim as a **quotable sentence with a location.** "The README says the domain layer
must not depend on infrastructure (README.md:44)" is a claim. "It's supposed to be clean
architecture" is not.

Note claims that exist only in folder names. `domain/` is a claim, and a weak one.

## Step 2 · Read the visible morphology

What the tree shows before you open a file:

- Top-level directories and what they group by — technical layer, feature, domain, deployment
  unit.
- Depth and fan-out. A tree that is deep and thin says something different from broad and flat.
- Test placement: colocated, mirrored tree, separate suite, absent.
- Generated, vendored, and source separation.
- File-size distribution — god modules at one end, one-function-per-file sprawl at the other.
- Naming grammar: is the same concept named the same way everywhere?
- Public surface: what is exported, and does the export list look designed or accidental?
- Where new code has landed recently. `git log` on the last twenty commits shows the *current*
  gravity, which is often not the documented one.

## Step 3 · Read the dependency phenotype

This is where the verdict is decided. Everything above is circumstantial; imports are facts.

- Which modules import which. Sample the actual import statements — do not infer from names.
- Direction against the claimed rule: does anything in `domain/` import from `infrastructure/`,
  `db/`, `http/`, or a framework package?
- Cycles between packages.
- The fan-in hub: which module does everything import? Is it supposed to be that?
- Boundary crossings: are there mapping/translation functions at the edges, or do foreign types
  pass straight through?
- Ports: is the interface used in production, or only in tests while production constructs the
  concrete type?

For most claims, one grep decides it. Run it and quote the result.

## Step 4 · Read behaviour placement

Where does the important logic actually live?

- Pick two or three representative behaviours — the ones a maintainer would name if asked what
  the system does.
- Trace each end to end. Note every module it passes through and where the *decisions* are made
  versus where data is merely moved.
- Then ask: **to change this behaviour, how many modules must be edited?** That number is the
  real modularity metric. A "vertical slice" architecture where one feature change touches five
  horizontal layers is not a vertical slice architecture.

## Step 5 · Compare and classify

State the claim, state the observation, return a verdict.

| Verdict | Means |
| --- | --- |
| **Conformant** | The claim is expressed and enforced. Dependencies obey it; boundaries hold; behaviour sits where the claim says. |
| **Partially conformant** | Mostly holds, with identified, bounded exceptions. Name them and say whether they are drift or documented pragmatism. |
| **Nominal** | The vocabulary is present; the functional property is absent. The folders say hexagonal; production constructs adapters directly. This is the most common verdict on AI-assisted codebases and the most important one to name. |
| **Contradictory** | The claim and the structure actively conflict, or two claims conflict with each other. |
| **Unverifiable** | The claim is not falsifiable as stated ("clean and modular"), or the evidence needed is not accessible. Say which. |
| **Coherent emergent alternative** | The code does not match the claim, but expresses a different, internally consistent architecture. **This is not automatically a defect.** Name the actual architecture. The finding is that the documentation is wrong, which is usually the cheaper thing to fix. |

That last verdict matters. An agent that treats every deviation from the README as decay will
recommend expensive rewrites toward a documented architecture the team abandoned deliberately
and correctly.

---

## Recognizing nominal architecture

Specific patterns worth checking directly, each of which is one command away from an answer:

| Claim | The check | Nominal when |
| --- | --- | --- |
| Layered / clean | Imports from inner to outer layers | `domain/` imports `infrastructure/`, or the dependency runs both ways |
| Hexagonal / ports & adapters | Who constructs adapters in production | Production instantiates the concrete adapter directly; the port is used only by tests |
| Bounded contexts | Shared internal types across contexts | Contexts import each other's internal models, or share one ORM entity set |
| Modular monolith | Controlled public surfaces | Modules import each other's internals; no visibility boundary is enforced by anything |
| Vertical slice | Files touched per feature change | Every feature change touches the same horizontal technical layers |
| Event-driven | Are events asynchronous and decoupled | Events are dispatched synchronously and awaited — a function call with extra steps |
| Compiler pipeline | Stage responsibilities | A stage performs another stage's job, or reaches back into a prior representation |
| Plugin architecture | Registration and discovery | The "plugins" are hard-coded and selected by a constant |
| Microservice boundaries | Data ownership | Services share one database schema and write to each other's tables |
| Repository pattern | Where queries live | Queries constructed at call sites outside the repository |

## The shared-module gravity well

Applies to every architecture. Check whether a generic `shared/`, `common/`, `core/`, `utils/`,
or `lib/` module has become the de facto authority: does it contain domain behaviour, and does
it import from feature modules? If the arrow points that way, it is not shared infrastructure —
it is the middle of the system, mislabelled. See
[`../taxonomy/authority.md`](../taxonomy/authority.md).

---

## Reporting

```markdown
### Architecture conformance

**Claim** · "Domain logic must not depend on infrastructure" — `README.md:44`, restated in `AGENTS.md:31`
**Observed** · 6 of 14 files under `src/domain/` import from `src/infra/`; `src/domain/order.ts:12`
imports the Prisma client directly. No architecture test enforces the rule.
**Verdict** · **Nominal.** The layer names are present; the dependency rule is not enforced and
is violated in 43% of domain files.
**Consequence** · The domain cannot be tested or reused without a database. Every Prisma upgrade
is a domain-logic migration.
**Correction** · Either (a) invert with repository interfaces owned by `domain/` and add a
dependency test, or (b) drop the layer vocabulary and document the transaction-script design
the code actually expresses. Both are honest; today's state is not.
**Confidence** · confirmed — import statements read directly.
```

The correction offering **both** directions is deliberate. Renaming the folders to match
reality is often the correct, cheap fix, and an audit that only ever recommends conforming to
the document is an audit that has confused documentation with architecture.
