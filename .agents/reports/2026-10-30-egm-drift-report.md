# Drift Report — EGM Contracts vs. Source (2026-10-30)

Scope: the 61 `EGM.md` contracts committed in `c8917b7` on branch
`docs/egm-architectural-contracts`, checked against the working tree at that
commit.

Method: mechanical checks first, then manual reads of the highest-risk claims.

1. **Symbol resolution** — every `[file:symbol]` citation parsed out of all 61
   files and resolved against the filesystem, then checked for a real
   definition or word-boundary occurrence in the cited file (not just a
   substring). Result: `1612 checked, 1612 resolved, 0 weak`.
2. **Link resolution** — every relative `EGM.md` reference checked on disk, in
   both markdown-link and backticked-path form. Result: 0 broken.
3. **Coverage** — every non-test source file in each documented directory
   checked for at least one mention in that directory's contract. Result: 2
   files unmentioned (both addressed below).
4. **Semantic spot-check** — manually verified the claims most likely to be
   wrong: hardcoded versions, numeric constants, default values, and
   "not yet implemented" / "known gap" assertions.

Overall: **4 defects found and fixed in the contracts; 5 residual drifts
reported for human triage.** No drift was found in the code-vs-doc direction
for the sampled claims — every version string, numeric default, and
documented limitation matched its source exactly.

---

## Fixed during this review

These were defects in the generated contracts themselves, corrected before
committing the report.

- **`EGM.md` (root)** — claimed "26 `.sea` DSL samples and 24 `.json` schemas".
  Actual counts are **49** `.sea` files under `fixtures/` and **11** `.json`
  files under `schemas/`. Replaced the invented numbers with an unqualified
  description plus a pointer to the authoritative location.
- **`EGM.md` (root)** — stated the system "projects that graph into 22 target
  ecosystems" without qualification. 22 is the count of directories under
  `domainforge-core/src/projection/`, but `ProjectFormat` in
  `domainforge-core/src/cli/project.rs` declares **31 variants exposing 29
  distinct CLI format names**, and several formats (`ai-llm`, `ai-graph-ml`,
  `cep-eval`, `ai-learning`, `kg`, `calm`) have no dedicated projection
  subdirectory. Added a clarifying note so the two numbers cannot be confused.
- **`domainforge-core/src/cli/EGM.md`** — cited `[lib.rs]`, a path that does not
  resolve from that subdirectory. Corrected to `[../lib.rs]`.
- **`scripts/verify/EGM.md`** — cited `[domainforge-baseline.sh:elif]`. The
  script uses `if/else`, never `elif`. Re-cited to the real fallback command.

---

## Residual drift (reportable, not fixed)

### Source doc comment points at a file that does not exist

- **Where:** `domainforge-core/src/kg.rs:1087`, inside the `to_graph` doc
  comment.
- **Divergence:** the comment directs readers to
  `` `.sea/interaction/README.md` "Known Toolchain Boundaries" `` for
  guidance on instance/policy round-tripping. There is **no `.sea/` directory
  and no `.sea/interaction/README.md`** in the repository (the only root
  `.sea*` entry is `.sea-registry.toml`). The referenced section is
  unreachable.
- **Why it matters:** this is the documented escape hatch for a real
  functional gap (`to_graph` silently drops `sea:EntityInstance` and
  `sea:Policy` triples). A reader following the pointer to learn the
  workaround hits a dead end.
- **Note:** this is pre-existing source drift, not introduced by the EGM
  contracts. `domainforge-core/src/EGM.md` faithfully reproduces the
  documented gap, so it does not compound the error.
- **Suggested closure:** create `.sea/interaction/README.md` with a "Known
  Toolchain Boundaries" section, or re-point the comment at an existing doc
  such as `docs/subsystems/graph-store.md`. Size S.

### Empty and artifact-only directories carry no contract

- **Where:** `python/sea_dsl/` (empty) and `python/domainforge/` (two
  gitignored `.so` build artifacts, no tracked source).
- **Divergence:** neither got an `EGM.md`, by design. `python/tests/` is
  test-only and also excluded. The whole `python/` subtree is therefore
  undocumented except as a negative entry in the root blueprint.
- **Why it matters:** if real Python sources are meant to live there, their
  absence is itself the drift — the importable package is served from
  `domainforge-python/domainforge/__init__.py` instead, so `python/` is dead
  weight in the tree.
- **Suggested closure:** commit the intended sources, or delete both
  directories. Already recorded in `.agents/next_steps.md`. Size S.

### `commitlint.config.cjs` is not mentioned by any contract

- **Where:** `commitlint.config.cjs` at the repository root.
- **Divergence:** the only non-test source-extension file in a documented
  directory with no mention in its `EGM.md`. The root contract describes
  `run_commitlint.sh` and the commit conventions but never the config that
  imposes the `scope-empty: never` and kebab-case scope rules.
