# Current State: Task 2 complete — ADR-013 Proposed, awaiting Human Gate B

Branch: `agent/projection-targets`. An unrelated pre-existing `.gitignore`
modification removes the `.agents/` ignore rule, so `.agents/` appears as
untracked; this work did not edit `.gitignore` and staged only the two Task 2
proposal documents. `devbox.lock` remains modified from an earlier window and
was not touched.

## Task 2 deliverables (2026-07-18)

- Created `docs/specs/ADR-013-sea-application-contract.md` — Status
  `Proposed`, empty ratification record, all 12 template headings, §5.4
  Adapter-field source matrix reproduced, D3 ownership pipeline, full
  file-impact list, additive classification with breaking contingency,
  fixture/test naming, failure modes, disconfirmation criteria.
- Created `docs/reference/sea-application-contract.md` — exact Pest grammar
  (record/enum/operation/entity-body, contextual tokens,
  `application_decl_prefix` lookahead for `resource_decl`), field-by-field
  `generation_ready` semantics, APP001–APP015 diagnostics, canonicalization
  and hash rules (D10 boundary), D3 import/identity semantics, exact Rust
  types (`ApplicationContract`, `ResolvedModuleSet`, constituents), public
  boundary and binding functions, compatibility/formatter rules, both D9
  flagship fixtures fenced with future paths, out-of-scope list, and the
  §5.1 traceability matrix with concrete fixture line references.

## Task 2 check evidence

- Both `test -f` checks → exit 0.
- ADR heading scan → all 12 required headings reported.
- Placeholder scan (`TBD|TODO|ADR-0XX|...`) over both files → exit 1, no
  matches; reference banned-phrase scan (for example / such as /
  implementation-defined) → exit 1, no matches.
- Code-fence balance `awk` check → exit 0.
- `git diff --check` on the two files → exit 0; commit
  "docs: propose SEA application contract" staged only the two named files.

## Gate status

Human Gate A CLOSED (packet records D1–D10 acceptance, 2026-07-18).
Human Gate B OPEN: ADR-013 is `Proposed`; no acceptance may be inferred. No
grammar, source, schema, binding, test, or `.sea` fixture file was created or
modified. Task 3 is blocked until explicit ratification (ratifier + date).
