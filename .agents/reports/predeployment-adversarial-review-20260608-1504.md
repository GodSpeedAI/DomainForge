# DomainForge Pre-Deployment Adversarial Review

## 1. Gate Verdict

**BLOCK RELEASE**

DomainForge is not release-ready. The repository-native `just all-tests` command fails in Rust doctests; the npm package dry-run omits the native addon required by `index.js`; multi-file Protobuf projection can write outside the requested output directory; generated TypeScript declarations are invalid; multiple projection paths are nondeterministic or lossy; release workflows can mask publish failures and expose release secrets through unverified tool downloads.

## 2. Executive Summary

Top blockers:

- `just all-tests` fails before Python/TypeScript suites run because Rust doctests do not compile.
- Published `@domainforge/sea` tarball cannot load its native binding.
- `sea project --format protobuf --multi-file` can write outside the requested output directory.
- Release secret decryption downloads and executes `sops` without checksum/signature verification.
- Projection outputs are not trustworthy: Protobuf can emit dangling/invalid types; CALM/RDF round trips lose semantics; Protobuf/CALM artifacts are nondeterministic.

Highest-risk unknowns: `cargo clippy --workspace --all-targets --all-features` and `cargo test --workspace --all-targets --all-features` were started but did not produce final results during the review window after overlapping Cargo builds. `cargo audit` could not fetch the advisory DB due an advisory lock. `npm audit` cannot run because no npm lockfile exists.

Verified: repository guidance, manifests, workflows, release scripts, README/docs, Rust/Python/TypeScript/WASM/package surfaces, parser/projection/security paths. Python and TypeScript tests passed independently. `npm run build` passed when rerun outside the sandbox after sandbox EPERM. `maturin build --release` produced a CPython 3.12 wheel.

Confidence: **High for the listed blockers**, because each has direct command output or static line evidence. Confidence is lower for full all-features Rust health and dependency vulnerability posture because the relevant commands did not complete.

## 3. Review Scope

- Repo path: `/home/sprime01/projects/domainforge`
- Commit: `630bfcdf452d56bc77da24d8a30fe1618fea3d5c`
- Branch: `dev`
- Date/time: `2026-06-08 15:04 America/New_York`
- Reviewer identity: Codex adversarial pre-deployment review plus five read-only subagents
- Environment: Linux, Rust `1.92.0`, Cargo `1.92.0`, Python `3.12.5`, Node `v24.13.0`, npm `11.6.2`, Bun `1.3.5`
- Tools available: `cargo`, `just`, `python3`, `.venv`, `npm`, `bun`, `maturin`, `wasm-pack`, `cargo-audit`; `cargo-deny` unavailable

## 4. Project Contract As Understood

DomainForge is a Rust-first semantic modeling system for the SEA DSL. The Rust `sea-core` crate is canonical. Python, TypeScript/N-API, and WASM bindings are supposed to wrap the same core behavior without duplicating business logic. The project claims cross-language parity, deterministic graph/projection behavior, controlled-language parsing, policy evaluation with three-valued logic, CALM export/import, RDF/Turtle knowledge graph output, Protobuf projection, CLI release artifacts, and publishable packages for PyPI/npm/Crates/WASM.

Project guidance in `.github/copilot-instructions.md` requires grammar-first parser changes, cross-binding parity for primitive/API changes, `IndexMap` for deterministic policy-relevant graph collections, and `just all-tests` as the preferred all-language proof command.

No root `AGENTS.md` exists on disk; the AGENTS instructions used for this review came from the user prompt.

## 5. Proof Commands Run

| Command | Result | Duration | Evidence/output summary | Status |
|---|---:|---:|---|---|
| `node ~/.codex/superpowers/.codex/superpowers-codex bootstrap` | 0 | <1s | Loaded required superpowers instructions. | Pass |
| `sed -n '1,260p' .github/copilot-instructions.md` | 0 | <1s | Confirmed Rust core canonical and cross-binding rules. | Pass |
| `sed -n '1,240p' AGENTS.md` | 2 | <1s | `No such file or directory`; root AGENTS absent. | Blocked |
| `just --list` | 0 | <1s | Listed recipes; release-doc recipes absent. | Pass |
| `cargo metadata --no-deps --format-version 1` | 0 | <1s | Workspace has `sea-core` `0.10.0`. | Pass |
| `just all-tests` | 101 | ~7m | Rust units/integration mostly passed; doctests failed in `entity.rs`, `flow.rs`, `instance.rs`, `resource.rs`; Python/TS not reached. | Fail |
| `cargo fmt --all --check` | 0 | 3s | No formatting output. | Pass |
| `just python-test` | 0 | 4s | 196 Python tests passed (`...................................................................`). | Pass |
| `just ts-test` | 0 | 4s | 167 Bun/Vitest tests passed. | Pass |
| `npm run build` | 1 | 7s | Sandbox `spawnSync /bin/sh EPERM`. | Blocked |
| `npm run build` escalated | 0 | 2m04s | N-API release build finished. | Pass |
| `maturin build --release` | 0 | 11m21s | Built `target/wheels/sea_dsl-0.10.0-cp312-cp312-manylinux_2_34_x86_64.whl`. | Pass |
| `cargo audit` | 1 | <1s | Could not obtain `/home/sprime01/.cargo/advisory-db..lock`. | Blocked |
| `npm audit --audit-level=moderate` | 1 | 5s | `ENOLOCK`; npm audit requires `package-lock.json`/shrinkwrap. | Blocked |
| `wasm-pack build --target web --release --features wasm` | 1 | 18m14s | Compiled, then failed installing `wasm-bindgen`: read-only filesystem creating temp dir. | Blocked/Fail |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | unresolved | >5m | Started; blocked/silent during overlapping Cargo builds. No pass claimed. | Blocked |
| `cargo test --workspace --all-targets --all-features` | unresolved | >5m | Started; blocked/silent during overlapping Cargo builds. No pass claimed. | Blocked |

## 6. Findings Summary