- **Why it matters:** the scope-required rule is exactly what a contributor
  needs before their first commit, and it is enforced by a hook, so a failure
  is a hard block.
- **Suggested closure:** add one bullet to the root `EGM.md` Local Interface
  section citing `[commitlint.config.cjs:rules]`. Size S.

### `domainforge-core/src/registry/tests.rs` sits inside a documented module

- **Where:** `domainforge-core/src/registry/tests.rs`.
- **Divergence:** `domainforge-core/src/registry/EGM.md` does not mention it.
  The file is `#[cfg(test)]`-gated inline unit tests, not an integration test
  under `tests/`, so it is neither pruned like a test directory nor described.
- **Why it matters:** low. It is genuinely test code and the contract's
  Verification section covers the suite. Flagged only because the coverage
  rule reports it.
- **Suggested closure:** accept as-is, or add a one-line note that unit tests
  live inline at `registry/tests.rs`. Size S.

### Contract scope excludes the `docs/` tree entirely

- **Where:** `docs/` (171 tracked Markdown files: ADRs, subsystem guides,
  workflows, tutorials, reference; `docs/archive/` is gitignored and excluded).
- **Divergence:** the bootstrap pruned `docs/` because it contains no source
  code, so no `EGM.md` describes the documentation system itself. Meanwhile
  `docs/` is where the authoritative ADRs live, and ADR-011 is cited
  repeatedly by the projection contracts.
- **Why it matters:** the projection contracts assert conformance to ADR-011
  §2, but nothing in the contract tree records where that ADR is or what it
  mandates. An agent reading only `EGM.md` files cannot resolve the citation.
- **Suggested closure:** either accept `docs/` as deliberately out of scope, or
  add a short `docs/EGM.md` that indexes the ADR set. Size S.

---

## Checks that found no drift

Recorded so the next reviewer does not repeat them.

- **Hardcoded versions all exact:** `ASYNCAPI_VERSION = "3.0.0"`,
  `LEAN_TOOLCHAIN = "leanprover/lean4:v4.15.0"`, `ZENML_TARGET_VERSION =
  "0.68"` with `ZENML_PIP_SPEC = "zenml>=0.68,<0.69"`,
  `OPENSHELL_VERSION = "0.1.3"` with `PROJECTION_VERSION =
  "domainforge-openshell/v1"`, `DAGGER_ENGINE_VERSION = "v0.21.4"`, and OPA
  `1.21.1` pinned in `rego.sh`.
- **Determinism claim exact:** `ids.rs` really does reuse
  `authority::compute_deterministic_hash`, which is `xxh64(data, 42)` formatted
  as `{:016x}` — matching the documented "xxh64 seed 42, 16 hex chars".
- **`formatter` defaults exact:** `IndentStyle::Spaces`, `indent_width: 4`,
  `max_line_width: 100`, `trailing_newline: true`, `preserve_comments: true`,
  `sort_imports: true`.
- **`kg.rs` known-gap claim exact:** the contract says `to_graph` drops
  `sea:EntityInstance`/`sea:Policy`; the source doc comment at `kg.rs:1079`
  says the same, and `from_graph` does emit those triples at lines 580 and
  647 — the direction of the gap is stated correctly, not inverted.
- **`buf` graceful degradation exact:** the `buf.rs:6` module doc and
  `BufLintResult::skipped("buf not installed")` at line 373 match the
  contract's claim.
- **`wasm` module asymmetry exact:** there is genuinely no `registry.rs` under
  `src/wasm/` (unlike `src/python/` and `src/typescript/`), as the contract
  states.
- **`cell` profile enumeration exact:** `KNOWN_PROFILES` is exactly the four
  listed ids — `python-agent-v1`, `typescript-agent-v1`, `rust-agent-v1`,
  `polyglot-agent-v1`.
- **Gate count exact:** `all.sh` invokes 14 per-target gate scripts, matching
  the root blueprint.
- **Projection coverage complete:** all 22 directories under `src/projection/`
  have a contract; none missing.
- **`bin/domainforge.rs` TODO claim exact:** the contract says global
  `verbose`/`quiet` are parsed but not consumed; the source carries
  `// TODO: Setup logging/tracing based on cli.verbose/quiet` at line 10.

## Notes on method limits

- The symbol check proves each cited name **exists** in the cited file. It
  cannot prove the surrounding prose describes it correctly. That is why the
  semantic spot-check above reads source directly rather than trusting the
  citation.
- Two rounds of this review already caught real errors the mechanical checks
  passed over: the invented `.sea`/`.json` counts in the root contract had
  valid-looking citations nearby, and no automated check would have flagged
  them. Treat the "no drift" result as scoped to what was sampled, not as a
  proof of total correctness.

## Recommended next step

The residual drift is all pre-existing source or scope-boundary debt, not
corruption introduced by the contracts. The highest-value fix is closing the
dead doc pointer at `kg.rs:1087`, since it is reachable by anyone reading the
public `to_graph` API documentation.
