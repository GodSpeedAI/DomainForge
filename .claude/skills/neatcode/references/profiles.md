# Engineering profiles

Loaded at Step 3. A profile is a **bounded set of trade-offs**, not a doctrine. It answers:
what does "good" mean *here*, given what this code is for?

**The profile is inherited, never chosen for variety.** If `engineering.md` names one, use it.
Otherwise read it off the code around the change. Deliberately differing from the surrounding
code is a defect in a repository, not a virtue — this is the point where the design skill
NeatCode descends from is inverted rather than translated. Visual work rotates its fingerprint
so two pages do not look alike; code that varies for variety's sake is unlearnable.

Different *modules* of one repository may legitimately run different profiles — a
performance-constrained core beside a boundary-hardened adapter layer. That is a property of
the modules, not a preference of the author.

---

### Direct

*For small, well-bounded behaviour.*

Few abstractions · logic colocated · explicit control flow · no speculative extension points ·
minimal dependencies · the shortest path from input to result.

**Choose when** the behaviour is comprehensible in one sitting, has one caller or a few, and
has no variation pressure.
**Good looks like** a function other people can read without navigating.
**Fails when** genuine complexity is compressed into one place and the function becomes
untestable in parts.
**Most common defect** — none, if honestly applied. The risk is applying it to something that
is not actually simple.

---

### Domain-centered

*For invariant-rich business behaviour.*

Behaviour lives near the state it governs · explicit domain vocabulary · illegal states
constrained by types where possible · infrastructure kept outside the model · tests organized
around rules rather than around classes.

**Choose when** the rules are the hard part and they change more often than the plumbing.
**Good looks like** a domain module you can read to learn the business.
**Fails when** applied to CRUD, where it produces ceremony around data movement.
**Most common defect** — anaemic models: data in the domain, behaviour in the services.

---

### Pipeline

*For staged transformations.*

Named intermediate representations · explicit stage contracts · diagnostics accumulated rather
than thrown · deterministic transformations · clear parse / validate / lower / emit separation.

**Choose when** the work is a sequence of transformations over a representation.
**Good looks like** each stage testable in isolation with a fixture in and a fixture out.
**Fails when** the stages are artificial and the data does not actually flow one way.
**Most common defect** — a stage reaching backward into a prior representation, or a stage
that does two stages' jobs.

---

### Boundary-hardened

*For external integrations.*

Translation at the boundary · timeouts and retries explicit · idempotency considered · external
types confined to the adapter · failures retaining operational context · rate limits and
circuit breaking where the dependency warrants.

**Choose when** the system talks to something it does not control.
**Good looks like** a module you can read to learn exactly how this system behaves when the
dependency is slow, wrong, or gone.
**Fails when** applied to internal calls, where it adds ceremony to something that cannot fail
that way.
**Most common defect** — the timeout is set and the retry is not idempotent.

---

### Operational

*For infrastructure and long-running services.*

Lifecycle explicit · structured observability · bounded resource use · graceful shutdown ·
degraded states represented rather than hidden · health and readiness separated.

**Choose when** the code runs continuously and someone is on call for it.
**Good looks like** an operator can answer "what is it doing and is it healthy?" without
attaching a debugger.
**Fails when** applied to short-lived scripts.
**Most common defect** — degradation that is invisible; startup that succeeds without its
dependencies.

---

### Performance-constrained

*For verified hot paths.*

Allocation and copying controlled · benchmark before abstraction · data layout considered ·
explicit complexity budget · readability sacrificed only against a measurement.

**Choose when** a measurement says so. Not before.
**Good looks like** a comment naming the benchmark and the budget, next to the code that is
otherwise inexplicable.
**Fails when** applied speculatively — which is nearly always how it is misapplied.
**Most common defect** — optimization with no measurement, producing code that is both slower
and unreadable.

**Report it as a finding** when this profile is applied without a benchmark. "Performance" is
not a licence; it is a claim requiring evidence.

---

### Evolutionary

*For systems under known variation pressure.*

Stable semantic core · narrow, deliberate extension seams · compatibility made explicit ·
versioning strategy stated · deprecation with a window.

**Choose when** variation has already happened at least twice, or the product's roadmap makes
it certain.
**Good looks like** a small number of well-defended seams, not general configurability.
**Fails when** the variation is imagined. Then it is speculative generality wearing a profile's
name.
**Most common defect** — every axis made extensible, which makes none of them usable.

---

## Reading the profile off a repository

When `engineering.md` does not name one, infer it — and say that you inferred it:

| Observation | Suggests |
| --- | --- |
| Rich types, few layers, behaviour on entities, ubiquitous domain nouns | domain-centered |
| Thin modules, direct calls, little indirection, small files | direct |
| Named IR types, stage directories, fixture-driven tests | pipeline |
| Adapter modules, explicit timeout/retry config, mapping functions at edges | boundary-hardened |
| Health endpoints, structured logging, graceful shutdown, metrics | operational |
| Benchmarks in the repository, allocation-conscious code, profiling notes | performance-constrained |
| Versioned APIs, deprecation notes, plugin registries with real plugins | evolutionary |

Mixed signals are normal and usually mean different modules run different profiles. Say which
module you are judging under which profile — a finding that applies the wrong profile is a
false positive with a confident tone.