| ID | Severity | Title | Category | Affected area | Release impact | Priority |
|---|---|---|---|---|---|---|
| DF-001 | High | `just all-tests` fails in Rust doctests | Test/CI | Rust docs/API | Blocks CI-equivalent proof | P0 |
| DF-002 | Critical | Protobuf multi-file projection can write outside output directory | Security | CLI/projection | Arbitrary file write within process permissions | P0 |
| DF-003 | High | npm package omits required native addon | Packaging | npm | Published package unusable | P0 |
| DF-004 | High | TypeScript declarations are invalid | Bindings | TypeScript | Consumers fail typecheck | P0 |
| DF-005 | High | Release secret workflow executes unverified downloaded `sops` | Supply chain | Release CI | Publish-token compromise risk | P0 |
| DF-006 | High | npm publish masks failures | Release | npm release | False green releases | P0 |
| DF-007 | High | npm release uses unlocked install and lifecycle scripts with token | Supply chain | npm release | Dependency/token exfiltration risk | P0 |
| DF-008 | High | Parser corrupts role namespaces as `"in "` | Parser semantics | Roles | Wrong ConceptIds and relation semantics | P0 |
| DF-009 | High | Parser rejects same-name concepts across namespaces | Parser semantics | Entities/resources/roles | Valid models rejected | P0 |
| DF-010 | Medium | Flow DSL quantities reject decimals/large integers | Parser correctness | Flows | Valid DSL rejected / precision mismatch | P1 |
| DF-011 | High | Protobuf services emit dangling response types | Projection | Protobuf | Invalid generated `.proto` | P0 |
| DF-012 | High | Protobuf identifier sanitization can emit invalid `.proto` | Projection | Protobuf | Invalid generated `.proto` | P0 |
| DF-013 | Medium | Protobuf output is nondeterministic | Reproducibility | Protobuf | Golden/freshness checks unreliable | P1 |
| DF-014 | Medium | CALM output is nondeterministic | Reproducibility | CALM | Generated artifacts drift | P1 |
| DF-015 | High | CALM import cannot round-trip Pattern nodes | Projection semantics | CALM | Exported models fail re-import | P0 |
| DF-016 | Medium | CALM import drops exported attributes | Projection semantics | CALM | Semantic loss | P1 |
| DF-017 | Medium | RDF/Turtle import loses names, namespaces, units | Projection semantics | RDF/KG | Semantic corruption on round trip | P1 |
| DF-018 | Medium | Python `.pyi` stub is stale and wrong | Bindings | Python | Static users see false API | P1 |
| DF-019 | High | WASM package name/version/API are stale | Packaging | WASM npm | Wrong package published | P0 |
| DF-020 | Medium | WASM docs do not match generated runtime | Docs/API | WASM | Examples fail | P1 |
| DF-021 | Medium | Runtime version reports `0.1.0` while packages are `0.10.0` | Release/versioning | Rust/Python | Misleading installed version | P1 |
| DF-022 | Low | Python/Rust support docs contradict manifests | Docs | README/package metadata | Bad install guidance | P2 |
| DF-023 | Low | Python metadata advertises MIT and Apache | Licensing | PyPI metadata | License ambiguity | P2 |
| DF-024 | Medium | Release docs reference missing `just` recipes | Release docs | Maintainer workflow | Broken release playbook | P1 |
| DF-025 | Medium | Archive verification helper allows tar traversal | Security | release helper | Unsafe extraction pattern | P1 |
| DF-026 | Medium | Dependabot automerge can merge before checks exist | Supply chain | GitHub Actions | Weak dependency gate | P1 |
| DF-027 | Medium | GitHub Release can omit WASM artifact | Release | GitHub release | Incomplete release assets | P1 |
| DF-028 | Medium | Published docs contain broken links/nonexistent `Model` API | Docs | README/docs | Install/quickstart failures | P1 |

## 7. Detailed Findings

### DF-001: `just all-tests` fails in Rust doctests

- Severity: High
- Category: Test/CI
- Affected files/symbols: `sea-core/src/primitives/entity.rs` doctests, `flow.rs`, `instance.rs`, `resource.rs`
- Evidence: `just all-tests` exits 101. Output shows `test result: FAILED. 10 passed; 15 failed; 4 ignored` for doctests and `error: Recipe rust-test failed`.
- Reproduction/proof command: `just all-tests`
- Observed behavior: doctests fail with `E0308` due duplicate `serde_json` and `rust_decimal` versions in doctest context.
- Expected behavior: repository-native all-language proof command passes before release.
- Failure scenario: CI/release cannot establish project correctness; docs examples compile-fail for Rust consumers.
- Recommended fix: make doctest examples use `sea_core` re-exported types or mark non-compilable examples explicitly; add a CI job for doctests.
- Regression test: `cargo test -p sea-core --features cli --doc`.
- Residual risk: downstream users copy broken Rust docs and hit type mismatches.

### DF-002: Protobuf multi-file projection can write outside output directory

- Severity: Critical
- Category: Security / path traversal
- Affected files/symbols: `sea-core/grammar/sea.pest` `@namespace`, `sea-core/src/projection/protobuf.rs::project_multi_file`, `sea-core/src/cli/project.rs::run`
- Evidence: Namespace strings are arbitrary; `project_multi_file` converts namespace into a `PathBuf`; CLI writes `args.output.join(rel_path)`. Absolute namespace paths discard the output base.
- Reproduction/proof command:
  ```bash
  tmp=/tmp/domainforgereviewabs
  mkdir -p "$tmp/out" "$tmp/escape"
  printf '@namespace "%s/escape"\nentity "Customer"\n' "$tmp" > "$tmp/poc.sea"
  cargo run -q -p sea-core --features cli --bin sea -- project --format protobuf --multi-file "$tmp/poc.sea" "$tmp/out"
  find "$tmp" -maxdepth 3 -type f -print | sort
  ```
