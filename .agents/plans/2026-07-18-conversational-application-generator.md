# Conversational Application Generator Gate 0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` to execute this plan task by task.
> Stop at each human gate. Do not dispatch implementation work beyond a gate.

**Goal:** Produce and ratify the SEA Application Contract decision that must
exist before anyone can write an unambiguous generator implementation plan.

**Architecture:** This plan is deliberately documentation-first. It gathers the
current compiler evidence, forces the unresolved language and ownership choices
into an explicit human decision, records the result in ADR-013 and a normative
language contract, and then writes one separate Milestone 0 code plan. It does
not modify grammar, AST, Graph, bindings, emitters, providers, or CLI behavior.

**Tech stack:** Markdown ADRs, the existing SEA grammar/parser/formatter, Rust
workspace evidence, repository fixtures and test conventions.

## Global constraints

- Governing spec:
  `.agents/specs/domainforge-conversational-application-generator-spec.md`,
  Proposed v0.2.
- Follow `AGENTS.md`, `.github/copilot-instructions.md`,
  `docs/reference/sea-language-evolution-policy.md`, and
  `docs/templates/ADR-template-sea-language-change.md`.
- ADR-013 is the next available number in this reviewed workspace. If ADR-013
  exists when this plan runs, stop and report that the plan is stale; do not
  renumber or overwrite without revising this plan.
- Do not edit `domainforge-core/grammar/sea.pest` or any source, schema,
  binding, fixture, provider, profile, CLI, or test file under this plan.
- Do not create unparseable files with a `.sea` extension. Proposed syntax
  belongs in fenced examples inside the ADR/reference until ratification.
- ADR status starts as `Proposed`. Only an explicit human ratification may set
  it to `Accepted`; record the ratifier and approval date.
- Never treat ADR-012's direct-AST Cell implementation as a default precedent.
  ADR-012 itself marks that choice as a disputed ADR-011 deviation.
- Do not infer application semantics from instance fields, generated port
  names, provider defaults, or renderer needs.
- Preserve the current Rust-core/cross-binding rule: an accepted public AST,
  Graph, or primitive change must plan Python, TypeScript, and WASM parity.
- `.agents/` files are shared working state. In the reviewed workspace they are
  currently untracked because an existing `.gitignore` edit removes the
  `.agents/` rule. Do not edit or stage `.gitignore`; let the human decide
  whether `.agents/` artifacts become tracked. ADR and reference files under
  `docs/` are tracked review artifacts.
- Do not use placeholders such as `TBD`, `TODO`, `ADR-0XX`, angle-bracket
  filenames, wildcard fixture arguments, or “implement per ADR” in the
  continuation code plan.
- Do not commit unrelated user changes. Inspect `git status --short` before
  each tracked commit and stage only the exact files named by that task.

---

## Why the previous full plan was not executable

The superseded plan scheduled grammar, IR, providers, staging, proof, emitters,
skills, and end-to-end tests before these decisions existed:

1. the exact SEA syntax for typed records and application operations;
2. whether application semantics belong in Graph, another resolved semantic
   model, or an explicitly justified AST-owned contract;
3. the exact Rust types and stable serialized schema;
4. the policy-to-operation enforcement model;
5. how imports contribute application contracts to the semantic closure;
6. which types are public and therefore require binding parity;
7. the flagship domain and its exact write/read semantics.

Those decisions control every downstream filename, signature, schema, fixture,
diagnostic, and test. Planning downstream code before ratification can only
produce guesses. This plan stops at the decision boundary and requires a fresh
code plan after the boundary closes.

The previous plan also contained literal execution defects:

- `parse fixtures/application_generation/*.sea` expands to multiple inputs,
  but `ParseArgs` accepts one `PathBuf`;
- `format --check fixtures/application_generation/*.sea` puts `--check` before
  the positional file and expands multiple files, while `FormatArgs` accepts one
  file;
