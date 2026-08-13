# Architectural signatures

Loaded when you need to name an architecture, or test whether a claimed one is real.

Each entry gives the **functional property** that defines it — the thing that is either true
or false — and the folder layout that is commonly mistaken for it. Judge by the functional
property. Folder names are a hypothesis.

**No layout is mandatory.** A repository that expresses none of these coherently is not
thereby broken; many excellent systems are "a well-organized set of modules" and nothing more.
Do not manufacture an architecture to have something to say.

---

### Layered

**Functional property** — Dependencies flow in one direction through ordered layers. A layer
never imports from a layer above it.
**Commonly looks like** — `presentation/`, `application/`, `domain/`, `data/`.
**The check** — Grep imports across layer boundaries. Any upward import falsifies it.
**Nominal when** — Layers exist as folders and imports cross freely, or a "service" layer only
forwards.
**Legitimate variants** — A relaxed rule permitting the top layer to see all layers.

### Clean architecture

**Functional property** — Dependencies point inward, toward policy. The innermost layer
depends on nothing external, including the framework. Interfaces are *defined* by the inner
layer and implemented by the outer.
**Commonly looks like** — `entities/`, `usecases/`, `adapters/`, `frameworks/`.
**The check** — Does the inner layer import any framework or infrastructure package? Where are
the interfaces declared — inside or outside?
**Nominal when** — Interfaces live in the outer layer, or entities carry ORM annotations.
**Cost note** — This architecture is frequently applied to systems whose policy is trivial,
where it produces pure ceremony. Judge whether the policy is worth the isolation.

### Hexagonal / ports and adapters

**Functional property** — The application core communicates only through ports it defines.
Every external interaction goes through an adapter, and **production wires them**.
**Commonly looks like** — `ports/`, `adapters/`, `core/` or `application/`.
**The check** — Does production code construct an adapter directly, or receive it through a
port? If the port's only consumer is a test, this is nominal.
**Nominal when** — The port exists to enable mocking and nothing else, while production takes
the direct path.

### Vertical slice

**Functional property** — A feature's code is colocated, and one feature change touches one
directory.
**Commonly looks like** — `features/checkout/`, `features/billing/`, each with its own handler,
logic, and data access.
**The check** — Take three recent feature commits. How many top-level directories did each
touch? One is conformant; five is a layered architecture with feature-shaped folders.
**Nominal when** — Slices exist but every change also edits shared horizontal layers.

### Feature-oriented organization

**Functional property** — Grouping by capability rather than by technical role, without the
stronger isolation claims of vertical slice or modular monolith.
**Commonly looks like** — Top-level directories named after product areas.
**The check** — Are the names domain nouns rather than technical roles?
**Not nominal by default** — This is a modest claim and usually true when it looks true.

### Domain-driven design

**Functional property** — A ubiquitous language expressed in the code; aggregates enforcing
invariants; bounded contexts with their own models; explicit context mapping.
**Commonly looks like** — `domain/`, aggregates, value objects, repositories, domain events.
**The check** — Do aggregates enforce invariants, or are they data bags with public setters?
Do contexts share models? Does the code's vocabulary match how the business speaks?
**Nominal when** — DDD tactical patterns are present as class-name suffixes with anaemic
models and one shared schema underneath. This is the most common false DDD.

### Modular monolith

**Functional property** — Modules with controlled public surfaces, deployed as one unit.
Module internals are unreachable from other modules — enforced by the language, the build, or a
test.
**Commonly looks like** — `modules/`, each with an `index`/`api`/`public` entry point.
**The check** — Can module A import module B's internal file? If yes and nothing prevents it,
the boundary is decorative.
**Nominal when** — Modules exist and import each other's internals freely.

### Event-driven

**Functional property** — Producers do not know consumers. Events are asynchronous and
delivered through a broker or a queue. Producers do not depend on consumer outcomes.
**Commonly looks like** — `events/`, handlers, an event bus.
**The check** — Is dispatch awaited? Does the producer branch on the result? Is there a real
transport, or an in-process synchronous emitter?
**Nominal when** — Events are synchronous function calls with extra indirection, which is the
most common form of this claim.

### Plugin architecture

**Functional property** — Capabilities are discovered and registered at runtime or build time
through a defined mechanism; adding one requires no change to the core.
**Commonly looks like** — `plugins/`, an interface, a registry.
**The check** — Is there a loading mechanism? Can a plugin be added without editing core code?
Is there more than one?
**Nominal when** — The registry is a hard-coded map and adding a "plugin" means editing it.

### Compiler pipeline

**Functional property** — Ordered stages with named intermediate representations. Each stage
consumes one representation and produces the next; no stage reaches backward.
**Commonly looks like** — `lexer/`, `parser/`, `ast/`, `resolve/`, `lower/`, `codegen/`.
**The check** — Does each stage have a distinct input and output type? Does any stage do
another's job — a parser that resolves names, a type checker that emits?
**Nominal when** — Stages exist as directories and share one mutable representation that every
stage annotates.

### Core plus adapters

**Functional property** — A stable core with a thin, replaceable perimeter. Weaker than
hexagonal: no port formalism, but the direction still holds.
**Commonly looks like** — `core/` plus several integration directories.
**The check** — Does `core/` import from any adapter?
**Note** — Often the honest description of a system that *claims* hexagonal. Offer it as the
rename.

### Monorepo / workspace topology

**Functional property** — Multiple independently-versioned or independently-buildable packages
in one repository, with declared inter-package dependencies.
**Commonly looks like** — `packages/`, `apps/`, `crates/`, plus a workspace manifest.
**The check** — Do packages depend on each other through declared manifest dependencies, or by
reaching across the filesystem into each other's source?
**Nominal when** — Packages exist but import each other by relative path, bypassing the
declared graph entirely.

---

## Judging without a claim

When nothing claims an architecture, do not invent one. Report what you observe:

> *"No architecture is claimed. Observed: feature-oriented top level (`billing/`, `catalog/`,
> `identity/`) with a shared `lib/` used only for genuinely generic helpers. Dependencies flow
> feature → lib, never lib → feature, and never feature → feature. This is coherent and
> internally consistent. The one exception is `catalog/pricing.ts` importing
> `billing/invoice.ts` (catalog/pricing.ts:9), which is the only cross-feature edge in the
> repository — worth either documenting or removing."*

That is more useful than assigning a textbook label. The exception found at the end is the
finding; the label would have been decoration.