- Observed behavior: CLI reports success for output directory, but writes `/tmp/domainforgereviewabs/escape.proto` outside `out`.
- Expected behavior: generated files remain under requested output directory or unsafe namespaces are rejected.
- Failure scenario: untrusted `.sea` can overwrite files writable by the CLI process.
- Recommended fix: validate namespaces against package-safe grammar, reject path separators/absolute paths/traversal, and verify final normalized destination remains under output root before writing.
- Regression test: CLI tests for absolute namespace, `../escape`, Windows drive prefixes, and `a/b`, asserting failure and no outside file.
- Residual risk: arbitrary write in user workspaces and release/codegen pipelines.

### DF-003: npm package omits required native addon

- Severity: High
- Category: Packaging
- Affected files/symbols: `package.json` `files`, `index.js` native loader, `.github/workflows/release-npm.yml`
- Evidence: `npm pack --dry-run --json` lists only `LICENSE`, `README.md`, `index.d.ts`, `index.js`, `package.json`; `index.js` requires `sea-core.linux-x64-gnu.node` or `@domainforge/sea-linux-x64-gnu`.
- Reproduction/proof command: subagent packed to `/tmp`, installed in temp app, ran `node -e "require('@domainforge/sea')"`.
- Observed behavior: `Cannot find module '@domainforge/sea-linux-x64-gnu'`.
- Expected behavior: `npm install @domainforge/sea && require('@domainforge/sea')` works on supported platforms.
- Failure scenario: npm release is dead on arrival.
- Recommended fix: include generated `sea-core.*.node` files at package root or publish/declare real optional platform packages.
- Regression test: CI `npm pack`, install tarball in clean temp project, require package.
- Residual risk: broken npm artifact and support load.

### DF-004: TypeScript declarations are invalid

- Severity: High
- Category: Bindings
- Affected files/symbols: `index.d.ts` `Expression.aggregation`, `NamespaceRegistry.discover`
- Evidence: TypeScript compiler API reports 22 declaration diagnostics, including `TS1390: 'function' is not allowed as a parameter name` and `TS2304: Cannot find name 'Self'`.
- Reproduction/proof command: TypeScript compiler API against `index.d.ts` with strict checking.
- Observed behavior: declarations cannot be typechecked with `skipLibCheck: false`.
- Expected behavior: published `.d.ts` is syntactically valid and type-correct.
- Failure scenario: consumers fail builds even if runtime loads.
- Recommended fix: rename reserved parameter names and replace `Self | null` with `NamespaceRegistry | null`.
- Regression test: `tsc --noEmit --strict --skipLibCheck false` against packed declarations.
- Residual risk: TypeScript package unusable for strict consumers.

### DF-005: Release secret workflow executes unverified downloaded `sops`

- Severity: High
- Category: Supply chain / secrets
- Affected files/symbols: `.github/actions/decrypt-secrets/action.yml`, release workflows for npm/PyPI/crates
- Evidence: composite action downloads `sops` with `curl -sSLo` then runs it with `SOPS_AGE_KEY`; no checksum/signature/digest verification is present.
- Reproduction/proof command: static workflow read of `.github/actions/decrypt-secrets/action.yml`.
- Observed behavior: downloaded binary handles release decryption secrets without integrity verification.
- Expected behavior: tools that receive release secrets are pinned and verified.
- Failure scenario: compromised download asset exfiltrates Age key and publish tokens.
- Recommended fix: use pinned verified installer, SHA256/cosign verification, or trusted publishing/OIDC to avoid long-lived tokens.
- Regression test: workflow lint fails on `curl` executable download without checksum/signature validation.
- Residual risk: release artifact compromise.

### DF-006: npm publish masks failures

- Severity: High
- Category: Release engineering
- Affected files/symbols: `.github/workflows/release-npm.yml`
- Evidence: publish steps use `npm publish ... || echo ...` and `continue-on-error: true`.
- Reproduction/proof command: static workflow read.
- Observed behavior: auth, package-content, or registry failures can produce green workflow.
- Expected behavior: only already-published reruns are tolerated explicitly.
- Failure scenario: release tag appears successful while npm publish failed.
- Recommended fix: remove `continue-on-error`; explicitly check `npm view` for existing versions before publishing.
- Regression test: action test stubs `npm publish` exit 1 and asserts workflow fails for non-duplicate error.
- Residual risk: false release readiness.

### DF-007: npm release uses unlocked install and lifecycle scripts with token

- Severity: High
- Category: Supply chain
- Affected files/symbols: `.github/workflows/release-npm.yml`, `package.json`
- Evidence: workflow uses `npm install`; no `package-lock.json`/npm shrinkwrap/yarn/pnpm lockfile exists; `package.json` has ranged dev deps and `prepublishOnly`; `npm publish` runs with `NODE_AUTH_TOKEN`.
- Reproduction/proof command: `ls package-lock.json npm-shrinkwrap.json yarn.lock pnpm-lock.yaml 2>/dev/null || true` produced no output.
- Observed behavior: release resolves npm dependencies at publish time; lifecycle scripts can run with token.
- Expected behavior: frozen reproducible install, build before token exposure, publish without lifecycle scripts when possible.
- Failure scenario: compromised build dependency steals npm token or modifies package contents.
- Recommended fix: use committed lockfile with `npm ci` or `bun install --frozen-lockfile`; set token only for publish; use `npm publish --ignore-scripts` if feasible.
- Regression test: workflow lint disallows `npm install` in release without lockfile/frozen mode and lifecycle policy.
- Residual risk: supply-chain compromise.

### DF-008: Parser corrupts role namespaces as `"in "`

- Severity: High
- Category: Parser semantics
- Affected files/symbols: `sea-core/grammar/sea.pest`, `sea-core/src/parser/ast.rs::parse_role`
- Evidence: `Role "Approver" in governance` parse JSON has `"namespace": "in "`.
- Reproduction/proof command: `target/debug/sea parse --format json /tmp/domainforge-role.sea`
- Observed behavior: namespace is `"in "`.
- Expected behavior: namespace/domain is `governance`.
- Failure scenario: role `ConceptId`s and role/relation semantics are wrong.
- Recommended fix: skip `Rule::in_keyword` and consume following identifier, mirroring entity/resource parsing.
- Regression test: parse role with explicit domain and assert AST/graph namespace.
- Residual risk: semantic corruption in governance/authority models.