- draft fixtures used `#` comments although SEA comments use `//`;
- commands contained unresolved filenames such as `flagship`, binding, and
  approval placeholders;
- provider paths and provider IDs used inconsistent naming;
- the profile required `integration_proven` providers before integration proof
  existed, creating a circular bootstrap;
- several “tasks” touched entire subsystems and had no RED/GREEN test cycle,
  exact interfaces, or independently reviewable change.

None of those commands or downstream tasks survives in this Gate 0 plan.

---

### Task 1: Capture the repository baseline and decision evidence

**Dependencies:** None.

**Estimated scope:** Medium documentation task; three currently untracked
state/report files, no source changes.

**Files:**

- Create:
  `.agents/reports/2026-07-18-application-contract-decision-packet.md`
- Modify: `.agents/current_state.md`
- Modify: `.agents/next_steps.md`

**Interfaces:**

- Consumes: revised spec Sections 2, 4, 5, 14, 19, and 21; current parser,
  formatter, AST schema, Graph, Domain IR, language policy, and ADR-012.
- Produces: one evidence-backed decision packet with the exact D1–D10 questions
  required by Task 2.

- [ ] **Step 1: Verify the starting tree and ADR number**

Run:

```bash
git status --short --branch
test ! -e docs/specs/ADR-013-sea-application-contract.md
test -e docs/specs/ADR-012-cell-environment-declarations.md
```

Expected:

- `git status` may show unrelated user work; record it and do not modify it.
- both `test` commands exit `0`.
- if ADR-013 already exists, stop and report that this plan is stale; never
  overwrite or silently renumber it.

- [ ] **Step 2: Verify the existing parser baseline**

Run:

```bash
just rust-test
devbox run -- cargo run -p domainforge-core --features cli --bin domainforge -- parse fixtures/projection_cell/basic/model.sea --ast --format json
```

Expected:

- the repository Rust suite passes;
- the parse command exits `0` and prints JSON with a top-level
  `declarations` array.
- record the exact pass/fail result in the decision packet. A baseline failure
  blocks this plan; do not attribute it to documentation work.

- [ ] **Step 3: Write the current-capability evidence table**

Create the decision packet with these exact evidence rows:

| Question | Current evidence | Required conclusion |
|---|---|---|
| Do `Entity` declarations own typed fields? | `grammar/sea.pest` has no entity body; `DomainIr` infers extra fields from instances as `str`. | Existing entities cannot define application DTO/storage schema. |
| Do current ports own application contracts? | `projection/domain/ir.rs` has no `PortIr`; renderers synthesize repository/bus/read-model traits. | Generated traits are outputs, not semantic input. |
| Are policies scoped to operations? | Current Domain IR applies every policy as a no-op guard to every aggregate method. | v0.1 must define explicit scope and proven enforcement. |
| Does Graph preserve Cell-like declarations? | `cli/project.rs` dispatches Cell before Graph because Graph drops Cell context. | Ownership must be deliberately settled; Cell is not automatic precedent. |
| Are AST JSON shapes public? | `parser/ast_schema.rs` provides a stable serializable schema; parser/AST changes trigger language policy. | Any new public declaration needs schema and compatibility treatment. |
| Do bindings expose graph/core structures? | `src/python/graph.rs`, `src/typescript/graph.rs`, and `src/wasm/graph.rs` wrap core surfaces. | Public Graph/primitive additions require parity planning. |
| Can the current source hash close imports? | Hashing a formatted entry file does not capture resolved import meaning. | The accepted ownership model must support an import-resolved semantic closure. |

Use exact repository-relative paths and line references found during inspection.
Do not copy claims from the old plan without verifying them.

- [ ] **Step 4: Add the D1–D10 decision records**

For each record below, include: `Question`, `Why it matters`, `Repository
evidence`, `Options considered`, `Recommended option`, `Trade-offs`, and
`Human decision`.

