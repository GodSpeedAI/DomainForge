# Milestone 0 Human Review Gate — Adversarial Review

Date: 2026-07-19
Scope: commits `3550baf` through `e3e82a4`, based on accepted ADR commit
`1320fed`
Decision: **REQUEST CHANGES — Milestone 0 is not accepted**

## Subsequent gate disposition

**ACCEPTED by the DomainForge repository maintainer on 2026-07-19.**

The acceptance applies to the remediated branch through commit `56895f2`, not
the originally reviewed `e3e82a4` snapshot. Remediation commits `cc6036f`
through `56895f2` address the recorded schema, field-semantics, shared-identity,
resolved-reference, canonicalization, persisted-metadata, diagnostic,
resource-budget, compatibility, semantic-pack, and cross-binding findings.
The maintainer explicitly directed the project to treat Milestone 0 as complete
and proceed to Milestone 1. This section records that later human decision; the
original adversarial findings below remain as historical review evidence.

Cross-model review was offered and explicitly skipped by the repository
maintainer. Three independent fresh-context reviews were reconciled with a
direct code and test inspection.

## Executive finding

The implementation is substantial and its existing test suite is green, but
it does not yet satisfy several normative ADR-013 requirements. The strongest
failures are semantic rather than cosmetic: invalid field contracts can be
emitted, imported identities can be changed or fabricated, canonical hashes
depend on authored ordering/spelling, persisted schemas accept malformed
documents, and the required cross-binding and diagnostic proof is absent.

Milestone 1 MUST NOT begin until every Critical and High finding below is fixed
and this gate is explicitly re-reviewed and accepted.

## Verification performed

- `just all-tests`: PASS — Rust, Python, and TypeScript suites completed; the
  TypeScript runner reported 195 passing tests. This recipe does not rebuild
  bindings and does not exercise WASM in a browser or `wasm32` environment.
- `cargo clippy -p domainforge-core --all-targets --features cli,python,typescript,wasm,json-schema -- -D warnings`:
  PASS with no warnings.
- `cargo fmt --all -- --check`: PASS.
- `git diff --check 1320fed..e3e82a4`: FAIL — trailing whitespace in
  `.agents/reports/2026-07-18-adr-013-gate-b-review.md:3`.
- Direct JSON Schema negative probe: malformed values inside contract and
  envelope collections were accepted because array items are unconstrained.
- Working tree was clean before this review report and state update.

Passing tests establish regression safety for what they cover; they do not
override the contract violations below.

## Release-blocking findings

### Critical

1. **APP003/APP012 field semantics are not implemented completely.**
   `domainforge-core/src/application/resolve.rs:1185` converts constraints but
   does not reject constraints on incompatible types, duplicate bounds, empty
   intervals, or invalid precision/scale combinations. Defaults at
   `resolve.rs:1247` are checked mainly for type shape, not key/optionality
   rules or satisfaction of constraints; supported quantity defaults are not
   lowered. Invalid authored models can therefore emit generation-ready
   contracts.

   Acceptance proof: table-driven negative tests for every reference-section
   invariant, including duplicate/inapplicable constraints, interval edges,
   decimal precision/scale, optional/key defaults, quantity defaults, and a
   default violating each applicable constraint.

2. **Graph and contract do not share one resolved identity model.**
   `domainforge-core/src/application/resolve.rs:67` concatenates all module
   ASTs under the first module's metadata and calls the legacy AST-to-Graph
   conversion again. Its own comment acknowledges that per-declaration
   namespaces are not preserved. Multi-namespace imports can therefore receive
   incorrect `ConceptId`s, contrary to D3 and the gate requirement.

   Acceptance proof: a multi-namespace diamond fixture in which Graph and
   contract IDs are compared for every shared declaration, plus duplicate-name
   cases proving no first-module namespace leakage.