### DF-009: Parser rejects same-name concepts across namespaces

- Severity: High
- Category: Parser semantics
- Affected files/symbols: `sea-core/src/parser/ast.rs::ast_to_graph_with_options`
- Evidence: name-only `HashMap`s reject `Entity "Customer" in sales` plus `Entity "Customer" in support`; docs say duplicates fail in the same namespace.
- Reproduction/proof command: `target/debug/sea parse --format json /tmp/domainforge-duplicate-ns.sea`
- Observed behavior: `Duplicate declaration: Entity 'Customer' already declared at 0:0`.
- Expected behavior: namespace-distinct concepts are valid; duplicates only fail within same namespace.
- Failure scenario: valid enterprise models cannot represent same business name in distinct namespaces.
- Recommended fix: key symbol tables by `(namespace, name)` and define unqualified reference resolution/ambiguity rules.
- Regression test: same name across two namespaces succeeds; same namespace duplicate fails.
- Residual risk: false invalid models and broken namespace contract.

### DF-010: Flow DSL quantities reject decimals/large integers

- Severity: Medium
- Category: Parser correctness
- Affected files/symbols: `sea-core/src/parser/ast.rs::parse_flow`, `parse_number`, `sea-core/src/primitives/flow.rs`
- Evidence: grammar allows decimal numbers and core `Flow.quantity` is `Decimal`, but parser parses `i32`.
- Reproduction/proof command: `target/debug/sea parse --ast --format json /tmp/domainforge-decimal-flow.sea`
- Observed behavior: `Invalid quantity: Invalid number: 1.5`; `2147483648` also fails.
- Expected behavior: decimal/large flow quantities parse losslessly into `Decimal`.
- Failure scenario: valid quantities in DSL are rejected.
- Recommended fix: change AST flow quantity to `Option<Decimal>` and parse with decimal parser.
- Regression test: `quantity 1.5` and `quantity 2147483648` parse and preserve value.
- Residual risk: precision/unit model undermined.

### DF-011: Protobuf services emit dangling response types

- Severity: High
- Category: Projection
- Affected files/symbols: `sea-core/src/projection/protobuf.rs::flows_to_services`
- Evidence: `--include-services` emits `returns (CameraResponse);` but no `message CameraResponse`.
- Reproduction/proof command: `cargo run -p sea-core --features cli -- project --format protobuf --include-services examples/namespaces/logistics/core.sea /tmp/domainforge-review-services.proto`
- Observed behavior: output has `Camera`, `Factory`, `Warehouse` messages and service returns undefined `CameraResponse`.
- Expected behavior: every RPC request/response type is defined or imported.
- Failure scenario: generated proto cannot compile in standard protobuf toolchains.
- Recommended fix: generate deterministic response messages or use existing/WKT response type consistently.
- Regression test: assert all service method types resolve locally, to WKT, or to imports.
- Residual risk: unusable service projections.

### DF-012: Protobuf identifier sanitization can emit invalid `.proto`

- Severity: High
- Category: Projection
- Affected files/symbols: `sea-core/src/projection/protobuf.rs::to_pascal_case`
- Evidence: `Entity "123 Customer"` generates `message 123Customer {`.
- Reproduction/proof command:
  ```bash
  printf 'Entity "123 Customer"\nEntity "class"\nResource "Money" USD\n' > /tmp/domainforge-review-invalid-ident.sea
  cargo run -p sea-core --features cli -- project --format protobuf /tmp/domainforge-review-invalid-ident.sea /tmp/domainforge-review-invalid-ident.proto
  ```
- Observed behavior: invalid Protobuf identifier.
- Expected behavior: generated identifiers obey Protobuf lexical/reserved-word rules or projection fails with diagnostic.
- Failure scenario: malformed generated code enters repos/CI.
- Recommended fix: centralize Protobuf identifier validation/sanitization for packages, messages, fields, enums, services, methods.
- Regression test: invalid names fail or emit valid prefixed identifiers.
- Residual risk: invalid external contracts.

### DF-013: Protobuf output is nondeterministic

- Severity: Medium
- Category: Reproducibility
- Affected files/symbols: `sea-core/src/projection/protobuf.rs::project`, `ProtoFile::to_proto_string`
- Evidence: generated artifacts include `generated_at = Utc::now()`.
- Reproduction/proof command: run same Protobuf projection twice, then `diff -u`.
- Observed behavior: only `// Generated At: ...` differs; `cmp_exit=1`.
- Expected behavior: deterministic projection mode for freshness/golden checks.
- Recommended fix: omit timestamps by default or honor `SOURCE_DATE_EPOCH`/fixed timestamp.
- Regression test: same input projected twice is byte-identical in deterministic mode.
- Residual risk: stale-artifact detection impossible.

### DF-014: CALM output is nondeterministic

- Severity: Medium
- Category: Reproducibility
- Affected files/symbols: `sea-core/src/calm/export.rs::export`, `sea-core/src/primitives/flow.rs::Flow::new_with_namespace`
- Evidence: CALM output changes `sea:timestamp` and Flow relationship UUID v4 across identical exports.
- Reproduction/proof command: run `sea project --format calm` twice for `examples/namespaces/logistics/core.sea`, then `diff -u`.
- Observed behavior: timestamp and relationship `unique-id` differ.
- Expected behavior: generated CALM artifacts reproducible for identical semantic input, or volatile fields opt-in.
- Recommended fix: deterministic IDs for parsed declarative flows and configurable timestamp injection.
- Regression test: same CALM projection twice byte-identical in deterministic mode.
- Residual risk: drift in generated artifacts and audit trails.

### DF-015: CALM import cannot round-trip Pattern nodes