| ID | Decision the human must settle |
|---|---|
| D1 | Whether typed application data is a new first-class SEA category, an extension of an existing category, or target configuration. |
| D2 | Whether use cases/operations are first-class SEA declarations or a typed extension of an existing semantic declaration. |
| D3 | Exact ownership and import behavior: Graph, a new resolved semantic model, or a justified AST-owned model. |
| D4 | Exact policy-to-operation scope and enforcement-point semantics; universal no-op guards are forbidden. |
| D5 | Minimal v0.1 type system, field constraints, required/optional rules, cardinality, identity, and reference semantics. |
| D6 | Exact effect/state, transaction, canonical failure, concurrency, idempotency, evidence, and lifecycle declarations, including an explicit `not_applicable` representation. |
| D7 | Stable serialized AST/contract schema version and which Rust types are public; binding consequences follow from this decision. |
| D8 | Backward compatibility, reserved keywords, formatter/printer behavior, module resolution, and semantic-closure contribution. |
| D9 | One flagship domain with exact command, query, policy/public-access decision, invalid/conflict path, persisted state, rollback/restart expectations, projection matrix, deterministic outputs, and expected outcome. |
| D10 | Canonicalization boundary: typed envelope normalization versus the existing semantic-pack canonical JSON primitive, including compatibility tests that preserve existing pack hashes/signatures. |

The packet must recommend one coherent set of choices, but it must not record a
human decision before the human supplies it.

- [ ] **Step 5: Verify the packet contains no undecided implementation claim**

Run:

```bash
rg -n '^## D([1-9]|10) ' .agents/reports/2026-07-18-application-contract-decision-packet.md
rg -n '^(TBD|TODO)|ADR-0XX|<flagship>|<binding|<approval' .agents/reports/2026-07-18-application-contract-decision-packet.md
```

Expected:

- the first command reports exactly ten decision headings;
- the second command exits `1` with no matches;
- every `Human decision` remains explicitly `Awaiting human decision`.

- [ ] **Step 6: Record checkpoint evidence**

Update `.agents/current_state.md` with the commands and results above. Set
`.agents/next_steps.md` to only:

1. human D1–D10 decision review;
2. ADR-013/reference drafting after those decisions;
3. ADR ratification and continuation-plan authoring.

Do not commit `.agents/` files unless the user explicitly changes repository
policy.

**Acceptance:** The decision packet contains verified evidence, one recommendation
per decision, no fabricated human decisions, and no unresolved file/interface
claims presented as settled.

---

### Human Gate A: Decide D1–D10

Present the decision packet to the domain and compiler maintainers. Ask them to
approve or amend each D1–D10 recommendation.

The response is sufficient only when it identifies:

- the approver or approving team;
- the decision date;
- an explicit answer for every D1–D10 record;
- any required change to the Proposed v0.2 release boundary.

Record the answers verbatim under each `Human decision` field in the packet.

**STOP:** Do not start Task 2 while any D1–D10 answer is absent, conditional, or
contradictory. Report the missing decision instead of choosing for the human.

---

### Task 2: Write ADR-013 and the normative language contract

**Dependencies:** Task 1 complete and Human Gate A satisfied.

**Estimated scope:** Medium documentation task; two tracked documents plus
currently untracked shared-state updates.

**Files:**

- Create: `docs/specs/ADR-013-sea-application-contract.md`
- Create: `docs/reference/sea-application-contract.md`
- Modify: `.agents/current_state.md`
- Modify: `.agents/next_steps.md`

**Interfaces:**

- Consumes: the recorded D1–D10 human decisions.
- Produces: a single Proposed architecture decision and a normative contract
  precise enough for a later plan to name grammar rules, Rust types, schemas,
  validation errors, fixtures, bindings, and tests without guessing.

- [ ] **Step 1: Write ADR-013 from the repository template**

Use every heading in
`docs/templates/ADR-template-sea-language-change.md`. Set:

```markdown
# ADR-013: SEA Application Contract

**Status:** Proposed
**Date:** 2026-07-18
**Deciders:** DomainForge Architecture Team

> **Ratification record** _(fill in only after explicit human acceptance)_:
> - Ratifier:
> - Approval date:
```

ADR-013 must:

- state one decision, not a menu of alternatives;
- explain why existing Entity/Resource/Flow/Policy, typed annotations,
  Mapping/Projection, Profile, imports, and provider bindings cannot safely
  express the chosen distinctions;
- include the spec Section 5.4 Adapter-field source matrix;
- declare exact semantic ownership and import-resolution behavior;
- list every affected parser, schema, formatter, module, Graph/model, projection,
  and binding surface;
- classify the change as additive or breaking and define migration accordingly;
- name exact positive, negative, compatibility, round-trip, schema, binding, and
  semantic-closure fixtures/tests required by the later implementation;
- define disconfirmation criteria.

- [ ] **Step 2: Write the normative language contract**

`docs/reference/sea-application-contract.md` must contain:

1. terminology and v0.1 scope;
2. complete proposed grammar in fenced Pest syntax with exact rule names;
3. complete proposed SEA examples for the D9 command and query;
4. a field-by-field semantic table for every `generation_ready` item;
5. exact validation invariants and stable diagnostic codes;
6. exact canonicalization set/sequence/default rules;
7. exact import, namespace, identity, and duplicate semantics;
8. exact internal Rust type names and field types proposed for implementation;
9. exact public/private boundary and binding obligations;
10. backward-compatibility and formatter/printer rules;
11. the two proposed proving fixtures as fenced SEA labelled with their future
    exact paths:
    `fixtures/application_generation/flagship/command-write.sea` and
    `fixtures/application_generation/flagship/query-read.sea`; do not create
    those `.sea` files until ADR-013 is accepted;
12. explicit out-of-scope items for v0.1.

The reference must not use “for example,” “such as,” “optional if needed,” or
“implementation-defined” for any item required to decide whether an operation
is `generation_ready`.

- [ ] **Step 3: Cross-check every required semantic field**

Create a traceability table in the reference with one row for each spec Section
5.1 item and these columns:

```text
Requirement | SEA syntax | AST/Rust field | Validation rule | APP diagnostic | D9 fixture line
```

Expected: every cell is concrete. An empty cell blocks review.

- [ ] **Step 4: Run documentation checks**

Run:

```bash
test -f docs/specs/ADR-013-sea-application-contract.md
test -f docs/reference/sea-application-contract.md
rg -n '^## (Proposed distinction|Current representational limitation|Existing mechanisms considered and why they fail|Cross-target semantics|Normative mappings|Grammar / AST impact|Compatibility impact|Migration path|Fixtures|Tests|Failure modes|Disconfirmation criterion)$' docs/specs/ADR-013-sea-application-contract.md
rg -n 'TBD|TODO|ADR-0XX|<flagship>|<binding|<approval|implement per ADR|optional if needed' docs/specs/ADR-013-sea-application-contract.md docs/reference/sea-application-contract.md
awk '/^```/{n++} END{exit n%2}' docs/specs/ADR-013-sea-application-contract.md docs/reference/sea-application-contract.md
```

Expected:

- both files exist;
- all ADR template headings are reported;
- placeholder scan exits `1` with no matches;
- code-fence check exits `0`.

- [ ] **Step 5: Inspect and commit only the proposal files**

Run:

```bash
git status --short
git diff --check -- docs/specs/ADR-013-sea-application-contract.md docs/reference/sea-application-contract.md
git diff -- docs/specs/ADR-013-sea-application-contract.md docs/reference/sea-application-contract.md
git add docs/specs/ADR-013-sea-application-contract.md docs/reference/sea-application-contract.md
git commit -m "docs: propose SEA application contract"
```

Expected:

- diff contains only the two named proposal files;
- `git diff --check` exits `0`;
- commit succeeds without staging unrelated work.

**Acceptance:** A reviewer can determine the exact proposed grammar, Rust model,
validation behavior, compatibility impact, and binding obligations from the two
documents alone.

---

### Human Gate B: Ratify ADR-013

Domain and compiler maintainers review ADR-013 and the normative reference.
They must test the D9 examples mentally against the domain meaning and verify
that every traceability row has one authoritative source.

If maintainers request changes, modify only the ADR/reference, rerun Task 2 Step
4, and resubmit. Do not edit compiler code.

After explicit acceptance:

1. replace `**Status:** Proposed` with `**Status:** Accepted`;
2. fill the ratifier and approval date with the human-provided values;
3. preserve the accepted decision text;
4. commit only the two documentation files with:

```bash
git add docs/specs/ADR-013-sea-application-contract.md docs/reference/sea-application-contract.md
git commit -m "docs: accept SEA application contract"
```

**STOP:** An agent may not infer acceptance from silence, an approved spec, or a
passing documentation check.

---

### Task 3: Write the Milestone 0 language implementation plan

**Dependencies:** Human Gate B satisfied and ADR-013 status is `Accepted` with a
complete ratification record.

**Estimated scope:** Small planning task; one currently untracked plan plus
shared state.

**Files:**

- Create:
  `.agents/plans/2026-07-18-conversational-application-generator-m0-language.md`
- Modify: `.agents/current_state.md`
- Modify: `.agents/next_steps.md`

**Interfaces:**

- Consumes: accepted ADR-013 and normative language contract.
- Produces: one code-level TDD plan for grammar-to-binding parity only.

- [ ] **Step 1: Verify ratification before planning code**

Run:

```bash
rg -n '^\*\*Status:\*\* Accepted$' docs/specs/ADR-013-sea-application-contract.md
rg -n '^> - Ratifier: .+' docs/specs/ADR-013-sea-application-contract.md
rg -n '^> - Approval date: [0-9]{4}-[0-9]{2}-[0-9]{2}$' docs/specs/ADR-013-sea-application-contract.md
```

Expected: each command reports exactly one line. Otherwise stop.

- [ ] **Step 2: Write one grammar-first continuation plan**

Use the `planning-and-task-breakdown`, `test-driven-development`, and
`superpowers:writing-plans` skills. The new plan must cover only:

1. grammar rules from the accepted reference;
2. internal AST parsing;
3. serialized AST schema and conversion;
4. formatter and parser printer;
5. module/import resolution;
6. accepted Graph/resolved-model ownership;
7. validation and diagnostics;
8. positive/negative/compatibility/round-trip fixtures;
9. Python, TypeScript, and WASM parity for every accepted public type;
10. the canonical semantic envelope, `source_set_hash`, and
    `semantic_closure_hash`, with typed normalization kept separate from the
    existing semantic-pack hash primitive unless D10 explicitly approves a
    versioned compatibility-preserving alternative.

The new plan must use exact accepted rule/type/field names, exact file paths,
small RED/GREEN/refactor packets, compilable test snippets, exact single-file
CLI commands, expected failures, and per-packet commits. It must not plan Adapter
IR, providers, generation, proof runners, Axum, SQLite, or skills.

- [ ] **Step 3: Verify the continuation plan is executable**

Run:

```bash
rg -n 'TBD|TODO|ADR-0XX|<flagship>|<binding|<approval|implement per ADR|similar to Task|write tests for' .agents/plans/2026-07-18-conversational-application-generator-m0-language.md
rg -n '^### Task [0-9]+:' .agents/plans/2026-07-18-conversational-application-generator-m0-language.md
rg -n '\*\*Files:\*\*|\*\*Interfaces:\*\*|Run:|Expected:' .agents/plans/2026-07-18-conversational-application-generator-m0-language.md
```

Expected:

- placeholder/shortcut scan exits `1` with no matches;
- task and execution-detail scans report content for every packet;
- manual self-review maps every accepted ADR/reference requirement to a task.

- [ ] **Step 4: Update shared state and stop**

Record the accepted ADR commit and continuation-plan path in
`.agents/current_state.md`. Set `.agents/next_steps.md` to only:

1. human review of the Milestone 0 code plan;
2. isolated-worktree creation for execution;
3. execution of the first approved TDD packet.

Do not execute the continuation plan in the same turn that writes it.

**Acceptance:** A lesser coding agent can implement the accepted language
contract without inventing syntax, ownership, types, tests, commands, or binding
scope.

---

## Later continuation plans — sequence only, not executable tasks

Create each plan only after the previous milestone has passed its spec stop gate:

1. Milestone 1: canonical envelope, semantic gate, inspect/review/diff/approval.
2. Milestone 2: Adapter IR, data-only providers, exact resolution, dry-run plan.
3. Milestone 3: planned sink, staging/lease/recovery, bounded proof process.
4. Milestone 4: compiled Rust local emitters and HTTP/SQLite integration proof.
5. Milestone 5: SEA/provider skills, progressive review UX, tutorial, evaluation.

Do not copy the deleted downstream tasks into these plans. Re-inspect the code
after each milestone because new accepted interfaces become the next plan's
ground truth.

Before writing the affected continuation plan, resolve these known governing
contract gaps through a spec amendment or dedicated ADR:

| Before milestone | Required settlement |
|---|---|
| Milestone 1 CLI/approval work | Define exact semantic and realization approval-capture commands, artifact paths, command inputs, and whether one or two approval arguments are consumed. |
| Milestone 1 machine contract | Close the JSON status enum: the spec envelope omits `generated` while its exit table permits it. Define command-specific legal statuses and transitions. |
| Milestone 2 provider work | Define truthful maturity bootstrap: descriptors begin below `integration_proven`; promotion occurs only after compiled emitter, conformance, and real integration evidence exist. |
| Milestone 2 scheduling | Reconcile the spec's Milestone 2 provider-binding-skill text with its later guided-workflow milestone before naming task ownership. |
| Milestone 3 safe writes | Specify the sink interface migration; wrapping current `ArtifactSink` cannot observe duplicate writes made directly through the inner sink. |
| Milestone 3 filesystem/process safety | Ratify supported platforms, case-fold/reserved-path rules, lease liveness, crash states, cleanup confirmation, executable allowlist, redaction algorithm, endpoint classification, and disposable-resource marker. |
| Milestone 4 generated app | Pin the exact generated tree, Cargo dependencies/versions/features, profile ceilings, provider configuration, and unique-temp-directory test protocol. |

These are not invitations for a later coding agent to choose. Each row is a
planning stop gate: settle the contract, then write the code plan.

---

## Final acceptance checklist for this Gate 0 plan

- [ ] Baseline parser evidence is recorded with command results.
- [ ] Decision packet contains D1–D10, verified repository evidence, one coherent
      recommendation, and no fabricated human answer.
- [ ] Human Gate A records explicit answers for every D1–D10 decision.
- [ ] ADR-013 and the normative reference contain exact proposed syntax, types,
      validation, imports, compatibility, tests, and binding scope.
- [ ] Documentation placeholder and fence checks pass.
- [ ] Proposal commit contains only ADR-013 and the normative reference.
- [ ] Human Gate B explicitly accepts ADR-013 and records ratifier/date.
- [ ] Acceptance commit contains only ADR-013 and the normative reference.
- [ ] Milestone 0 code plan contains no placeholders, optional ownership paths,
      multi-file CLI globs, or downstream generator work.
- [ ] `.agents/current_state.md` and `.agents/next_steps.md` match the last
      completed gate and never claim human approval early.

## Stop condition

This plan is complete when ADR-013 is explicitly accepted and the separate
Milestone 0 language code plan is ready for human review. It does not complete
the conversational application generator. That narrower claim is intentional:
the accepted language contract is the evidence required to plan the generator
without ambiguity.