3. **Envelope references are not consistently resolved.**
   `domainforge-core/src/application/envelope.rs:330` classifies declarations
   using the local namespace and raw authored names. Imported/aliased targets
   can be reconstructed as nonexistent local IDs. Lines 252–255 explicitly
   defer rewriting policy role references, although the accepted envelope
   requires resolved target identities.

   Acceptance proof: imported and aliased references for every reference-bearing
   declaration and policy role, with exact target `ConceptId`s asserted in
   both declarations and `resolved_references`.

4. **Canonical documents are not semantic-order/spelling independent.**
   Authored strings are copied into semantic payloads, module payload hashes
   retain AST declaration order (`envelope.rs:609` and `:649`), and the global
   declaration sort at `:645` omits logical module ID as the primary key.
   `document_self_hash` consequently protects serialized output, not the fully
   normalized semantic value promised by ADR-013.

   Acceptance proof: golden documents showing identical bytes and hashes after
   declaration/module reordering, import alias changes, equivalent decimal
   spellings, and required Unicode normalization; meaningful semantic changes
   must change the relevant hashes.

5. **The persisted JSON Schemas are not strict.**
   `schemas/application-contract-v1.schema.json:49`–`:52` declares four arrays
   without `items`. `schemas/canonical-semantic-envelope-v1.schema.json:76`–`:81`
   leaves most arrays and the nested contract untyped. Arbitrary scalars and
   objects therefore validate as supposedly strict persisted documents.

   Acceptance proof: complete closed schemas for all nested wire types plus
   negative vectors for unknown fields, missing fields, wrong scalar types,
   malformed nested objects, and malformed array members.

6. **Persisted-envelope validation does not validate all bound inputs and
   metadata.** `validate_semantic_envelope_document_json` at
   `domainforge-core/src/application/envelope.rs:934` checks schema version and
   two hashes, but does not independently enforce all required version,
   producer, source-set, semantic-pack, and envelope/contract consistency
   invariants. A self-consistent but contract-invalid artifact may be accepted.

   Acceptance proof: APP015 tamper vectors for every persisted metadata and
   input field, including internally recomputed self-hashes.

### High

1. **The APP001–APP015 proof claim is incomplete.** APP010 and APP012 are
   essentially registry/wording checks rather than behavioral rejection tests;
   APP009, APP014, and APP015 do not cover all closed reasons and metadata
   invariants. Diagnostics commonly omit required source coordinates, field
   path, expected/actual values, evidence, and remediation. Source/graph
   construction failures are also collapsed into APP015 in
   `application/resolve.rs:84`, conflating authored-source errors with persisted
   artifact validation.

2. **Cross-binding byte parity is not proved.** Python and TypeScript tests each
   compare one binding with itself. No test compares Rust, Python, TypeScript,
   and WASM output from the same source bytes. `just all-tests` neither rebuilds
   the native bindings nor runs a real WASM target, so it can pass against stale
   artifacts.

3. **Compatibility evidence is not a valid pre-change oracle.** The test says
   compatibility hashes must never be regenerated, but
   `application_compatibility_tests.rs:21` and `:41` say the keyword fixture's
   AST and formatter hashes were re-captured after implementation. The diff
   also changes `domainforge-core/std/core.sea` entities from private to
   exported without an accepted compatibility decision. Changing the observed
   baseline cannot prove unchanged behavior.

4. **D10 semantic-pack and D9 canonical golden evidence is insufficient.**
    Semantic-pack tests cover one synthetic canonical-JSON value and the empty
    pack set, not existing signed pack fixtures. D9 determinism tests compare
    two executions to each other rather than fixed accepted contract/envelope
    bytes and hashes. The gate did not provide canonical D9 document hashes or
    signature oracle values.

5. **The fixed public envelope boundary is incomplete.** Semantic packs are
    hardcoded empty at `domainforge-core/src/application/envelope.rs:676` and
    `:835`, so non-empty D10 inputs cannot cross the public boundary. The
    implementation also exposes `resolve_application_graph`, which was not the
    accepted fixed interface and currently exercises the broken merge described
    above.

