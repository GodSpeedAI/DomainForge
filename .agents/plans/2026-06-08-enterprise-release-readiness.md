# DomainForge Enterprise Release Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Do not skip verification. Do not mark a task complete unless its evidence commands have been run and captured.

**Goal:** Resolve every release-blocking and enterprise-readiness issue from `.agent/reports/predeployment-adversarial-review-20260608-1504.md` so DomainForge can be deployed as trustworthy executable domain meaning infrastructure.

**Architecture:** Keep Rust `sea-core` canonical. Fix semantic correctness and deterministic projections in Rust first, then regenerate or align Python, TypeScript, WASM, packaging, release workflows, and docs around that source truth. Treat `.sea` files and package/release inputs as untrusted data; generated artifacts must be reproducible and produced by CI from clean source, not treated as hand-edited source truth.

**Tech Stack:** Rust 2021, Pest, PyO3/maturin, napi-rs, wasm-bindgen/wasm-pack, Bun/npm, GitHub Actions, Python 3.11+, TypeScript, Protobuf/CALM/RDF/Turtle.

## Enterprise Definition of Done

DomainForge is enterprise ready only when all of the following are true:

- `just all-tests`, `cargo test --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, Python tests, TypeScript tests, packaging smoke tests, WASM build/smoke tests, and dependency audits pass from a clean checkout.
- `.sea` parser behavior is namespace-correct, precision-preserving, deterministic, bounded, and safe for untrusted input.
- CALM, RDF/Turtle, Protobuf, AST/JSON, Python, TypeScript, Rust, and WASM surfaces preserve the same domain meaning or fail explicitly with diagnostics.
- Projections are byte-stable for identical semantic input by default. Volatile timestamps, random IDs, local paths, or machine state are opt-in.
- Generated release artifacts are produced by CI from source, packed, installed, loaded, and smoke-tested before publication.
- Release workflows fail on real publish/build/security failures and do not expose tokens to unverified downloaded tools or dependency lifecycle scripts.
- Public docs, package metadata, examples, version constants, stubs, declarations, badges, and release playbooks match working code.
- Every finding in the adversarial report is closed by a regression test or static workflow lint with a named proof command.

## Non-Negotiable Implementation Rules

- Read `.github/copilot-instructions.md` before making code changes.
- Use TDD for each defect: write the failing regression first, run it, implement the minimal fix, rerun the test.
- Commit by task or tightly related task group. Do not mix parser, projection, packaging, release, and docs fixes in one large commit.
- Do not hand-edit generated build outputs as the fix. Fix generators, manifests, scripts, or workflows.
- Treat checked-in `pkg`, `sea-core/pkg`, `.node`, `dist`, wheels, tarballs, old logs, and scratch files as build outputs unless explicitly converted into fixtures.
- Prefer Bun as the JavaScript development/test package manager if keeping existing project direction, but release automation must use a frozen install (`bun install --frozen-lockfile`) or switch fully to npm with a committed `package-lock.json` and `npm ci`.
- Do not claim any proof command passed unless it was run fresh after the relevant change.

## Task 0: Prepare the Branch and Baseline Evidence

**Findings covered:** all, establishes baseline.

**Files:**
- Read: `.github/copilot-instructions.md`
- Read: `.agent/reports/predeployment-adversarial-review-20260608-1504.md`
- Modify only if useful: `.agent/plans/2026-06-08-enterprise-release-readiness.md`

**Steps:**

1. Read project guidance:
   ```bash
   sed -n '1,260p' .github/copilot-instructions.md
   ```

2. Capture baseline state:
   ```bash
   git status --short --branch
   git rev-parse HEAD
   just --list
   ```

3. Run the known failing proof command and save its current failure for comparison:
   ```bash
   just all-tests
   ```
   Expected baseline: FAIL in Rust doctests, matching DF-001.

4. Do not fix anything in this task. Commit nothing unless this plan file itself was added.

**Definition of Done:** Baseline failure is reproduced or explained if behavior has changed.

**Evidence Required:** Command output showing branch, commit, and current `just all-tests` status.

## Task 1: Establish Release-Readiness Policy and Artifact Hygiene

**Findings covered:** DF-019, stale artifacts, unanswered generated artifact policy, release reproducibility.

**Files:**
- Modify: `.gitignore`
- Modify: `docs/RELEASE_PROCESS.md`
- Modify: `docs/playbooks/local-release-preparation.md`
- Create: `docs/reference/generated-artifacts-policy.md`
- Modify: `README_WASM.md`
- Modify: `.github/copilot-instructions.md`
- Optional remove if tracked and confirmed build outputs: `dist/*`, `*.node`, `sea.tar.gz`, `pkg/*`, `sea-core/pkg/*`, old `fix_*.py`, old logs under `tmp/` and `.logs/`

**Steps:**

1. Write `docs/reference/generated-artifacts-policy.md` defining:
   - Source truth: Rust source, grammar, schemas, manifests, lockfiles, docs, examples, tests.
   - Build outputs: `pkg`, `sea-core/pkg`, `dist`, wheels, tarballs, native `.node`, generated WASM JS/d.ts/wasm.
   - Allowed checked-in generated files only when they are explicit fixtures or golden test inputs under a fixture path.
   - CI must regenerate, pack, install, and smoke-test release artifacts.

2. Update `.gitignore` to ignore build outputs that should not be source truth:
   ```gitignore
   /dist/
   /target/
   /*.node
   /sea*.tar.gz
   /pkg/
   /sea-core/pkg/
   /.logs/
   /tmp/
   ```
   Keep exceptions only for intentional fixtures.

3. Remove tracked generated outputs only after confirming with `git ls-files`:
   ```bash
   git ls-files 'dist/*' '*.node' 'pkg/*' 'sea-core/pkg/*' 'sea*.tar.gz' 'tmp/*' '.logs/*'
   ```
   If files are tracked and are not fixtures, remove them with `git rm`.

4. Update release docs and Copilot instructions to state that generated artifacts are not hand-edited source truth.

5. Run:
   ```bash
   git status --short
   ```

**Definition of Done:** Repository policy is explicit, build outputs are ignored or removed if tracked, and future agents know not to patch generated artifacts as source.

**Evidence Required:** `git ls-files` output before cleanup, `git status --short`, and docs diff showing the generated artifact policy.

## Task 2: Fix Rust Doctests and Make `just all-tests` Reach Every Suite

**Findings covered:** DF-001.

**Files:**
- Modify: `sea-core/src/primitives/entity.rs`
- Modify: `sea-core/src/primitives/resource.rs`
- Modify: `sea-core/src/primitives/flow.rs`
- Modify: `sea-core/src/primitives/instance.rs`
- Optional modify: `sea-core/src/lib.rs` if re-exports are needed

**Steps:**

1. Run the focused failing doctest command:
   ```bash
   cargo test -p sea-core --features cli --doc
   ```
   Expected before fix: FAIL with `E0308` in doctests.

2. Update doctest examples to import dependency types from the same crate version as `sea_core`. Prefer examples like:
   ```rust
   use sea_core::{Entity, Resource, Flow, ConceptId};
   use sea_core::units::{Dimension, Unit};
   use sea_core::rust_decimal::Decimal;
   use sea_core::serde_json::json;
   ```
   If `rust_decimal` or `serde_json` are not re-exported, either re-export them intentionally from `sea-core/src/lib.rs` or rewrite doctests to avoid external direct dependency types.

3. Do not silence real examples with `ignore` unless they are intentionally illustrative and cannot compile by design. Enterprise docs should compile.

4. Run:
   ```bash
   cargo test -p sea-core --features cli --doc
   cargo test -p sea-core --features cli
   just all-tests
   ```

**Definition of Done:** Rust doctests compile, `just all-tests` runs through Rust, Python, and TypeScript suites.

**Evidence Required:** Passing output for `cargo test -p sea-core --features cli --doc` and `just all-tests`.

## Task 3: Fix Parser Namespace Semantics and Decimal Quantities

**Findings covered:** DF-008, DF-009, DF-010.

**Files:**
- Modify: `sea-core/grammar/sea.pest` only if grammar clarification is needed
- Modify: `sea-core/src/parser/ast.rs`
- Modify: `sea-core/src/parser/ast_schema.rs` if AST schema type changes
- Modify: `sea-core/src/parser/ast_convert.rs` if schema conversion changes
- Add or modify tests: `sea-core/tests/parser_tests.rs`
- Add or modify tests: `sea-core/tests/parser_integration_tests.rs`
- Add or modify tests: `sea-core/tests/namespace_registry_tests.rs` if reference resolution behavior changes
- Modify docs: `docs/reference/grammar-spec.md`

**Steps:**

1. Add failing regression tests:
   - `Role "Approver" in governance` parses namespace/domain as `governance`.
   - `Entity "Customer" in sales` and `Entity "Customer" in support` both parse into graph successfully.
   - same-name duplicate in the same namespace still fails.
   - `Resource "Money" USD in finance` and same name in another namespace work.
   - `Flow "Money" from "A" to "B" quantity 1.5` parses and preserves `Decimal("1.5")`.
   - `quantity 2147483648` parses and preserves value.

2. Run focused tests and confirm failure:
   ```bash
   cargo test -p sea-core --features cli parser --test parser_tests
   cargo test -p sea-core --features cli parser --test parser_integration_tests
   ```

3. Fix `parse_role` in `sea-core/src/parser/ast.rs` so it skips `Rule::in_keyword` and consumes the following identifier for the namespace/domain.

4. Change duplicate symbol tables in graph conversion from name-only keys to namespace-aware keys. Use a helper key type such as:
   ```rust
   fn concept_key(namespace: &str, name: &str) -> (String, String) {
       (namespace.to_string(), name.to_string())
   }
   ```
   Resolve unqualified references within the active/default namespace first. If multiple namespace candidates exist and no active namespace disambiguates, return an ambiguity diagnostic rather than choosing silently.

5. Change flow AST quantity from `Option<i32>` to `Option<Decimal>` or equivalent precision-preserving type. Parse numeric text with `Decimal::from_str`.

6. Update JSON/schema conversion and docs for decimal flow quantities.

7. Run:
   ```bash
   cargo test -p sea-core --features cli --test parser_tests
   cargo test -p sea-core --features cli --test parser_integration_tests
   cargo test -p sea-core --features cli --test flow_tests
   ```

**Definition of Done:** Namespaces are semantic identity boundaries, roles parse correctly, decimal/large quantities are accepted without precision loss, and ambiguity fails explicitly.

**Evidence Required:** Passing parser/flow tests plus at least one CLI proof:
```bash
printf 'Role "Approver" in governance\n' > /tmp/role.sea
cargo run -p sea-core --features cli --bin sea -- parse --format json /tmp/role.sea
```
Output must show `governance`, not `"in "`.

## Task 4: Harden Namespace, Import, Registry, and Output Path Trust Boundaries

**Findings covered:** DF-002 and threat-model answer.

**Files:**
- Modify: `sea-core/src/registry/mod.rs`
- Modify: `sea-core/src/module/resolver.rs`
- Modify: `sea-core/src/cli/project.rs`
- Modify: `sea-core/src/projection/protobuf.rs`
- Add tests: `sea-core/tests/cli_tests.rs`
- Add tests: `sea-core/tests/module_resolution_tests.rs`
- Add tests: `sea-core/tests/protobuf_projection_tests.rs`
- Add docs: `docs/reference/security-model.md`

**Steps:**

1. Add failing tests for unsafe namespace output:
   - `@namespace "/tmp/escape"` with `--multi-file` fails.
   - `@namespace "../escape"` fails.
   - `@namespace "a/b"` fails unless slash namespaces are explicitly supported and mapped safely.
   - Windows drive/prefix-like namespace strings fail on all platforms.

2. Add tests for import/registry boundaries:
   - Imports cannot escape workspace root through `..` unless explicitly allowed by a safe resolver policy.
   - Registry file resolution does not follow symlinks outside the registry root by default.

3. Implement a namespace/package validation helper in `sea-core/src/projection/protobuf.rs` or a shared module:
   ```rust
   pub fn validate_proto_package_namespace(ns: &str) -> Result<(), ProjectionError>
   ```
   Accept only package-safe dotted identifiers: `^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)*$`.

4. In `sea-core/src/cli/project.rs`, after joining `args.output` and `rel_path`, normalize and verify the final path stays under the output root. Do this even after namespace validation.

5. In `registry/mod.rs`, change `follow_links(true)` to false by default or add an explicit option for following symlinks. Enterprise default is no symlink escape.

6. In `module/resolver.rs`, enforce a safe import root for filesystem modules and keep `std:` imports as the only built-in external namespace.

7. Document `.sea` threat model in `docs/reference/security-model.md`.

8. Run:
   ```bash
   cargo test -p sea-core --features cli --test cli_tests
   cargo test -p sea-core --features cli --test module_resolution_tests
   cargo test -p sea-core --features cli --test protobuf_projection_tests
   ```

**Definition of Done:** Untrusted `.sea` cannot cause writes outside explicit output roots or read/import outside allowed roots by default.

**Evidence Required:** Regression tests pass and the original DF-002 proof command now fails without creating `/tmp/domainforgereviewabs/escape.proto`.

## Task 5: Make Protobuf Projection Valid, Safe, and Deterministic

**Findings covered:** DF-011, DF-012, DF-013.

**Files:**
- Modify: `sea-core/src/projection/protobuf.rs`
- Modify: `sea-core/src/projection/mod.rs` if error types/config are shared
- Modify: `sea-core/src/cli/project.rs`
- Add tests: `sea-core/tests/protobuf_projection_tests.rs`
- Add docs: `docs/how-tos/export-to-protobuf.md`
- Add docs: `docs/reference/protobuf-projection.md` if absent

**Steps:**

1. Add failing tests:
   - `--include-services` generates no unresolved request/response types.
   - `Entity "123 Customer"` fails with diagnostic or emits valid prefixed identifier such as `Sea123Customer`.
   - reserved words such as `class`, `message`, `service`, `rpc`, `package` are handled.
   - projecting same input twice yields byte-identical output by default.

2. Create a central identifier sanitizer:
   ```rust
   fn sanitize_proto_ident(raw: &str, kind: ProtoIdentKind) -> Result<String, ProjectionError>
   ```
   It must handle message names, fields, enums, services, methods, packages, and reserved words consistently.

3. Generate deterministic response messages for services or use a stable imported type such as `google.protobuf.Empty`. Prefer explicit response messages if they carry useful semantics.

4. Remove `Utc::now()` from default generated `.proto` output. Support optional metadata only through a CLI flag such as `--include-generated-metadata` or environment-controlled `SOURCE_DATE_EPOCH`.

5. Add an internal generated-type resolver test that parses the generated `ProtoFile` model, not only the output string.

6. Run:
   ```bash
   cargo test -p sea-core --features cli --test protobuf_projection_tests
   cargo run -p sea-core --features cli -- project --format protobuf --include-services examples/namespaces/logistics/core.sea /tmp/services.proto
   cargo run -p sea-core --features cli -- project --format protobuf sea-core/examples/basic.sea /tmp/p1.proto
   cargo run -p sea-core --features cli -- project --format protobuf sea-core/examples/basic.sea /tmp/p2.proto
   cmp /tmp/p1.proto /tmp/p2.proto
   ```

**Definition of Done:** Protobuf projection emits valid, deterministic, path-safe contracts that standard Protobuf consumers can compile.

**Evidence Required:** Tests pass; `cmp` exits 0; service output contains every referenced response message or import.

## Task 6: Make CALM Export/Import Semantically Complete and Deterministic

**Findings covered:** DF-014, DF-015, DF-016.

**Files:**
- Modify: `sea-core/src/calm/export.rs`
- Modify: `sea-core/src/calm/import.rs`
- Modify: `sea-core/src/calm/models.rs` if needed
- Modify: `sea-core/src/primitives/flow.rs` or parser graph conversion for deterministic parsed flow IDs
- Add tests: `sea-core/tests/calm_round_trip_tests.rs`
- Add tests: `sea-core/tests/calm_schema_validation_tests.rs`
- Modify docs: `docs/reference/calm-mapping.md`
- Modify docs: `docs/how-tos/export-to-calm.md`
- Modify docs: `docs/how-tos/import-from-calm.md`

**Steps:**

1. Add failing tests:
   - Export/import graph with one Pattern preserves pattern name and regex.
   - Export/import entity/resource attributes preserves scalar, array, object values.
   - Same input projected to CALM twice is byte-identical by default.
   - Existing CALM schema validation still passes.

2. Make export deterministic by default:
   - Remove wall-clock `sea:timestamp` from default output, or set it only when explicit option is passed.
   - For parsed declarative flows, derive stable IDs from namespace/resource/from/to/quantity/source identity instead of UUID v4.

3. Update importer to dispatch by `metadata["sea:primitive"]` before assuming `NodeType::Constraint` means Policy.

4. Deserialize `sea:attributes` for entities, resources, and instances and reapply via public setters.

5. Update docs to state deterministic default behavior and any opt-in volatile metadata flag.

6. Run:
   ```bash
   cargo test -p sea-core --features cli --test calm_round_trip_tests
   cargo test -p sea-core --features cli --test calm_schema_validation_tests
   cargo run -p sea-core --features cli -- project --format calm examples/namespaces/logistics/core.sea /tmp/c1.json
   cargo run -p sea-core --features cli -- project --format calm examples/namespaces/logistics/core.sea /tmp/c2.json
   cmp /tmp/c1.json /tmp/c2.json
   ```

**Definition of Done:** CALM preserves SEA meaning for supported primitives and emits byte-stable output by default.

**Evidence Required:** CALM round-trip/schema tests pass; `cmp` exits 0 for repeated projection.

## Task 7: Make RDF/Turtle/KG Round Trips Preserve Meaning

**Findings covered:** DF-017.

**Files:**
- Modify: `sea-core/src/kg.rs`
- Modify: `sea-core/src/kg_import.rs` if Turtle/RDF import path uses it
- Add tests: `sea-core/tests/turtle_entity_export_tests.rs`
- Add tests: `sea-core/tests/turtle_resource_export_tests.rs`
- Add tests: `sea-core/tests/kg_uri_encoding_tests.rs`
- Add tests: `sea-core/tests/rdf_xml_typed_literal_tests.rs`
- Modify docs: `docs/how-tos/generate-rdf-turtle.md`
- Modify docs: `docs/reference/calm-mapping.md` only if cross-format notes change

**Steps:**

1. Add failing round-trip tests:
   - `Entity "Central Warehouse" in logistics` survives KG export/import with exact name and namespace.
   - `Resource "Money" USD in finance` survives KG export/import with unit `USD` and namespace.
   - names with spaces, Unicode, quotes, and percent-sensitive characters survive.

2. Implement percent-decoding for local names during import.

3. Read `sea:namespace` and `sea:unit` triples before constructing graph primitives.

4. Preserve deterministic ordering in KG output. If any `HashMap` affects output order, sort or use `IndexMap`/`BTreeMap` in output paths.

5. Run:
   ```bash
   cargo test -p sea-core --features cli --test kg_uri_encoding_tests
   cargo test -p sea-core --features cli --test turtle_entity_export_tests
   cargo test -p sea-core --features cli --test turtle_resource_export_tests
   cargo test -p sea-core --features cli --test rdf_xml_typed_literal_tests
   ```

**Definition of Done:** KG/RDF round-trip preserves names, namespaces, units, and encoded characters for supported SEA concepts.

**Evidence Required:** New round-trip tests fail before fix and pass after fix.

## Task 8: Align Runtime Versioning Across Rust, Python, TypeScript, WASM, and Packages

**Findings covered:** DF-021.

**Files:**
- Modify: `sea-core/src/lib.rs`
- Modify: `sea-core/src/python/mod.rs` or current version export file
- Modify: `sea-core/src/typescript/mod.rs` if version export exists or should be added
- Modify: `sea-core/src/wasm/mod.rs` if version export exists or should be added
- Add tests: `sea-core/tests/runtime_toggle_tests.rs` or create `sea-core/tests/version_tests.rs`
- Add tests: `tests/test_primitives.py` or create `tests/test_version.py`
- Add tests: `typescript-tests/primitives.test.ts` or create `typescript-tests/version.test.ts`
- Modify docs: `README.md`, `README_PYTHON.md`, `README_TYPESCRIPT.md`, `README_WASM.md`

**Steps:**

1. Add tests asserting:
   - Rust `sea_core::VERSION == env!("CARGO_PKG_VERSION")`.
   - Python `sea_dsl.__version__` matches `importlib.metadata.version("sea-dsl")` when installed, and at least equals Cargo/package version in dev.
   - TypeScript and WASM expose the same version if version export exists.

2. Change Rust version constant:
   ```rust
   pub const VERSION: &str = env!("CARGO_PKG_VERSION");
   ```

3. Ensure bindings export the Rust version rather than hard-coded strings.

4. Run:
   ```bash
   cargo test -p sea-core --features cli --test version_tests
   just python-test
   just ts-test
   ```

**Definition of Done:** Runtime and package versions match across supported surfaces.

**Evidence Required:** Version tests pass and `PYTHONPATH=python python -c "import sea_dsl; print(sea_dsl.__version__)"` prints `0.10.0` or the current manifest version.

## Task 9: Repair Python Binding Type Stubs and Static API Contract

**Findings covered:** DF-018.

**Files:**
- Modify: `python/sea_dsl/sea_dsl.pyi`
- Modify: `python/sea_dsl/__init__.py` only if exports are wrong
- Modify: `sea-core/src/python/*.rs` only if runtime exports need naming cleanup
- Add tests: `tests/test_python_stub_parity.py`
- Add or modify: `pyproject.toml` for mypy config if needed

**Steps:**

1. Add a parity test that imports `sea_dsl.__all__` and checks each public symbol has a top-level declaration in `sea_dsl.pyi`.

2. Add a type-check sample under `tests/typing/sea_dsl_public_api.py` importing every public symbol and constructing representative `Entity`, `Resource`, `Flow`, `Instance`, `ResourceInstance`, `Role`, `Relation`, `Expression`, semantic-pack APIs, and authority APIs.

3. Update `sea_dsl.pyi` to match runtime exports and correct `Instance` vs `ResourceInstance` shapes.

4. Run:
   ```bash
   just python-test
   .venv/bin/python -m mypy tests/typing/sea_dsl_public_api.py
   ```

**Definition of Done:** Python runtime exports and stubs agree, and public typing samples pass.

**Evidence Required:** Stub parity test and mypy sample pass.

## Task 10: Repair TypeScript Declarations and Package Smoke Tests

**Findings covered:** DF-004.

**Files:**
- Modify generator source if declarations are generated: `sea-core/src/typescript/*.rs`
- Modify fallback/manual declarations only if source of truth: `index.d.ts`
- Add tests: `typescript-tests/declarations.test.ts`
- Add config: `typescript-tests/tsconfig.declarations.json`
- Modify: `package.json`
- Modify: `tsconfig.json`

**Steps:**

1. Add a declaration typecheck command to `package.json`:
   ```json
   "test:types": "tsc --noEmit --strict --skipLibCheck false -p typescript-tests/tsconfig.declarations.json"
   ```

2. Create `typescript-tests/tsconfig.declarations.json` that includes `index.d.ts` and a small consumer file.

3. Add consumer sample importing `Expression.aggregation`, `NamespaceRegistry.discover`, and common public APIs.

4. Fix the declaration generator/source:
   - Reserved parameter `function` becomes `aggregateFunction` or `_function`.
   - `Self | null` becomes `NamespaceRegistry | null`.
   - Regenerate or update `index.d.ts`.

5. Run:
   ```bash
   npm run test:types
   just ts-test
   ```

**Definition of Done:** Published TypeScript declarations compile under strict consumer settings.

**Evidence Required:** `npm run test:types` exits 0.

## Task 11: Fix Native npm Package Content and Install/Require Proof

**Findings covered:** DF-003.

**Files:**
- Modify: `package.json`
- Modify: `index.js` if loader/package naming needs alignment
- Modify: `napi.config.js`
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release-npm.yml`
- Add script: `scripts/smoke_npm_package.sh`
- Add test or script: `scripts/check-npm-pack.mjs`

**Steps:**

1. Decide packaging model:
   - Preferred short-term release fix: include generated `sea-core.*.node` files at package root in `files`.
   - Longer-term optional platform packages are acceptable only if they are actually built, published, and declared in `optionalDependencies`.

2. Update `package.json` `files` to include the active platform root native addon:
   ```json
   "files": [
     "index.js",
     "index.d.ts",
     "sea-core.*.node",
     "README.md",
     "LICENSE"
   ]
   ```
   Adjust if napi-rs produces a different name.

3. Add `scripts/check-npm-pack.mjs` to parse `npm pack --dry-run --json` and assert required package files are present.

4. Add `scripts/smoke_npm_package.sh`:
   - builds native package
   - runs `npm pack`
   - installs tarball into `mktemp -d`
   - runs `node -e "const sea=require('@domainforge/sea'); console.log(Object.keys(sea).length)"`

5. Add CI step after TypeScript build.

6. Run:
   ```bash
   npm run build
   node scripts/check-npm-pack.mjs
   bash scripts/smoke_npm_package.sh
   ```

**Definition of Done:** Packed npm package installs and loads in a clean temp project on the current platform.

**Evidence Required:** `scripts/smoke_npm_package.sh` exits 0 and `npm pack --dry-run --json` includes native addon or declared optional platform package.

## Task 12: Make JavaScript Release Installs Frozen and Token-Safe

**Findings covered:** DF-007 and accepted package-manager answer.

**Files:**
- Modify: `.github/workflows/release-npm.yml`
- Modify: `.github/workflows/ci.yml`
- Modify: `package.json`
- Keep: `bun.lock`
- Optional add if choosing npm instead: `package-lock.json`
- Add script: `scripts/lint_release_workflows.py`

**Steps:**

1. Choose one release install strategy:
   - If Bun remains authoritative, use `bun install --frozen-lockfile` in release and CI.
   - If npm becomes authoritative for release, generate and commit `package-lock.json`, then use `npm ci`.

2. Remove `npm install` from release workflows unless `package-lock.json` is committed and `npm ci` is used.

3. Build before setting `NODE_AUTH_TOKEN`.

4. Avoid lifecycle scripts during publish if feasible:
   ```bash
   npm publish --ignore-scripts --access public
   ```
   If scripts must run, document why and prove token exposure is minimized.

5. Add `scripts/lint_release_workflows.py` checks:
   - no `npm install` in release workflows
   - no `npm publish` before frozen install/build
   - no token env during build steps

6. Run:
   ```bash
   python3 scripts/lint_release_workflows.py
   ```

**Definition of Done:** Release npm workflow is reproducible and does not expose publish tokens to dependency install/build lifecycle.

**Evidence Required:** Workflow lint passes and release workflow diff shows frozen install and token scoped only to publish.

## Task 13: Fix WASM Package Generation, Metadata, Docs, and Smoke Tests

**Findings covered:** DF-019, DF-020, DF-027 partly.

**Files:**
- Modify: `scripts/build-wasm.sh`
- Modify: `README_WASM.md`
- Modify: `docs/reference/wasm-api.md`
- Modify: `package.json` if root scripts include WASM
- Modify: `.github/workflows/release.yml`
- Modify: `.github/workflows/release-npm.yml`
- Add script: `scripts/smoke_wasm_package.mjs`
- Add script: `scripts/check-wasm-pack.mjs`
- Remove or stop tracking stale: `pkg/package.json`, generated `pkg/*`, `sea-core/pkg/*` if Task 1 policy removes generated outputs

**Steps:**

1. Make `scripts/build-wasm.sh` produce a canonical publish directory, for example `target/wasm-pkg`, with package name `@domainforge/sea-wasm` and version from Cargo/package metadata.

2. Do not publish from stale root `pkg`.

3. Add pack check asserting:
   - package name `@domainforge/sea-wasm`
   - version equals Cargo/package version
   - documented exports exist
   - `.wasm`, JS, and `.d.ts` files exist

4. Add Node smoke script using the correct generated target behavior. If using `--target web`, document and test explicit WASM bytes for Node. If Node support is a goal, consider a separate `--target nodejs` package or dual target.

5. Update docs to use generated JS property API (`entity.id`, not `entity.id()`) and actual constructors (not `Entity.new`).

6. Fix `.github/workflows/release.yml` so GitHub release creation depends on `build-wasm-release`.

7. Run:
   ```bash
   ./scripts/build-wasm.sh
   node scripts/check-wasm-pack.mjs
   node scripts/smoke_wasm_package.mjs
   ```

**Definition of Done:** WASM package is generated from source, named/versioned correctly, smoke-tested, and included in release dependencies.

**Evidence Required:** Pack check and smoke script pass; workflow `create-release.needs` includes WASM build.

## Task 14: Secure Release Secret Handling and Publish Failure Semantics

**Findings covered:** DF-005, DF-006.

**Files:**
- Modify: `.github/actions/decrypt-secrets/action.yml`
- Modify: `.github/workflows/release-npm.yml`
- Modify: `.github/workflows/release-pypi.yml`
- Modify: `.github/workflows/release-crates.yml`
- Add script: `scripts/lint_release_security.py`
- Modify docs: `docs/RELEASE_PROCESS.md`

**Steps:**

1. Replace raw `curl` execution of `sops` with one of:
   - a pinned official setup action with digest pinning, or
   - explicit SHA256 verification before execution, or
   - preinstalled trusted tool on GitHub runner with version assertion.

2. Prefer OIDC trusted publishing for PyPI/npm/crates if feasible. If not feasible, keep encrypted secrets but minimize exposure.

3. Remove `continue-on-error: true` and `|| echo` from publish steps.

4. Implement explicit already-published logic:
   ```bash
   if npm view "$PKG@$VERSION" version >/dev/null 2>&1; then
     echo "Already published"
     exit 0
   fi
   npm publish --access public --ignore-scripts
   ```
   Use equivalent checks for PyPI/crates where applicable.

5. Add `scripts/lint_release_security.py` checking:
   - no `curl` downloaded executable without checksum/signature
   - no `continue-on-error` on publish steps
   - no `|| echo` masking publish failures

6. Run:
   ```bash
   python3 scripts/lint_release_security.py
   ```

**Definition of Done:** Release workflows fail on real publish failures and never execute unverified downloaded tools with release secrets.

**Evidence Required:** Release security lint passes and workflow diffs show explicit duplicate-version handling.

## Task 15: Fix Archive Extraction and CI Helper Security

**Findings covered:** DF-025.

**Files:**
- Modify: `scripts/ci_tasks.py`
- Add tests: `tests/test_ci_tasks.py` or `scripts/tests/test_ci_tasks.py`
- Modify: `justfile` if test recipe is added

**Steps:**

1. Add failing test that builds a tar containing `../outside.txt`, calls the package verification helper, and asserts:
   - helper rejects archive
   - no file appears outside extraction root

2. Add equivalent zip traversal test with `../outside.txt` and absolute path entries.

3. Implement safe extraction:
   - reject absolute paths
   - reject `..`
   - reject symlinks and hardlinks in tar
   - resolve joined destination and require it stays under temp root
   - use `filter="data"` on Python 3.12+ as defense in depth

4. Run:
   ```bash
   python3 -m pytest tests/test_ci_tasks.py -q
   just ci-verify-package <known-good-package> sea
   ```
   If no known-good package exists, create one in `/tmp` inside the test.

**Definition of Done:** Package verification cannot write outside temp extraction root for tar or zip.

**Evidence Required:** Traversal tests pass.

## Task 16: Fix Dependabot and Release Workflow Gates

**Findings covered:** DF-026, DF-027.

**Files:**
- Modify: `.github/workflows/dependabot-automerge.yml`
- Modify: `.github/workflows/release.yml`
- Add script: `scripts/lint_workflow_gates.py`
- Add tests: `tests/test_workflow_lints.py`

**Steps:**

1. Update `release.yml`:
   ```yaml
   create-release:
     needs: [build-release, build-python-wheels, build-wasm-release]
   ```

2. Update Dependabot auto-merge:
   - Do not proceed when checks list is empty.
   - Require known check names or rely on branch protection with `gh pr merge --auto`.
   - Fail on pending, queued, cancelled, skipped, timed out, neutral, or missing checks unless explicitly allowed.

3. Add workflow lint tests:
   - every artifact-uploading release job is in `create-release.needs`
   - Dependabot workflow does not treat `[]` as success

4. Run:
   ```bash
   python3 scripts/lint_workflow_gates.py
   python3 -m pytest tests/test_workflow_lints.py -q
   ```

**Definition of Done:** Release waits for all artifacts and dependency auto-merge cannot bypass CI.

**Evidence Required:** Workflow gate lint/tests pass.

## Task 17: Repair Docs, Examples, Metadata, and Release Playbooks

**Findings covered:** DF-022, DF-023, DF-024, DF-028, documentation claims audit.

**Files:**
- Modify: `README.md`
- Modify: `README_PYTHON.md`
- Modify: `README_TYPESCRIPT.md`
- Modify: `README_WASM.md`
- Modify: `pyproject.toml`
- Modify: `docs/RELEASE_PROCESS.md`
- Modify: `docs/playbooks/local-release-preparation.md`
- Modify: `docs/reference/README.md`
- Modify: `docs/reference/grammar-spec.md`
- Modify: `docs/how-tos/*.md` as needed
- Add script: `scripts/check_docs.py`
- Add tests: `tests/test_docs_examples.py`

**Steps:**

1. Replace nonexistent `from sea import Model` examples with real `sea_dsl.Graph` examples.

2. Update badges and requirements:
   - Rust `1.77+` unless lowered and tested.
   - Python `3.11+`.
   - Node/Bun requirements match package metadata.

3. Remove MIT classifier from `pyproject.toml`.

4. Update docs links to existing files.

5. Fix release docs so every `just` recipe referenced exists, or replace with script commands that exist.

6. Remove or qualify unproven marketing claims:
   - “Always Correct. Always in Sync.”
   - “provable correctness”
   - “10,000 entities under 100ms”
   - “pre-built packages, no compilation required”
   Keep claims only if backed by tests/benchmarks/package smoke evidence.

7. Add `scripts/check_docs.py`:
   - local markdown link check
   - extract `just ...` commands and verify recipes exist
   - verify version support text matches manifests

8. Add README smoke tests for Python snippets that should run.

9. Run:
   ```bash
   python3 scripts/check_docs.py
   python3 -m pytest tests/test_docs_examples.py -q
   ```

**Definition of Done:** Public docs are accurate, executable where promised, and free of broken local links and unsupported API claims.

**Evidence Required:** Docs checker and docs example tests pass.

## Task 18: Add Dependency Audit and Release Verification Tooling

**Findings covered:** audit evidence gaps, enterprise readiness.

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `justfile`
- Add: `deny.toml`
- Modify: `package.json`
- Add script: `scripts/enterprise_verify.sh`
- Add script: `scripts/check_release_artifacts.sh`

**Steps:**

1. Add `deny.toml` with explicit license/advisory/bans policy. Start strict enough for enterprise:
   - deny known vulnerabilities
   - deny yanked crates
   - allow Apache-2.0/MIT/BSD/Unicode-style compatible licenses as appropriate
   - document exceptions explicitly

2. Add `just audit`:
   ```just
   audit:
       cargo audit
       cargo deny check
       bun audit || true
   ```
   If Bun audit is not reliable, document the limitation and use a supported JS audit tool aligned with the chosen package manager.

3. Add `just enterprise-verify` that runs the full gate:
   ```just
   enterprise-verify:
       cargo fmt --all --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace --all-targets --all-features
       cargo test -p sea-core --features cli --doc
       just all-tests
       just audit
       npm run test:types
       bash scripts/smoke_npm_package.sh
       ./scripts/build-wasm.sh
       node scripts/smoke_wasm_package.mjs
       maturin build --release
       python3 scripts/check_docs.py
       python3 scripts/lint_release_security.py
       python3 scripts/lint_release_workflows.py
       python3 scripts/lint_workflow_gates.py
   ```

4. Add CI job or final release preparation workflow running `just enterprise-verify`.

5. Run:
   ```bash
   just enterprise-verify
   ```

**Definition of Done:** One command proves release readiness across code, projections, bindings, packages, docs, and release workflows.

**Evidence Required:** `just enterprise-verify` exits 0 in a clean checkout.

## Task 19: Performance and Resource-Abuse Guardrails

**Findings covered:** threat model, docs claims, enterprise deployability.

**Files:**
- Add tests: `sea-core/tests/parser_resource_limits_tests.rs`
- Add tests: `sea-core/tests/projection_resource_limits_tests.rs`
- Modify: `sea-core/src/parser/mod.rs`
- Modify: `sea-core/src/module/resolver.rs`
- Modify: `sea-core/src/cli/*.rs` if limit flags are added
- Add docs: `docs/reference/security-model.md`
- Add docs: `docs/reference/performance.md`

**Steps:**

1. Add tests for:
   - deeply nested expressions fail gracefully or parse within bounded depth
   - long strings/comments do not panic
   - import cycles produce deterministic diagnostics
   - excessive import graph depth fails with clear error
   - projection of large but valid model does not write outside output root or exhaust memory in tests

2. Add parser/module resolver limits with documented defaults:
   - max import depth
   - max source file size for CLI, configurable
   - max diagnostic suggestions, to avoid huge error output

3. If performance claims remain, add benchmark or test evidence. If no benchmark is added, remove performance claims from public docs.

4. Run:
   ```bash
   cargo test -p sea-core --features cli --test parser_resource_limits_tests
   cargo test -p sea-core --features cli --test projection_resource_limits_tests
   ```

**Definition of Done:** Malformed or adversarial `.sea` input fails safely with bounded resource usage and deterministic diagnostics.

**Evidence Required:** Resource-limit tests pass and docs state limits.

## Task 20: Final Enterprise Release Gate and Report Closure

**Findings covered:** all.

**Files:**
- Create: `.agent/reports/predeployment-remediation-closure-YYYYMMDD-HHMM.md`
- Modify: `CHANGELOG.md` if release-facing changes require notes
- Modify: `RELEASE_NOTES.md` if applicable

**Steps:**

1. Run from a clean checkout:
   ```bash
   git status --short --branch
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-targets --all-features
   cargo test -p sea-core --features cli --doc
   just all-tests
   cargo audit
   cargo deny check
   npm run build
   npm run test:types
   bash scripts/smoke_npm_package.sh
   ./scripts/build-wasm.sh
   node scripts/check-wasm-pack.mjs
   node scripts/smoke_wasm_package.mjs
   maturin build --release
   python3 scripts/check_docs.py
   python3 scripts/lint_release_security.py
   python3 scripts/lint_release_workflows.py
   python3 scripts/lint_workflow_gates.py
   just enterprise-verify
   ```

2. Re-run original adversarial proof commands from the report for each finding, especially:
   - original Protobuf path traversal PoC
   - original npm pack/install/require PoC
   - original TypeScript declaration check
   - original role namespace parse
   - original duplicate namespace parse
   - original decimal quantity parse
   - repeated Protobuf/CALM `cmp`
   - CALM Pattern/attribute round trip
   - RDF namespace/unit round trip
   - WASM pack metadata check

3. Create closure report with table:
   - finding ID
   - fix commit
   - regression test
   - proof command
   - result
   - residual risk

4. Do not claim `RELEASE ACCEPTABLE` unless all P0/P1 findings have passing proof and unresolved audit commands are settled.

**Definition of Done:** Closure report proves every finding is fixed or explicitly deferred with owner/risk, and no Critical/High release blockers remain.

**Evidence Required:** `.agent/reports/predeployment-remediation-closure-YYYYMMDD-HHMM.md` plus full command output summaries.

## Suggested Commit Sequence

1. `docs: define generated artifact and threat model policy`
2. `test: expose parser namespace and quantity regressions`
3. `fix: correct parser namespace and decimal quantity semantics`
4. `fix: confine projection output paths`
5. `fix: make protobuf projection valid and deterministic`
6. `fix: preserve calm round-trip semantics`
7. `fix: preserve kg round-trip semantics`
8. `fix: align runtime versions`
9. `fix: repair python and typescript binding contracts`
10. `fix: repair npm and wasm packaging`
11. `ci: harden release publishing and secrets`
12. `ci: add enterprise verification gates`
13. `docs: repair public claims and release playbooks`
14. `test: add resource-abuse guardrails`
15. `docs: add remediation closure report`

## Anything Easy to Forget

- Regenerate lockfiles or workflows consistently. Do not leave Bun and npm release behavior split.
- Check Windows path behavior for namespace/path traversal fixes, even if tests run on Linux.
- Validate generated Protobuf with an actual parser/compiler if available, not only string assertions.
- Verify packed artifacts from a clean temp install, not the repo working tree.
- Re-run doctests after changing public docs in Rust comments.
- Keep docs claims tied to tests or benchmarks. If there is no proof, remove or qualify the claim.
- Ensure release workflows use least-privilege permissions and pinned action versions.
- Keep all new lint scripts deterministic and runnable without network unless they are explicitly audit commands.