- Severity: High
- Category: Projection semantics
- Affected files/symbols: `sea-core/src/calm/export.rs::export_pattern`, `sea-core/src/calm/import.rs::import_constraint_node`
- Evidence: exporter writes Pattern as `node-type: constraint` with `sea:primitive = Pattern`; importer treats every `constraint` as Policy and requires `sea:expression`.
- Reproduction/proof command: static proof from cited code paths.
- Observed behavior: exported Pattern node lacks required Policy field for importer.
- Expected behavior: importer dispatches on `sea:primitive` and preserves Pattern/Metric/Policy distinctly.
- Recommended fix: inspect `sea:primitive`; import Pattern via `sea:regex`, Metric via metric metadata.
- Regression test: Graph with one Pattern exports to CALM and imports preserving count/name/regex.
- Residual risk: CALM round-trip fails for advertised primitives.

### DF-016: CALM import drops exported attributes

- Severity: Medium
- Category: Projection semantics
- Affected files/symbols: `sea-core/src/calm/export.rs::export_entity/export_resource`, `sea-core/src/calm/import.rs::import_entity/import_resource`
- Evidence: exporter inserts `sea:attributes`; importer reconstructs entities/resources from name/namespace/unit only.
- Reproduction/proof command: static proof from code paths.
- Observed behavior: attributes are not read on import.
- Expected behavior: CALM round-trip preserves primitive attributes.
- Recommended fix: deserialize `sea:attributes` and reapply via `set_attribute`.
- Regression test: entity/resource attributes round-trip with scalar/array/object values.
- Residual risk: silent semantic loss.

### DF-017: RDF/Turtle import loses names, namespaces, units

- Severity: Medium
- Category: Projection semantics
- Affected files/symbols: `sea-core/src/kg.rs::KnowledgeGraph::to_graph`
- Evidence: export percent-encodes subjects and writes namespace/unit triples; import splits on `:` without percent-decoding, hardcodes namespace `"default"` and unit `"units"`.
- Reproduction/proof command: static proof from `kg.rs` line reads.
- Observed behavior: encoded local names, namespaces, and units are not restored.
- Expected behavior: KG round-trip preserves local names, namespaces, and units.
- Recommended fix: percent-decode local names and read `sea:namespace`/`sea:unit` triples before constructing graph.
- Regression test: round-trip `Entity "Central Warehouse" in logistics` and `Resource "Money" USD`.
- Residual risk: semantic corruption in RDF pipelines.

### DF-018: Python `.pyi` stub is stale and wrong

- Severity: Medium
- Category: Bindings
- Affected files/symbols: `python/sea_dsl/sea_dsl.pyi`, `python/sea_dsl/__init__.py`
- Evidence: runtime `__all__` includes APIs missing from stub; stub types `Instance` like resource/entity-location while runtime `Instance('order_123','Order')` exposes name/entity_type and lacks resource_id/entity_id.
- Reproduction/proof command: `PYTHONPATH=python python -c ...` runtime/stub comparison.
- Observed behavior: missing symbols and wrong type shape.
- Expected behavior: `.pyi` mirrors public runtime exports.
- Recommended fix: regenerate/update stub from PyO3 exports.
- Regression test: compare `sea_dsl.__all__` against stub declarations and run type checker sample importing all public symbols.
- Residual risk: Python static users receive incorrect API contract.

### DF-019: WASM package name/version/API are stale

- Severity: High
- Category: Packaging
- Affected files/symbols: `scripts/build-wasm.sh`, `pkg/package.json`, `sea-core/pkg/package.json`, `README_WASM.md`
- Evidence: `npm pack --dry-run` in `pkg/` reports `@sprime01/sea-wasm@0.4.0`; docs advertise `@domainforge/sea-wasm`; repo version is `0.10.0`; root `pkg` lacks newer `Graph.parseToAstJson`.
- Reproduction/proof command: `npm pack --dry-run` in `pkg/`; WASM runtime smoke of root `pkg` vs `sea-core/pkg`.
- Observed behavior: stale/wrong package target.
- Expected behavior: one canonical WASM package with documented name/version/API.
- Recommended fix: publish from one generated dir with correct metadata; remove stale checked-in `pkg` or update build pipeline.
- Regression test: CI pack dry-run asserts name/version and documented exports.
- Residual risk: wrong WASM package published.

### DF-020: WASM docs do not match generated runtime

- Severity: Medium
- Category: Documentation/API
- Affected files/symbols: `README_WASM.md`, `docs/reference/wasm-api.md`
- Evidence: docs say `await init()` works in Node and call `cameras.id()`/`Entity.new(...)`; generated web target fails Node `await init()` via `fetch(file://...)`, exposes `id` as property, and has no `Entity.new`.
- Reproduction/proof command: Node ESM smoke tests by subagent.
- Observed behavior: `TypeError: fetch failed`; `id` is string property; `Entity.new undefined`.
- Expected behavior: docs match generated WASM API and target behavior.
- Recommended fix: document explicit WASM byte init or ship Node target; replace Rust-style methods with JS property API.
- Regression test: executable README smoke snippets for browser and Node.
- Residual risk: users cannot follow install docs.

### DF-021: Runtime version reports `0.1.0` while packages are `0.10.0`

- Severity: Medium
- Category: Versioning
- Affected files/symbols: `sea-core/src/lib.rs::VERSION`, `pyproject.toml`, `package.json`
- Evidence: `PYTHONPATH=python python3 -c "import sea_dsl; print(sea_dsl.__version__)"` prints `0.1.0`; manifests are `0.10.0`.
- Reproduction/proof command: version comparison script from subagent.
- Observed behavior: runtime/core version stale.
- Expected behavior: runtime version equals package version.
- Recommended fix: `pub const VERSION: &str = env!("CARGO_PKG_VERSION");` and update binding tests.
- Regression test: Rust/Python tests assert runtime versions match manifest/importlib metadata.
- Residual risk: support/debugging and compatibility checks misidentify releases.

### DF-022: Python/Rust support docs contradict manifests