6. **Import-to-namespace lookup is ambiguous.** Application resolution
    reconstructs imports using stripped relative paths and suffix matching.
    Two modules with the same suffix can resolve differently from the shared
    resolver's actual edge, undermining deterministic D3 identity.

7. **Public synchronous entry points have no resource budgets.** The Python,
    TypeScript, and WASM APIs accept unbounded source maps; resolution eagerly
    parses/clones sources and recursively traverses imports without a depth or
    size budget. This creates a straightforward memory/stack/latency denial-of-
    service surface for the intended conversational application workflow.

8. **Unrelated mutable tool configuration entered the milestone.**
    `.agents/specs/mise.toml` adds `npm:9router = "latest"`. It is unrelated to
    Milestone 0, mutable, and an avoidable supply-chain risk. Remove it from the
    milestone unless separately justified, reviewed, and immutably pinned.

## Human-gate checklist

- [ ] Both flagship sources emit **strict**, normatively valid documents.
  Parsing succeeds, but schema and semantic defects prevent confirmation.
- [ ] APP001–APP015 and every closed reason/invariant have focused proof.
- [ ] Graph and contract share resolved existing-concept IDs.
- [ ] Rust, Python, TypeScript, and WASM return byte-identical canonical JSON.
- [ ] Existing AST, formatter, Graph, policy, and semantic-pack behavior remains
  compatible. Green regression tests are encouraging, but recaptured oracles
  and the `std/core.sea` visibility change prevent confirmation.
- [x] No Adapter IR, provider, generator, proof runner, Axum, SQLite, approval,
  or skill implementation was found in the diff. Evidence: full diff/stat and
  path review of `1320fed..e3e82a4`.

## Evidence inventory supplied by the implementation

The 15 commits are `3550baf`, `20c3866`, `32ee2b9`, `b8e1b96`, `4cefb82`,
`8fa2aaa`, `b27f4fb`, `2a1fdb3`, `9329df1`, `26ce8a6`, `bf6e7fc`, `e8ff24d`,
`ea0fa3d`, `66a32b4`, and `e3e82a4`.

Recorded compatibility values currently in the tree:

- entity-no-body AST:
  `ffeba958f8f68f003b7b66ec3457aa2dc4aa01ff58591ce26fd141dd0a518c27`
- entity-no-body formatter:
  `7c5d6d7377345393f69109ad1967979b1e2b6a9c504a511f36129ef35836afc6`
- keyword-collision AST (post-change re-capture; not accepted as an oracle):
  `a8b14905e28026643cf295c8680a18a5816908ede5b117d8854c18a7cb37357f`
- keyword-collision formatter (post-change re-capture; not accepted):
  `4638add0cb48386e1d847c229e4aa14a73f00a4a046d7a236fce6b5e76fe166a`
- synthetic semantic canonical-JSON hash:
  `sha256:aa651abb9cf26447f1e097abebda14b989f16d90ef4272d6c0a584f89ca33ea8`
- empty semantic-pack-set hash:
  `sha256:6e5312099a8d89e1d271bd89d9fcb36b031069bd8dad72be397af6b331d85c40`
- Required existing-pack signature vectors and fixed D9 contract/envelope hashes:
  **not supplied**.

## Re-review entry criteria

1. Fix all Critical and High findings without weakening ADR-013 or changing
   golden expectations to follow the new output.
2. Add fixed, independently derived golden fixtures and negative tests that
   demonstrate each acceptance proof above.
3. Rebuild and test every binding, run a genuine WASM target where applicable,
   pass all focused/full quality commands including `git diff --check`, and
   provide exact artifacts and hashes required by the plan.
4. Present a finding-to-commit/test matrix. A new explicit human acceptance is
   required before any Milestone 1 planning or implementation.
