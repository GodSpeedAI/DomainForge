# Adversarial Review: Conversational Application Generator Spec

Date: 2026-07-18

Artifact reviewed:
`.agents/specs/domainforge-conversational-application-generator-spec.md`
(Proposed v0.1.1, 2,301 lines)

Outcome: request changes. The draft had a strong product direction but was not
implementable or safe as written. It treated roadmap concepts as existing
compiler contracts, gave provider packs an undefined executable boundary, and
combined several releases and a research benchmark into one definition of done.
The artifact was revised in place as Proposed v0.2.

## Highest-risk findings and dispositions

1. **Missing semantics (critical).** Actual `DomainIr` does not own typed use
   cases, API shapes, policy scope, transaction, idempotency, authority, or
   lifecycle. It infers fields from instances as strings and generates generic
   ports/no-op guards. Evidence: `projection/domain/ir.rs` and language
   renderers. Resolution: a Semantic Expressiveness Gate, grammar-first
   Application Contract settlement, and Adapter-field source matrix.

2. **Unsafe provider execution (critical).** Data, lowering rules, templates,
   emitters, and proof commands were conflated. Resolution: v0.1 descriptors are
   data-only and reference compiled allowlisted emitters/suites; external
   executable packs require a separate sandbox/signature/ABI design.

3. **False sink guarantees (critical).** Current `ArtifactSink` overwrites text
   and does not plan, reject duplicates, stage, or commit atomically. Evidence:
   `projection/sink.rs`. Resolution: specified a planned/staged sink, leases,
   same-filesystem commit, collision/symlink tests, and conservative pruning.

4. **Policy safety (critical).** Existing “every policy guards every method”
   no-op hooks could make generated endpoints appear governed. Resolution:
   explicit policy-to-operation scope and proven enforcement are generation
   prerequisites; affected operations block otherwise.

5. **Approval replay/TOCTOU.** Prior approvals omitted profile, catalog,
   requirements, selected pack hashes, resolver policy, proof profile, and
   operational consequences. Resolution: complete semantic/realization review
   snapshots and revalidation before plan and commit.

6. **Incorrect semantic hash.** Canonically formatted entry-source bytes do not
   close imports, namespace resolution, semantic packs, or interpretation.
   Resolution: canonical semantic-closure hash plus separate source-set hash.

7. **Unbounded scope.** Two service profiles, many providers, generic solving,
   skills, three domains, production faults, and model benchmarking were one
   release. Resolution: Foundation, v0.1 local, substitution, production, and
   research conformance levels with stop gates.

8. **Circular/variable proof boundary.** Application locks and proof artifacts
   could hash one another; runtime observations were treated as deterministic.
   Resolution: resolution/plan lock, application-content lock, and downstream
   evidence manifest; deterministic proof plan, observational proof record.

9. **Supply-chain gaps.** Ranges lacked source/checksum/revision, features,
   license/advisory policy, resolver identity, image/system-package identity,
   and locked mode. Resolution: exact dependency provenance and frozen native
   verification; missing cache is blocked, integrity/policy failure is invalid.

10. **Canonicalization ambiguity.** Set/sequence order, Unicode, duplicates,
    defaults, duration/number formats, and YAML aliases were unspecified.
    Resolution: explicit canonical rules and golden vectors.

11. **Unsafe recovery/deletion.** Stale staging and orphan files could be
    deleted without reliable ownership. Resolution: owner-token leases,
    inspect/clean recovery, default refusal, and hash-checked `--prune`.

12. **Weak secret boundary.** Schema validation cannot recognize every secret
    string, and subprocesses can echo resolved values. Resolution: closed
    `SecretRef`, constrained schemas, environment allowlists, bounded redaction,
    and canary-secret tests.

13. **Undefined completion and classifications.** “Complete,” blocked,
    incomplete, and failed were not total or aligned with CLI behavior.
    Resolution: exact v0.1 behavior, total classification table, exit codes,
    cancellation, retry, cleanup, and required-skipped semantics.

14. **Organizational-projection ambiguity.** “Applicable” was not a conformance
    rule. Resolution: a `rust-local-v1` required/optional/not-applicable matrix
    and manifest status for every known projection.

15. **Provider substitution churn.** Full provider provenance in every domain
    file contradicted stable domain output. Resolution: ownership-scoped short
    headers and complete provenance in manifest/lock.

16. **Binding/API parity ambiguity.** Application orchestration and pure IR
    projections were conflated. Resolution: CLI-only filesystem orchestration
    for v0.1; ADR-011 in-memory surfaces for pure inspection; public core types
    follow Python/TypeScript/WASM parity rules.

17. **Review UX debt.** The draft emitted many artifacts but did not bound
    decision burden or require change summaries, blocking-question handling,
    data/external-effect disclosure, or first-use instructions. Resolution:
    progressive one-page reviews, semantic/realization diff, inspect/doctor/
    dry-run, sample request, `.env.example`, and comprehension-oriented evals.

## Verification performed

- Read all 2,301 original lines and all current repository instructions.
- Inspected the grammar, AST, Domain IR and renderers, CLI, `ArtifactSink`, Cell
  IR/lock, projection ADR, canonical hash helpers, security policy, generated
  artifact policy, workspace manifest, schemas, fixtures, scripts, and justfile.
- Ran an independent fresh-context adversarial review and reconciled all 17
  findings. Cross-model review was offered and not run during this revision.
- Replaced the spec with 1,193 lines / 8,307 words of revised normative text.
- Structural checks: 69 unique headings, 22 balanced code fences, no trailing
  whitespace, and every repository path cited in the baseline exists.
- Searched for superseded high-risk promises and found none outside explicit
  forbidden/deferred context.

No source code or runtime behavior changed, so compilation/test suites were not
run for this documentation-only task.