- Severity: Low
- Category: Documentation
- Affected files/symbols: `README.md`, `README_PYTHON.md`, `pyproject.toml`, `sea-core/Cargo.toml`
- Evidence: README badges/docs advertise Rust 1.75+ and Python 3.8+/3.9+; manifests require Rust 1.77 and Python >=3.11.
- Reproduction/proof command: static `rg` over README/manifests.
- Observed behavior: docs promise unsupported versions.
- Expected behavior: docs match package metadata and CI matrix.
- Recommended fix: update docs/badges or lower manifest requirements after testing.
- Regression test: docs lint compares requirements to manifests.
- Residual risk: failed installs for documented users.

### DF-023: Python metadata advertises MIT and Apache

- Severity: Low
- Category: Licensing
- Affected files/symbols: `pyproject.toml`
- Evidence: `license = Apache-2.0`, classifiers include both MIT and Apache.
- Reproduction/proof command: static manifest read.
- Observed behavior: PyPI metadata implies MIT license not present in repo license.
- Expected behavior: one accurate license declaration.
- Recommended fix: remove MIT classifier.
- Regression test: metadata lint asserts classifiers match SPDX/license file.
- Residual risk: legal/commercial ambiguity.

### DF-024: Release docs reference missing `just` recipes

- Severity: Medium
- Category: Release engineering
- Affected files/symbols: `docs/RELEASE_PROCESS.md`, `docs/playbooks/local-release-preparation.md`, `justfile`
- Evidence: docs reference `just release-preview`, `prepare-release`, `ci-pipeline`, `changelog-entry`, `test-changelog-logic`; `just --list` shows none.
- Reproduction/proof command: `just --list`; `rg` for recipe definitions.
- Observed behavior: documented release commands fail.
- Expected behavior: playbooks use existing commands or recipes exist.
- Recommended fix: add recipes or update docs to scripts.
- Regression test: extract `just` commands from release docs and assert recipes exist.
- Residual risk: maintainers follow broken release process.

### DF-025: Archive verification helper allows tar traversal

- Severity: Medium
- Category: Security
- Affected files/symbols: `scripts/ci_tasks.py::unpack_and_verify`, `just ci-verify-package`, `.github/workflows/release.yml`
- Evidence: helper uses `tarfile.extractall(temp_dir)` without member filtering.
- Reproduction/proof command: Python tar with `../outside.txt` then `extractall`.
- Observed behavior: `outside_exists True`, `outside_content owned` on Python 3.12.5.
- Expected behavior: archive verification never writes outside extraction root.
- Recommended fix: use `filter="data"` and explicit path/symlink/hardlink validation for tar and zip.
- Regression test: malicious tar fixture asserts nonzero exit and no outside file.
- Residual risk: unsafe archive verification pattern in CI/release tooling.

### DF-026: Dependabot automerge can merge before checks exist

- Severity: Medium
- Category: Supply chain
- Affected files/symbols: `.github/workflows/dependabot-automerge.yml`
- Evidence: workflow sleeps 10s, treats `gh pr checks` returning `[]` as success, and only detects completed failures.
- Reproduction/proof command: static workflow read.
- Observed behavior: missing/pending checks can pass gate.
- Expected behavior: known required checks present and successful before merge.
- Recommended fix: rely on branch protection plus `gh pr merge --auto`, or poll expected check set and fail on missing/pending/cancelled/skipped/timed out.
- Regression test: mocked `gh pr checks` `[]` and pending states fail gate.
- Residual risk: dependency updates can merge before CI.

### DF-027: GitHub Release can omit WASM artifact

- Severity: Medium
- Category: Release engineering
- Affected files/symbols: `.github/workflows/release.yml`
- Evidence: `build-wasm-release` uploads `sea-core-wasm`, but `create-release.needs` excludes `build-wasm-release`.
- Reproduction/proof command: static workflow read.
- Observed behavior: release creation can download artifacts before WASM job finishes.
- Expected behavior: release waits for every artifact-producing job.
- Recommended fix: add `build-wasm-release` to `create-release.needs`.
- Regression test: workflow lint asserts every upload-artifact job is included in release needs.
- Residual risk: incomplete GitHub releases.

### DF-028: Published docs contain broken links and nonexistent `Model` API

- Severity: Medium
- Category: Documentation/API
- Affected files/symbols: `README.md`, `README_PYTHON.md`, `README_TYPESCRIPT.md`, `README_WASM.md`
- Evidence: README examples use `from sea import Model`/`Model(...)`; no `sea` module or `Model` export exists. Path checks for several documented docs links return false.
- Reproduction/proof command:
  ```bash
  PYTHONPATH=python python3 -c "from sea import Model"
  PYTHONPATH=python python3 -c "import sea_dsl; print(hasattr(sea_dsl, 'Model'))"
  ```
- Observed behavior: `ModuleNotFoundError`; `False`.
- Expected behavior: registry-facing quickstarts use supported `sea_dsl.Graph`/binding APIs and links target existing docs.
- Recommended fix: replace Model examples with real Graph examples and update links.
- Regression test: markdown link checker plus smoke tests for README code blocks.
- Residual risk: first-run user experience fails.

## 8. Domain Semantics and Projection Integrity

- `.sea` model behavior: role namespace parsing corrupts explicit role domains; same-name namespace-distinct declarations are rejected; decimal flow quantities are rejected despite grammar/core Decimal support.
- AST/JSON consistency: AST can preserve namespace-distinct duplicate declarations while graph conversion rejects them, proving AST/graph semantic drift.
- Protobuf consistency: service generation emits undefined response types; identifiers can be syntactically invalid; output includes volatile timestamp.
- RDF/Turtle consistency: export writes namespace/unit/encoded names, but import does not restore them.
- CALM consistency: Pattern nodes cannot re-import, primitive attributes are dropped, timestamp and UUID v4 make output nondeterministic.
- Python/TypeScript/Rust/WASM consistency: TypeScript `.d.ts` invalid; Python stub omits/mistypes runtime APIs; WASM package/docs are stale; npm package lacks native addon.
- Generated artifact freshness: root `pkg` is stale relative to `sea-core/pkg`; checked-in binary/package artifacts and `dist/0.8.1` artifacts exist in the repo while manifests are `0.10.0`.
- Determinism/reproducibility: Protobuf and CALM projections are byte-nondeterministic for identical inputs.

