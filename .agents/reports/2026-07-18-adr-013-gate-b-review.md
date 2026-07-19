# ADR-013 Gate B Adversarial Review

Date: 2026-07-18  
Outcome: Technical blockers corrected; ADR-013 and amendments accepted by the
DomainForge repository maintainer on 2026-07-18; ratification commit `1320fed`.

## Material findings corrected

- Semantic hashing originally omitted resolved non-application meaning and used
  ambiguous/bare hash encodings. The revision defines source-set,
  semantic-pack-set, semantic-envelope, and artifact self-hashes with strict
  framing and `sha256:` prefixes.
- Existing exact-byte `ConceptId` identity was incorrectly described as NFC.
  The revision preserves legacy identity and limits normalized identity to new
  application symbols.
- Strict grammar would have emitted parser errors instead of APP001 for missing
  clauses. Operations now parse to ordered partial clauses and validate later.
- The Serde enum rule was incompatible with newtype/tuple variants. Public
  enums now use adjacent tagging.
- The flagship did not state how input became state or how `status = placed`
  was produced. Entity defaults and same-name state/output lowering now make
  that outcome explicit and testable.
- Policy evaluation, constraints, concurrency, idempotency, failures, and
  failure-detail population were under-specified. The revision closes the
  executable policy subset, interval rules, field compatibility, execution
  order, failure-kind mappings, and defers structured failure details.
- Canonical closure payloads reused authored AST nodes and Graph-random IDs.
  The revision requires dedicated resolved canonical payloads, deterministic
  flow occurrence IDs, and parity with deterministic instance `ConceptId`s.
- Import aliasing, relative logical paths, public binding names, diagnostics,
  schemas, and test paths were incomplete or inconsistent with the repository.
  The ADR/reference now specify them against actual files and APIs.

## Validation evidence

- Required ADR heading scan: 12 headings.
- Placeholder and ambiguity phrase scans: no matches.
- Code-fence balance: 18 fences, even.
- `git diff --check`: passed.
- D9 fenced fixture line enumeration: command lines 1–52 and query lines 1–30;
  traceability references match.
- Fresh-context adversarial review: three cycles; each reported blocker was
  either corrected or explicitly deferred from v0.1.

No source tests were run because this review changes documentation and shared
agent state only; compiler implementation remains prohibited before Gate B.