## 9. Security and Threat Model

Trust boundaries:

- Untrusted `.sea` source files feed parser, module resolver, projections, package build pipelines, and CLI file writes.
- Release workflows handle npm/PyPI/Cargo tokens and SOPS Age key.
- Generated archives and npm packages are consumed by external users.

Untrusted input risks:

- Protobuf multi-file output trusts namespace strings as paths.
- Module/registry code follows symlinks during namespace file resolution (`follow_links(true)`), requiring careful workspace boundary policy.
- Archive verification uses unsafe `tarfile.extractall`.

File/network/process surfaces:

- CLI reads arbitrary input paths and writes user-provided output paths.
- Release action downloads and executes a binary from the network before decrypting secrets.
- npm release runs install/publish lifecycle with dependency resolution and token environment.

Dependency/supply-chain risk:

- `cargo audit` blocked by advisory lock; no result.
- `cargo-deny` unavailable.
- `npm audit` blocked because npm lockfile is absent; repo uses `bun.lock`, but release workflow uses `npm install`.
- Dependabot automerge can approve/merge before checks exist.

Abuse cases:

- Malicious SEA namespace writes generated files outside output root.
- Compromised `sops` binary exfiltrates publish secrets.
- Malicious npm build dependency resolved during release exfiltrates `NODE_AUTH_TOKEN`.
- Malicious tar archive writes outside verification temp dir.

## 10. Release Engineering Review

- CI coverage gap: `just all-tests` fails locally; full all-features clippy/test did not complete in review window.
- npm package risk: root package does not include or declare the native binding it loads.
- WASM release risk: package name/version/API are stale; GitHub release can omit WASM artifact due missing dependency.
- Versioning risk: runtime reports `0.1.0`, manifests `0.10.0`; stale `release-0.7.0.sh` and docs reference 0.7.0/0.8.1 artifacts.
- Publishing risk: npm publish failure masked; release secrets handled by unverified downloaded binary.
- Reproducibility gap: Protobuf/CALM outputs include volatile fields; release npm install is not frozen against npm lockfile.
- Documentation/install proof: README quickstart imports nonexistent `sea.Model`; docs point to missing files/recipes.

## 11. Test Coverage and Evidence Gaps

Missing/weak tests:

- Packed npm install/require smoke test.
- TypeScript declaration `tsc --skipLibCheck false`.
- Python stub/runtime parity test.
- WASM package metadata/export smoke test.
- Protobuf generated type resolution and lexical validation tests.
- Deterministic Protobuf/CALM golden tests.
- CALM Pattern/attribute round-trip tests.
- RDF/KG namespace/unit/percent-decoding round-trip tests.
- Parser namespace duplicate and role namespace regression tests.
- Decimal/large flow quantity parser tests.
- Release workflow static lints for token/lifecycle/failure masking.
- Archive traversal tests.
- Dependency gate tests for Dependabot.

Commands not settled:

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: unresolved during overlapping Cargo builds.
- `cargo test --workspace --all-targets --all-features`: unresolved during overlapping Cargo builds.
- `cargo audit`: blocked by advisory DB lock.
- `cargo deny check`: tool unavailable.
- `npm audit`: blocked by missing npm lockfile.
- `wasm-pack build`: failed after compilation due read-only filesystem when installing `wasm-bindgen`.

## 12. Documentation and Claims Audit

Proven:

- Rust/Python/TypeScript APIs exist and have passing independent Python/TS test suites.
- CALM, KG, Protobuf, policy, semantic-pack modules exist.
- `cargo fmt --all --check` passes.
- `maturin build --release` can build a local CPython 3.12 wheel.

Partially proven:

- Cross-language parity: some Python/TS tests pass, but stubs/declarations/packages are broken and `just all-tests` fails.
- CALM/RDF/Protobuf support: exporters exist, but round-trip and generated validity defects are present.
- WASM support: source bindings exist, but package/docs are stale and local `wasm-pack build` failed.

Unproven:

- “Always Correct. Always in Sync.”
- “Formal mathematical rigor” / “provable correctness.”
- “10,000 entities validated in under 100 milliseconds.”
- “Pre-built packages for PyPI, npm, and Crates.io. No compilation required.”
- WASM `<500KB gzipped` claim for current package.
- Same version compatibility across bindings.

Contradicted:

- README Python quickstart `from sea import Model` contradicted by `ModuleNotFoundError`.
- README version support contradicted by manifests.
- npm install claim contradicted by packed tarball missing native addon.
- WASM package docs contradicted by stale `pkg/package.json` and Node init/API behavior.
- Release process docs contradicted by missing `just` recipes.

## 13. Recommended Fix Plan

P0 release blockers:

- Fix `just all-tests` doctests. Proof required: `just all-tests` exits 0 from clean workspace.
- Fix npm package native addon packaging. Proof required: `npm pack`, install tarball in temp project, `require('@domainforge/sea')`.
- Block Protobuf path traversal. Proof required: absolute/traversal namespace tests fail safely with no outside files.
- Verify release secret tooling. Proof required: checksum/signature validation or OIDC trusted publishing; workflow lint passes.
- Stop masking npm publish failures and freeze install. Proof required: workflow test with failing publish fails; frozen install uses committed lockfile or `bun.lock`.
- Fix parser role namespace and namespace-distinct duplicates. Proof required: parser regression tests pass.
- Fix invalid/dangling Protobuf output. Proof required: generated `.proto` validates with standard toolchain or resolver test.
- Fix invalid TypeScript declarations. Proof required: strict `tsc` against packed package declarations.
- Fix WASM package target. Proof required: pack dry-run shows `@domainforge/sea-wasm@0.10.0` and documented exports smoke-test.

P1 high-risk fixes:

- Add deterministic projection mode for CALM/Protobuf.
- Preserve CALM Pattern/attributes and RDF names/namespaces/units.
- Regenerate Python stubs and add parity test.
- Fix runtime version constant.
- Fix release docs and broken README links/code blocks.
- Add archive traversal protection.
- Fix GitHub Release `needs` and Dependabot checks gate.

P2 hardening:

- Add cargo audit/deny to CI and resolve local advisory DB lock issue.
- Add markdown link/code-block smoke tests.
- Remove stale release artifacts, old scripts, and scratch/log files from the tracked release surface if tracked.
- Add fuzz/property tests for parser malformed input, namespace resolution, and projections.

P3 cleanup:

- Align docs badges with manifests.
- Remove MIT classifier.
- Clarify generated artifact policy and whether checked-in `pkg`, `dist`, native `.node` files are source-of-truth or build outputs.

## 14. Follow-Up Verification Plan

Required before deployment:

```bash
git clean -ndx
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test -p sea-core --features cli --doc
just all-tests
cargo audit
cargo deny check
npm run build
npm pack --dry-run --json
tmp=$(mktemp -d); npm pack --pack-destination "$tmp"; (cd "$tmp" && npm init -y >/dev/null && npm install ./*.tgz && node -e "require('@domainforge/sea')")
npx tsc --noEmit --strict --skipLibCheck false
just python-test
PYTHONPATH=python python - <<'PY'
import sea_dsl
assert sea_dsl.__version__ == "0.10.0"
PY
wasm-pack build --target web --release --features wasm
maturin build --release
```

Add targeted regression commands/tests for every P0/P1 finding before retagging.

## 15. Appendix

Notable grep/static observations:

- `rg` found production `TODO` in `cli/validate.rs` and module import diagnostics, extensive `unwrap` in tests/docs, file read/write surfaces in CLI project/pack/format/parse/import/authority paths.
- `find` showed existing built artifacts/logs: `index.node`, `sea-core.linux-x64-gnu.node`, `sea_core.node`, `dist/sea-0.8.1...`, `dist/sea_dsl-0.8.1...`, `.logs/subtask2.log`, `tmp/wasm_test_local*.log`, `fix_tests.py`, `fix_unit_new.py`.
- `package.json` version is `0.10.0`; `pkg/package.json` is `@sprime01/sea-wasm@0.4.0`; `sea-core/pkg/package.json` is `sea-core@0.10.0`.

Files inspected included:

- `.github/copilot-instructions.md`, workflows under `.github/workflows`, `.github/actions/decrypt-secrets/action.yml`
- `README.md`, `README_PYTHON.md`, `README_TYPESCRIPT.md`, `README_WASM.md`, `CONTRIBUTING.md`, release docs, grammar docs
- `Cargo.toml`, `sea-core/Cargo.toml`, `pyproject.toml`, `package.json`, `pkg/package.json`, `sea-core/pkg/package.json`, `justfile`
- `sea-core/src/parser/*`, `registry/mod.rs`, `module/resolver.rs`, `cli/project.rs`, `projection/protobuf.rs`, `calm/export.rs`, `calm/import.rs`, `kg.rs`, binding modules
- Python tests, TypeScript tests, Rust integration tests inventory

Subagent summaries:

- Rust/parser agent found role namespace corruption, namespace-distinct duplicate rejection, and `i32` flow quantity parsing.
- Projection/codegen agent found dangling Protobuf service types, invalid identifiers, nondeterministic Protobuf/CALM, CALM Pattern/attribute loss, and RDF/KG semantic loss.
- Bindings agent found broken npm package, invalid `.d.ts`, stale Python stub, stale WASM package, and mismatched WASM docs/API.
- Security/supply-chain agent found Protobuf path traversal, unsafe tar extraction, unverified `sops`, npm lifecycle/token risk, and Dependabot automerge weakness.
- CI/release/docs agent found broken npm tarball, masked npm publish, missing WASM release dependency, stale runtime version, inaccurate support docs, license classifier mismatch, missing release recipes, and broken docs/API claims.

Answers to unanswered questions:

  1. Generated artifacts policy

     Treat checked-in pkg, sea-core/pkg, native .node, dist, wheels, tarballs, and old release artifacts as build outputs, not source
     truth. Do not commit them except for narrowly justified fixtures/goldens.

     Source truth should be Rust source, schemas, manifests, lockfiles, docs, examples, and tests. Release artifacts should be produced by
     CI from a clean checkout, packed, smoke-tested, and uploaded.

  2. Authoritative JavaScript package manager

     Pick Bun for development/test install if that is the project preference, but use a release workflow that is internally consistent.

     Best answer for release: either:
      - commit and use package-lock.json with npm ci for npm publishing, or
      - switch release workflows to bun install --frozen-lockfile using committed bun.lock.

     Do not use bun.lock locally while release CI runs unfrozen npm install.

  3. Projection determinism

     Projection outputs should default to deterministic mode. Volatile metadata, timestamps, random UUIDs, and machine-local paths should
     be opt-in flags or external release metadata.

     This project promises executable domain meaning and cross-format consistency. That requires byte-stable projections for the same
     semantic input, especially for CALM, RDF/Turtle, Protobuf, generated bindings, golden tests, and artifact freshness checks.

  4. Threat model for untrusted .sea inputs

     Treat .sea files as untrusted input in CLI, CI, codegen, registry, import, and projection paths.

     Required policy:
      - no generated writes outside explicit output roots
      - no path traversal through namespaces/imports/includes
      - no symlink escape unless explicitly allowed
      - bounded parsing and projection resource use
      - deterministic diagnostics for malformed input
      - no process execution from model content
      - no network access from model content
      - safe archive extraction and package verification in release tooling

  Given DomainForge’s intended role as “executable domain meaning infrastructure,” the bias should be conservative: source-controlled
  semantic inputs may be authored by humans, but every parser, resolver, projector, and release job must handle them as hostile until
  validated.