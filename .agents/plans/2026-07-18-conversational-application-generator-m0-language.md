# SEA Application Contract Milestone 0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` to implement this plan task by task.
> Steps use checkbox (`- [ ]`) syntax for tracking. Use
> `test-driven-development`, `incremental-implementation`, and
> `code-review-and-quality` at every packet. Stop at the Milestone 0 gate; this
> plan does not authorize generator or adapter work.

**Goal:** Implement accepted ADR-013 from SEA source through one resolved,
validated, canonical Application Contract and expose byte-identical contract
JSON through Rust, Python, TypeScript, and WASM.

**Architecture:** Extend the grammar and partial AST first, then replace the
resolver's validation-only output with one immutable `ResolvedModuleSet` used
by both Graph and the new `application` module. Resolve and validate the closed
v0.1 type/operation model before canonicalizing it into strict documents.
Bindings remain thin wrappers over one Rust entry point.

**Tech stack:** Rust 1.77, Pest, Serde, Schemars, `IndexMap`,
`rust_decimal`, `chrono`, `unicode-normalization`, SHA-256, PyO3, napi-rs,
wasm-bindgen, Python pytest, TypeScript Vitest.

## Global constraints

- Normative authorities are accepted
  `docs/specs/ADR-013-sea-application-contract.md` and
  `docs/reference/sea-application-contract.md`. Where this plan and either
  accepted document differ, stop and repair the plan before coding.
- Start syntax changes in `domainforge-core/grammar/sea.pest`, then update the
  internal AST, serialized AST v3, conversions, both printers, resolver,
  application model, Graph construction, and bindings in that order.
- Preserve existing `ConceptId` exact-byte, case-sensitive identity. Apply NFC
  only to new application declaration names, authored application strings, and
  the canonical forms named by the reference.
- Keep `ResolvedModuleSet` crate-private. Graph and `ApplicationContract` must
  consume the same resolved symbols; neither may resolve text independently.
- Use `IndexMap` for policy-relevant or serialized ordered maps. Reject duplicate
  JSON keys before converting `sources_json` into a map.
- Public document structs use `#[serde(deny_unknown_fields)]`; public enums use
  adjacent tagging `#[serde(tag = "kind", content = "data",
  rename_all = "snake_case")]`; newtype IDs use `#[serde(transparent)]`.
- Preserve the AST v3 top-level `{metadata,declarations}` shape and byte-stable
  existing AST output. A non-additive result stops implementation and invokes
  ADR-013's ast-v4 contingency.
- Keep application normalization separate from semantic-pack meaning. The
  existing `semantic_pack::canonical_json` may serialize bytes, but application
  hashes must never feed pack hashes or signatures.
- Every behavior packet follows RED → GREEN → refactor. A RED command must fail
  for the stated missing behavior, never from a broken fixture or compile error
  unrelated to that packet.
- Stage only paths listed in the packet. Inspect `git status --short` before
  every commit. Leave `.agents/`, `.gitignore`, `devbox.lock`, and unrelated
  user work unstaged.
- Run commands from the repository root. Test commands name one test target or
  one exact test; do not pass shell globs or directory-wide fixture arguments.
- Milestone 0 excludes Adapter IR, `PortIr`, providers, generation, proof
  runners, Axum, SQLite, approval UX, skills, and runtime execution of D9
  behavioral vectors. It proves that the contract encodes those vectors.

## Fixed interfaces and ownership

The following names are settled for all packets. Do not rename or duplicate
them while implementing a packet.

```rust
// domainforge-core/src/module/resolver.rs — crate-private
pub(crate) struct SourceMap(IndexMap<String, String>);
pub(crate) struct ResolvedModuleSet { /* exact fields from reference §8 */ }
pub(crate) fn resolve_source_map(
    entry_logical_path: &str,
    sources: &SourceMap,
) -> Result<ResolvedModuleSet, Vec<ApplicationDiagnostic>>;

// domainforge-core/src/application/resolve.rs — public Rust boundary
pub fn resolve_application_contract(
    entry_logical_path: &str,
    sources_json: &str,
) -> Result<ApplicationContractDocument, Vec<ApplicationDiagnostic>>;
pub fn resolve_application_contract_json(
    entry_logical_path: &str,
    sources_json: &str,
) -> Result<String, Vec<ApplicationDiagnostic>>;

// domainforge-core/src/application/envelope.rs — public Rust boundary
pub fn build_canonical_semantic_envelope(
    resolved: &ResolvedModuleSet,
    contract: &ApplicationContract,
    semantic_packs: &[CanonicalSemanticPackRef],
) -> Result<CanonicalSemanticEnvelopeDocument, Vec<ApplicationDiagnostic>>;
```

`SourceMap::parse_json` must reject non-object input, duplicate raw keys,
duplicate normalized logical IDs, non-string values, invalid logical paths,
and a missing entry. Its logical-path algorithm is exactly reference §7.
Native registry resolution remains available through `ModuleResolver`; it must
adapt filesystem sources into the same `SourceMap`/`ResolvedModuleSet` engine.

Public binding methods are fixed:

```text
Python:     Graph.resolve_application_contract_json(entry_logical_path: str, sources_json: str) -> str
TypeScript: Graph.resolveApplicationContractJson(entryLogicalPath: string, sourcesJson: string) -> string
WASM:       Graph.resolveApplicationContractJson(entryLogicalPath: string, sourcesJson: string) -> string
```

TypeScript also adds the missing
`Graph.parseToAstJson(source: string): string`. All binding failures contain
the canonical compact JSON array of `ApplicationDiagnostic`; bindings do not
reclassify diagnostics.

## File responsibility map

| Path | Responsibility |
|---|---|
| `domainforge-core/grammar/sea.pest` | Accepted contextual syntax only. |
| `domainforge-core/src/parser/ast.rs` | Partial authored AST with spans and source-order operation clauses. |
| `domainforge-core/src/parser/ast_schema.rs` | Additive public AST v3 wire types. |
| `domainforge-core/src/parser/ast_convert.rs` | Exhaustive internal-to-schema conversion. |
| `domainforge-core/src/parser/printer.rs` | `PrettyPrinter` rendering. |
| `domainforge-core/src/formatter/printer.rs` | Canonical source formatting and round-trip form. |
| `domainforge-core/src/module/resolver.rs` | Source map, import graph, visibility, aliases, origins, symbol table. |
| `domainforge-core/src/application/contract.rs` | Exact public contract types from reference §8. |
| `domainforge-core/src/application/resolve.rs` | AST-to-contract reference/type resolution and public entry point. |
| `domainforge-core/src/application/validate.rs` | APP001–APP014 invariant checks and lowering rules. |
| `domainforge-core/src/application/policy_context.rs` | Closed policy subset validation and fail-closed evaluation. |
| `domainforge-core/src/application/canonical.rs` | Typed normalization and all non-envelope hashes. |
| `domainforge-core/src/application/envelope.rs` | Canonical semantic envelope and exhaustive payload conversion. |
| `domainforge-core/src/application/diagnostic.rs` | Stable APP diagnostic wire model. |
| `schemas/application-contract-v1.schema.json` | Strict persisted contract document schema. |
| `schemas/canonical-semantic-envelope-v1.schema.json` | Strict persisted envelope document schema. |

---

### Task 1: Capture compatibility oracles and add proving sources

**Files:**

- Create: `fixtures/application_generation/flagship/command-write.sea`
- Create: `fixtures/application_generation/flagship/query-read.sea`
- Create: `fixtures/application_generation/compat/entity-no-body.sea`
- Create: `fixtures/application_generation/compat/keyword-collision.sea`
- Create: `domainforge-core/tests/application_compatibility_tests.rs`

**Interfaces:**

- Consumes: reference §§10–11 verbatim.
- Produces: exact D9 source-map inputs and committed legacy AST/formatter
  oracle hashes captured before grammar changes.

- [ ] **Step 1: Create the four exact fixtures**

Copy the two fenced sources from reference §11 byte for byte. Make
`entity-no-body.sea` a valid legacy file with namespace `compat.legacy`, one
bodyless entity, one resource whose unit is the identifier `operation`, and a
flow. Make `keyword-collision.sea` exercise each new token as every currently
legal identifier-like position; include `Resource "Record" operation` to
prove the resource lookahead remains contextual.

- [ ] **Step 2: Add the pre-change oracle test**

```rust
#[test]
fn legacy_entity_and_contextual_tokens_keep_ast_and_format() {
    for path in [
        "../fixtures/application_generation/compat/entity-no-body.sea",
        "../fixtures/application_generation/compat/keyword-collision.sea",
    ] {
        let source = std::fs::read_to_string(path).unwrap();
        let ast = domainforge_core::parser::parse(&source).unwrap();
        let schema: domainforge_core::parser::ast_schema::Ast = (&ast).into();
        let ast_json = serde_json::to_vec(&schema).unwrap();
        let formatted = domainforge_core::formatter::format(
            &source,
            domainforge_core::formatter::FormatConfig::default(),
        ).unwrap();
        assert_eq!(sha256(&ast_json), expected_ast_hash(path));
        assert_eq!(sha256(formatted.as_bytes()), expected_format_hash(path));
    }
}
```

Define local `sha256`, `expected_ast_hash`, and `expected_format_hash` helpers
with the four values printed by the current compiler. Do not regenerate these
values after grammar work.

- [ ] **Step 3: Run the oracle before changing grammar**

Run: `cargo test -p domainforge-core --features cli --test application_compatibility_tests legacy_entity_and_contextual_tokens_keep_ast_and_format -- --exact`

Expected: PASS. The flagship files are intentionally not parsed in this step.

- [ ] **Step 4: Commit the oracle packet**

```bash
git add fixtures/application_generation/flagship/command-write.sea fixtures/application_generation/flagship/query-read.sea fixtures/application_generation/compat/entity-no-body.sea fixtures/application_generation/compat/keyword-collision.sea domainforge-core/tests/application_compatibility_tests.rs
git commit -m "test(parser): capture application compatibility oracles"
```

---

### Task 2: Parse the accepted grammar into a partial internal AST

**Files:**

- Modify: `domainforge-core/grammar/sea.pest`
- Modify: `domainforge-core/src/parser/ast.rs`
- Modify: `domainforge-core/src/policy/expression.rs`
- Create: `domainforge-core/tests/application_parser_tests.rs`

**Interfaces:**

- Consumes: reference §2 grammar exactly.
- Produces: `EntityBody`, `FieldDecl`, `FieldTypeRef`, `FieldConstraintDecl`,
  `RecordDecl`, `EnumDecl`, `OperationDecl { clauses: Vec<OperationClause> }`,
  and `Expression::RoleReference { role: String }`.

- [ ] **Step 1: Add RED tests for both complete and partial syntax**

```rust
#[test]
fn parses_flagship_declarations_and_keeps_partial_operations() {
    let command = std::fs::read_to_string(
        "../fixtures/application_generation/flagship/command-write.sea",
    ).unwrap();
    let ast = parse(&command).expect("accepted command syntax must parse");
    assert!(ast.declarations.iter().any(|d| matches!(
        unwrap_export(&d.node), AstNode::Operation(OperationDecl { clauses, .. })
            if clauses.len() == 15
    )));

    let partial = parse("operation incomplete { direction inbound }").unwrap();
    assert!(matches!(unwrap_export(&partial.declarations[0].node),
        AstNode::Operation(OperationDecl { clauses, .. }) if clauses.len() == 1));
}
```

Add focused tests for entity defaults, all field types/constraints, alias
qualified references, `not_applicable`, all parsed-but-invalid v0.1 clause
values, and `role<Customer>`/`role<alias.Customer>`.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_parser_tests parses_flagship_declarations_and_keeps_partial_operations -- --exact`

Expected: FAIL because the new declaration grammar and AST variants do not
exist.

- [ ] **Step 3: Implement grammar and AST one-to-one**

Use the exact Pest block from reference §2. Keep `operation_clause*`; store
clauses in authored order. Represent every authored `symbol_ref` as its text
until resolution. Use `Expression` for defaults but accept only the grammar's
`literal` branch. Add `RoleReference` to display/normalization code paths as an
opaque leaf; application validation will restrict its placement.

- [ ] **Step 4: Run GREEN and the compatibility oracle**

Run: `cargo test -p domainforge-core --features cli --test application_parser_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli --test application_compatibility_tests legacy_entity_and_contextual_tokens_keep_ast_and_format -- --exact`

Expected: PASS with the pre-change hashes.

- [ ] **Step 5: Commit the parser packet**

```bash
git add domainforge-core/grammar/sea.pest domainforge-core/src/parser/ast.rs domainforge-core/src/policy/expression.rs domainforge-core/tests/application_parser_tests.rs
git commit -m "feat(parser): parse SEA application declarations"
```

---

### Task 3: Extend serialized AST v3 without changing legacy JSON

**Files:**

- Modify: `domainforge-core/src/parser/ast_schema.rs`
- Modify: `domainforge-core/src/parser/ast_convert.rs`
- Modify: `schemas/ast-v3.schema.json`
- Modify: `domainforge-core/tests/parser_ast_v3.rs`

**Interfaces:**

- Consumes: Task 2 internal AST.
- Produces: additive schema twins for every Task 2 type and
  `Expression::RoleReference { role: String }`; entity gains omitted
  `body: Option<EntityBody>`.

- [ ] **Step 1: Add the RED schema-shape test**

```rust
#[test]
fn application_nodes_are_additive_ast_v3_variants() {
    let source = std::fs::read_to_string(
        "../fixtures/application_generation/flagship/command-write.sea",
    ).unwrap();
    let internal = parse(&source).unwrap();
    let schema: ast_schema::Ast = (&internal).into();
    let value = serde_json::to_value(schema).unwrap();
    assert_eq!(value["declarations"][4]["node"]["type"], "Entity");
    assert!(value["declarations"][4]["node"]["body"].is_object());
    assert!(value["declarations"].as_array().unwrap().iter()
        .any(|d| d["node"]["type"] == "Operation"));
    assert!(value.get("metadata").is_some());
}
```

Also serialize `entity-no-body.sea` and assert the entity node has no `body`
key and equals the Task 1 oracle bytes.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test parser_ast_v3 application_nodes_are_additive_ast_v3_variants -- --exact`

Expected: FAIL because schema variants/conversions are absent.

- [ ] **Step 3: Add exact schema types and exhaustive conversions**

Mirror the internal partial AST. Preserve `#[serde(tag = "type")]` for
`ast_schema::AstNode`; do not apply the contract's adjacent tagging to AST v3.
Regenerate `schemas/ast-v3.schema.json` with the existing ignored schema test,
then inspect the diff for additive definitions only.

- [ ] **Step 4: Run GREEN and validate representative JSON**

Run: `cargo test -p domainforge-core --features cli,json-schema --test parser_ast_v3`

Expected: PASS; the legacy bodyless entity JSON is unchanged.

- [ ] **Step 5: Commit the AST schema packet**

```bash
git add domainforge-core/src/parser/ast_schema.rs domainforge-core/src/parser/ast_convert.rs schemas/ast-v3.schema.json domainforge-core/tests/parser_ast_v3.rs
git commit -m "feat(parser): extend AST v3 for application contracts"
```

---

### Task 4: Canonically print and round-trip new declarations

**Files:**

- Modify: `domainforge-core/src/parser/printer.rs`
- Modify: `domainforge-core/src/formatter/printer.rs`
- Modify: `domainforge-core/tests/printer_tests.rs`
- Modify: `domainforge-core/tests/round_trip_tests.rs`

**Interfaces:**

- Consumes: partial AST from Task 2.
- Produces: reference §10 formatting from both printers and parse-print-parse
  AST equality.

- [ ] **Step 1: Add RED formatting and round-trip assertions**

```rust
#[test]
fn flagship_format_is_canonical_and_idempotent() {
    let source = std::fs::read_to_string(
        "../fixtures/application_generation/flagship/command-write.sea",
    ).unwrap();
    let once = format(&source, FormatConfig::default()).unwrap();
    let twice = format(&once, FormatConfig::default()).unwrap();
    assert_eq!(once, twice);
    assert!(once.contains("    key order_id: uuid\n"));
    assert!(once.contains("    placed = \"placed\"\n"));
    assert_eq!(parse(&source).unwrap(), parse(&once).unwrap());
}
```

Add query fixture coverage, default-after-constraints ordering, failure/policy
binding sequence preservation, and source-order partial-operation printing.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test printer_tests flagship_format_is_canonical_and_idempotent -- --exact`

Expected: FAIL because printers lack new variants.

- [ ] **Step 3: Implement both printer branches**

Use four spaces, one field/clause/member per line, canonical clause order for a
valid operation, and authored order for a partial/invalid operation so APP001
evidence is not destroyed before validation. Emit enum commas exactly as §10.

- [ ] **Step 4: Run GREEN and legacy oracle**

Run: `cargo test -p domainforge-core --features cli --test printer_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli --test round_trip_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli --test application_compatibility_tests`

Expected: PASS with unchanged legacy hashes.

- [ ] **Step 5: Commit the printer packet**

```bash
git add domainforge-core/src/parser/printer.rs domainforge-core/src/formatter/printer.rs domainforge-core/tests/printer_tests.rs domainforge-core/tests/round_trip_tests.rs
git commit -m "feat(formatter): print application declarations canonically"
```

---

### Task 5: Build the single source-map module resolver

**Files:**

- Modify: `domainforge-core/src/module/resolver.rs`
- Modify: `domainforge-core/src/module/mod.rs`
- Modify: `domainforge-core/tests/module_resolution_tests.rs`
- Create: `domainforge-core/tests/application_contract_tests.rs`

**Interfaces:**

- Consumes: AST exports/imports and `SourceMap` fixed above.
- Produces: crate-private `ResolvedModuleSet`, `ResolvedModule`, `ImportEdge`,
  `AliasBinding`, `ResolvedSymbol`, `OriginRef`, and exact APP014 reasons.

- [ ] **Step 1: Add RED source-map closure tests**

```rust
#[test]
fn resolves_named_relative_import_into_one_symbol_table() {
    let sources = serde_json::json!({
        "flagship/command-write.sea": include_str!(
            "../../fixtures/application_generation/flagship/command-write.sea"),
        "flagship/query-read.sea": include_str!(
            "../../fixtures/application_generation/flagship/query-read.sea"),
    }).to_string();
    let set = resolve_for_test("flagship/query-read.sea", &sources).unwrap();
    assert_eq!(set.modules.iter().map(|m| m.logical_id.as_str()).collect::<Vec<_>>(),
        ["flagship/command-write.sea", "flagship/query-read.sea"]);
    assert!(set.symbols.contains_key("flagship.orders.entity.Order"));
    assert_eq!(set.import_graph.len(), 1);
}
```

Add one test each for relative root escape, duplicate normalized source key,
named visibility, wildcard alias qualification, unresolved alias/specifier,
cycle, symbol collision, and diamond-origin collapse. Assert the exact APP014
closed reason string.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests resolves_named_relative_import_into_one_symbol_table -- --exact`

Expected: FAIL because `ResolvedModuleSet` and source-map resolution are absent.

- [ ] **Step 3: Implement one resolver engine and compatibility adapter**

Replace `HashMap`/`HashSet` output ordering with deterministic `IndexMap` plus
explicit sorted vectors. Keep `ModuleResolver::validate_entry` and
`validate_dependencies` callable, but have them collect native sources and
delegate to the same closure engine. Do not canonicalize existing concept names
when building qualified legacy IDs.

- [ ] **Step 4: Run GREEN and current module tests**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests`

Expected: resolver-focused cases PASS; later contract tests may remain filtered
out by exact test names until their packets.

Run: `cargo test -p domainforge-core --features cli --test module_resolution_tests`

Expected: PASS through the compatibility adapter.

- [ ] **Step 5: Commit the resolver packet**

```bash
git add domainforge-core/src/module/resolver.rs domainforge-core/src/module/mod.rs domainforge-core/tests/module_resolution_tests.rs domainforge-core/tests/application_contract_tests.rs
git commit -m "feat(module): resolve one deterministic semantic closure"
```

---

### Task 6: Define strict public contract types and diagnostics

**Files:**

- Create: `domainforge-core/src/application/mod.rs`
- Create: `domainforge-core/src/application/contract.rs`
- Create: `domainforge-core/src/application/diagnostic.rs`
- Modify: `domainforge-core/src/lib.rs`
- Create: `domainforge-core/tests/application_public_types_tests.rs`

**Interfaces:**

- Consumes: exact type list in reference §8 and diagnostic context in §5.
- Produces: every public contract type, `ApplicationDiagnostic`,
  `ApplicationDiagnosticCode`, `ApplicationDiagnosticContext`, and stable JSON.

- [ ] **Step 1: Add RED public wire-shape tests**

```rust
#[test]
fn public_enums_use_adjacent_tags_and_reject_unknown_fields() {
    let ty = FieldType::Scalar { scalar: ScalarType::String };
    assert_eq!(serde_json::to_value(&ty).unwrap(), serde_json::json!({
        "kind": "scalar", "data": { "scalar": "string" }
    }));
    let bad = r#"{"kind":"scalar","data":{"scalar":"string","extra":1}}"#;
    assert!(serde_json::from_str::<FieldType>(bad).is_err());
}
```

Assert `ApplicationSymbolId` is a transparent string, absent options are
omitted, required booleans serialize explicitly, hashes include `sha256:`, and
an APP014 diagnostic contains `reason`, `expected`, `actual`, `remediation`,
logical ID, line, and column.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_public_types_tests`

Expected: FAIL because the application module is absent.

- [ ] **Step 3: Implement the exact public model**

Transcribe every contract/document/value type from reference §8, including all
field types and serde attributes. Add document constructors only after the
canonical packet; raw public fields must not permit construction to bypass
later validation APIs.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p domainforge-core --features cli --test application_public_types_tests`

Expected: PASS.

- [ ] **Step 5: Commit the public model packet**

```bash
git add domainforge-core/src/application/mod.rs domainforge-core/src/application/contract.rs domainforge-core/src/application/diagnostic.rs domainforge-core/src/lib.rs domainforge-core/tests/application_public_types_tests.rs
git commit -m "feat(application): define strict contract wire types"
```

---

### Task 7: Resolve types and construct normalized declarations

**Files:**

- Create: `domainforge-core/src/application/resolve.rs`
- Create: `domainforge-core/src/application/validate.rs`
- Modify: `domainforge-core/src/application/mod.rs`
- Modify: `domainforge-core/tests/application_contract_tests.rs`

**Interfaces:**

- Consumes: `ResolvedModuleSet` and public contract types.
- Produces: normalized enums, records, typed entities, operation references,
  `ApplicationSymbolId`, and APP002–APP006/APP013.

- [ ] **Step 1: Add RED construction tests**

```rust
#[test]
fn resolves_flagship_types_without_changing_concept_identity() {
    let doc = resolve_fixture_contract("flagship/command-write.sea").unwrap();
    assert_eq!(doc.contract.records[0].id.0,
        "flagship.orders.record.PlaceOrderInput");
    let order = doc.contract.entities.iter().find(|e| e.name == "Order").unwrap();
    assert_eq!(order.concept_id,
        ConceptId::from_concept("flagship.orders", "Order"));
    assert_eq!(order.key_field, "order_id");
}
```

Add exact diagnostic tests for unknown/wrong-kind references, quoted names that
cannot be referenced, duplicate fields, invalid/no/multiple keys, record
defaults, invalid list nesting, enum duplicate names/wires, and entity-as-DTO.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests resolves_flagship_types_without_changing_concept_identity -- --exact`

Expected: FAIL because contract construction is absent.

- [ ] **Step 3: Implement declaration construction**

Resolve every reference through `ResolvedModuleSet::symbols`; never scan AST by
name. Normalize constraints to the accepted closed enum, preserve field/member
sequence, and return all deterministic diagnostics sorted by logical ID, line,
column, then APP code.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests resolves_flagship_types_without_changing_concept_identity -- --exact`

Expected: PASS.

- [ ] **Step 5: Commit the type-resolution packet**

```bash
git add domainforge-core/src/application/resolve.rs domainforge-core/src/application/validate.rs domainforge-core/src/application/mod.rs domainforge-core/tests/application_contract_tests.rs
git commit -m "feat(application): resolve typed contract declarations"
```

---

### Task 8: Validate complete operation semantics and lowering

**Files:**

- Modify: `domainforge-core/src/application/validate.rs`
- Modify: `domainforge-core/src/application/resolve.rs`
- Modify: `domainforge-core/tests/application_contract_tests.rs`
- Create: `fixtures/application_generation/invalid/app001-missing-application-semantics.sea`
- Create: `fixtures/application_generation/invalid/app011-effect-state-mismatch.sea`

**Interfaces:**

- Consumes: resolved declaration candidates.
- Produces: valid `OperationContract` or APP001/APP008–APP012; exact state,
  output, failure, idempotency, concurrency, and bound rules from reference §4.

- [ ] **Step 1: Add RED operation matrix tests**

```rust
#[test]
fn command_and_query_lower_to_the_closed_v01_semantics() {
    let command = resolve_fixture_contract("flagship/command-write.sea").unwrap();
    let place = operation(&command, "place_order");
    assert!(matches!(place.effect, EffectKind::Creates));
    assert!(matches!(place.transaction, TransactionBoundary::SingleAggregate));
    assert!(matches!(place.idempotency,
        IdempotencyStrategy::KeyedBy { ref field } if field == "client_order_id"));
    assert!(matches!(place.concurrency,
        ConcurrencyStrategy::UniqueKey { ref field } if field == "client_order_id"));

    let query = resolve_fixture_contract("flagship/query-read.sea").unwrap();
    let read = operation(&query, "get_order_status");
    assert!(matches!(read.access, AccessMode::Public));
    assert!(matches!(read.effect, EffectKind::Reads));
}
```

Add table-driven inline sources for every APP001 reason, failure-code/kind
mapping, strategy/effect pairing, not-applicable proof, inclusive/exclusive
interval, decimal precision/scale, create source/default precedence, mutate
field mapping, read key lookup, and output projection compatibility.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests command_and_query_lower_to_the_closed_v01_semantics -- --exact`

Expected: FAIL because operation validation is incomplete.

- [ ] **Step 3: Implement validation as named pure phases**

Implement `validate_clause_shape`, `validate_fields`, `validate_failures`,
`validate_state_lowering`, and `validate_strategies`; merge diagnostics before
constructing an operation. APP001 must report missing, duplicate, out-of-order,
unsupported, and transaction defects together for one operation.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests command_and_query_lower_to_the_closed_v01_semantics -- --exact`

Expected: PASS.

- [ ] **Step 5: Commit the operation packet**

```bash
git add domainforge-core/src/application/validate.rs domainforge-core/src/application/resolve.rs domainforge-core/tests/application_contract_tests.rs fixtures/application_generation/invalid/app001-missing-application-semantics.sea fixtures/application_generation/invalid/app011-effect-state-mismatch.sea
git commit -m "feat(application): validate operation semantics"
```

---

### Task 9: Validate and evaluate the closed policy subset

**Files:**

- Create: `domainforge-core/src/application/policy_context.rs`
- Modify: `domainforge-core/src/application/validate.rs`
- Modify: `domainforge-core/src/application/mod.rs`
- Modify: `domainforge-core/src/policy/normalize.rs`
- Create: `domainforge-core/tests/application_policy_tests.rs`

**Interfaces:**

- Consumes: existing `policy::Expression`, resolved role IDs, input/state field
  types, `ApplicationPolicyContext`.
- Produces: subset validation plus
  `evaluate_precondition(expression, context) -> EvaluationResult`, with false,
  unknown, and error mapped by the caller to the binding failure code.

- [ ] **Step 1: Add RED policy tests**

```rust
#[test]
fn flagship_limit_uses_quantity_unit_and_fails_closed() {
    let fixture = resolved_place_order();
    assert_eq!(evaluate_total(&fixture, "10000"), EvaluationResult::True);
    assert_eq!(evaluate_total(&fixture, "10000.01"), EvaluationResult::False);
    assert_eq!(evaluate_missing_total(&fixture), EvaluationResult::Unknown);
}
```

Add actor-role equality tests using `role<Customer>`, anonymous actor unknown,
input/state member access, division by zero, wrong units, forbidden expression
nodes/operators, wrong policy kind, dangling binding, and reserved
invariant/postcondition. Assert APP007 and the declared failure code.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_policy_tests`

Expected: FAIL because the application evaluator is absent.

- [ ] **Step 3: Implement the application-owned evaluator**

Do not extend Graph evaluation context or synthesize instances. Convert a bare
numeric operand to a quantity only when paired with a declared quantity field.
Keep existing three-valued truth tables. Update every exhaustive match affected
by `Expression::RoleReference`; outside application validation it remains an
unevaluated leaf or returns the existing invalid-expression path.

- [ ] **Step 4: Run GREEN and policy regressions**

Run: `cargo test -p domainforge-core --features cli --test application_policy_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli --test three_valued_quantifiers_tests`

Expected: PASS with unchanged existing policy behavior.

- [ ] **Step 5: Commit the policy packet**

```bash
git add domainforge-core/src/application/policy_context.rs domainforge-core/src/application/validate.rs domainforge-core/src/application/mod.rs domainforge-core/src/policy/normalize.rs domainforge-core/tests/application_policy_tests.rs
git commit -m "feat(application): enforce typed operation policies"
```

---

### Task 10: Register APP diagnostics and complete negative fixtures

**Files:**

- Modify: `domainforge-core/src/validation_error.rs`
- Modify: `docs/reference/error-codes.md`
- Modify: `docs/diagnostics.md`
- Modify: `domainforge-core/tests/application_parser_tests.rs`
- Create: `domainforge-core/tests/application_diagnostic_tests.rs`

**Interfaces:**

- Consumes: `ApplicationDiagnostic` from Task 6 and validators from Tasks 7–9.
- Produces: stable APP001–APP015 documentation, sorting, severity, source/artifact
  evidence, and remediation assertions.

- [ ] **Step 1: Add RED diagnostic contract tests**

```rust
#[test]
fn every_app_code_has_stable_slug_and_error_severity() {
    let expected = [
        ("APP001", "missing_application_semantics"),
        ("APP002", "unknown_contract_reference"),
        ("APP003", "invalid_record_shape"),
        ("APP004", "duplicate_field"),
        ("APP005", "invalid_aggregate_key"),
        ("APP006", "operation_type_mismatch"),
        ("APP007", "policy_scope_error"),
        ("APP008", "invalid_failure_declaration"),
        ("APP009", "invalid_strategy_reference"),
        ("APP010", "invalid_not_applicable"),
        ("APP011", "effect_state_mismatch"),
        ("APP012", "constraint_error"),
        ("APP013", "enum_error"),
        ("APP014", "closure_resolution_error"),
        ("APP015", "artifact_schema_violation"),
    ];
    assert_eq!(ApplicationDiagnosticCode::all_code_slugs(), expected);
    assert!(ApplicationDiagnosticCode::all().iter().all(|c| c.severity() == "error"));
}
```

For APP001–APP014, add focused inline source cases covering every invariant in
reference §5. For APP014 assert all five closed reason values. APP015 artifact
cases land with Task 12.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_diagnostic_tests every_app_code_has_stable_slug_and_error_severity -- --exact`

Expected: FAIL until the registry and documentation are complete.

- [ ] **Step 3: Register codes without forcing them into legacy errors**

Add a dedicated `ValidationError::Application(ApplicationDiagnostic)` bridge
only where existing CLI/error formatting requires it. Preserve current E-code
values and JSON. Document exact context fields and smallest remediation text.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p domainforge-core --features cli --test application_diagnostic_tests`

Expected: PASS for APP001–APP014 and registry shape.

- [ ] **Step 5: Commit the diagnostics packet**

```bash
git add domainforge-core/src/validation_error.rs docs/reference/error-codes.md docs/diagnostics.md domainforge-core/tests/application_parser_tests.rs domainforge-core/tests/application_diagnostic_tests.rs
git commit -m "feat(diagnostics): register application contract errors"
```

---

### Task 11: Canonicalize source sets and Application Contract documents

**Files:**

- Create: `domainforge-core/src/application/canonical.rs`
- Modify: `domainforge-core/src/application/resolve.rs`
- Modify: `domainforge-core/src/application/mod.rs`
- Modify: `domainforge-core/tests/application_contract_tests.rs`
- Create: `domainforge-core/tests/semantic_pack_compat_tests.rs`

**Interfaces:**

- Consumes: validated contract and resolved source set.
- Produces: canonical decimals/strings/quantities, canonical input fingerprint,
  `source_set_hash`, `semantic_pack_set_hash`, contract `self_hash`, and compact
  canonical JSON.

- [ ] **Step 1: Add RED golden-vector tests**

```rust
#[test]
fn source_set_hash_is_framed_sorted_and_path_independent() {
    let a = source_set_hash(&[("b.sea", "Entity \"B\""), ("a.sea", "Entity \"A\"")]).unwrap();
    let b = source_set_hash(&[("a.sea", "Entity \"A\""), ("b.sea", "Entity \"B\"")]).unwrap();
    assert_eq!(a, b);
    assert!(a.starts_with("sha256:"));
    assert_eq!(a.len(), 71);
}
```

Add fixed expected hashes for the source-set framing, empty semantic-pack set,
`125.00` → `125`, NFC-equivalent application strings, quantity base-unit
normalization, contract self-hash omission rule, and canonical input
fingerprint. Capture current semantic-pack fixture hash/signature values before
changing any shared helper.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests source_set_hash_is_framed_sorted_and_path_independent -- --exact`

Expected: FAIL because application canonicalization is absent.

- [ ] **Step 3: Implement typed normalization**

Build typed canonical values first, then pass `serde_json::Value` to the
unchanged `semantic_pack::canonical_json` byte serializer. Compute contract
`semantic_closure_hash` only in Task 12 because it depends on the full envelope.

- [ ] **Step 4: Run GREEN and pack compatibility**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests source_set_hash_is_framed_sorted_and_path_independent -- --exact`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli --test semantic_pack_compat_tests`

Expected: PASS with byte-identical pre-packet pack hashes/signatures.

- [ ] **Step 5: Commit the canonical contract packet**

```bash
git add domainforge-core/src/application/canonical.rs domainforge-core/src/application/resolve.rs domainforge-core/src/application/mod.rs domainforge-core/tests/application_contract_tests.rs domainforge-core/tests/semantic_pack_compat_tests.rs
git commit -m "feat(application): canonicalize contract documents"
```

---

### Task 12: Build and validate the canonical semantic envelope

**Files:**

- Create: `domainforge-core/src/application/envelope.rs`
- Modify: `domainforge-core/src/application/mod.rs`
- Create: `domainforge-core/tests/application_schema_tests.rs`
- Create: `schemas/application-contract-v1.schema.json`
- Create: `schemas/canonical-semantic-envelope-v1.schema.json`

**Interfaces:**

- Consumes: full resolved closure, normalized contract, semantic-pack refs.
- Produces: exact reference §8 envelope types, exhaustive canonical payload
  conversion, `semantic_closure_hash`, document self-hashes, strict schemas,
  and APP015.

- [ ] **Step 1: Add RED envelope and schema tests**

```rust
#[test]
fn envelope_hash_ignores_formatting_but_covers_import_meaning() {
    let a = envelope_for_sources(flagship_sources()).unwrap();
    let b = envelope_for_sources(reformatted_flagship_sources()).unwrap();
    assert_eq!(a.semantic_closure_hash, b.semantic_closure_hash);
    let changed = envelope_for_sources(policy_limit_changed_sources()).unwrap();
    assert_ne!(a.semantic_closure_hash, changed.semantic_closure_hash);
}
```

Add occurrence-ID duplicate indexing, deterministic instance-ID parity with
Graph, exhaustive semantic declaration classification, source-position/export
wrapper rejection, unknown JSON fields, wrong schema versions, missing inputs,
bad self-hash, bad semantic hash, invalid hash prefix/hex, and JSON Pointer
context tests.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli,json-schema --test application_schema_tests envelope_hash_ignores_formatting_but_covers_import_meaning -- --exact`

Expected: FAIL because the envelope and schemas are absent.

- [ ] **Step 3: Implement exhaustive envelope conversion and schemas**

Use one `match` without `_` over semantic AST variants. Use reference §6 kind
rank, sequence rules, defaults, module content hash, resolved-reference field
paths, occurrence IDs, and document hash omission rules. Validate schema,
inputs, and self-hash before returning a deserialized persisted artifact.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p domainforge-core --features cli,json-schema --test application_schema_tests`

Expected: PASS, including APP015 cases for both documents.

- [ ] **Step 5: Commit the envelope packet**

```bash
git add domainforge-core/src/application/envelope.rs domainforge-core/src/application/mod.rs domainforge-core/tests/application_schema_tests.rs schemas/application-contract-v1.schema.json schemas/canonical-semantic-envelope-v1.schema.json
git commit -m "feat(application): emit canonical semantic envelopes"
```

---

### Task 13: Construct Graph and contract from the same resolved set

**Files:**

- Modify: `domainforge-core/src/parser/ast.rs`
- Modify: `domainforge-core/src/parser/mod.rs`
- Modify: `domainforge-core/src/graph/mod.rs`
- Modify: `domainforge-core/tests/graph_integration_tests.rs`
- Modify: `domainforge-core/tests/application_contract_tests.rs`

**Interfaces:**

- Consumes: `ResolvedModuleSet`.
- Produces: one internal `build_graph(&ResolvedModuleSet, &ParseOptions)` path
  and proof that Graph existing-concept IDs equal contract references.

- [ ] **Step 1: Add RED shared-identity test**

```rust
#[test]
fn graph_and_contract_share_order_and_role_concept_ids() {
    let resolved = resolved_flagship_set();
    let graph = build_graph_for_test(&resolved).unwrap();
    let contract = build_contract_for_test(&resolved).unwrap();
    let order = contract.entities.iter().find(|e| e.name == "Order").unwrap();
    assert!(graph.has_entity(&order.concept_id));
    let role = match &operation(&contract, "place_order").actor {
        ActorRef::Role { role } => role,
        ActorRef::Anonymous => panic!("place_order must have Customer actor"),
    };
    assert!(graph.get_role(role).is_some());
}
```

Add diamond-import no-duplicate Graph construction and deterministic instance
ID parity. Records, enums, and operations must not appear as Graph primitives.

- [ ] **Step 2: Run RED**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests graph_and_contract_share_order_and_role_concept_ids -- --exact`

Expected: FAIL while Graph still converts only the entry AST.

- [ ] **Step 3: Route module-aware graph construction through the set**

Keep single-source `parse_to_graph` behavior unchanged by resolving a one-entry
source map internally. Delete independent module traversal from Graph paths;
retain only the native adapter in the module resolver.

- [ ] **Step 4: Run GREEN and Graph regressions**

Run: `cargo test -p domainforge-core --features cli --test application_contract_tests graph_and_contract_share_order_and_role_concept_ids -- --exact`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli --test graph_integration_tests`

Expected: PASS.

- [ ] **Step 5: Commit the shared ownership packet**

```bash
git add domainforge-core/src/parser/ast.rs domainforge-core/src/parser/mod.rs domainforge-core/src/graph/mod.rs domainforge-core/tests/graph_integration_tests.rs domainforge-core/tests/application_contract_tests.rs
git commit -m "refactor(module): share resolution across graph and contract"
```

---

### Task 14: Expose thin Python, TypeScript, and WASM JSON bindings

**Files:**

- Modify: `domainforge-core/src/python/graph.rs`
- Modify: `domainforge-core/src/typescript/graph.rs`
- Modify: `domainforge-core/src/wasm/graph.rs`
- Modify: `tests/test_parser.py`
- Modify: `typescript-tests/native-binding.test.ts`
- Modify: `domainforge-core/tests/wasm_tests.rs`

**Interfaces:**

- Consumes: `application::resolve_application_contract_json` only.
- Produces: the exact public methods in “Fixed interfaces and ownership” and
  byte-identical compact canonical JSON.

- [ ] **Step 1: Add RED parity tests in each target**

Python:

```python
def test_resolve_application_contract_json_is_canonical() -> None:
    sources = json.dumps(flagship_sources(), separators=(",", ":"))
    raw = domainforge.Graph.resolve_application_contract_json(
        "flagship/query-read.sea", sources
    )
    assert raw == canonical_contract_golden()
```

TypeScript:

```typescript
it('exposes AST JSON and canonical application contract JSON', () => {
  expect(JSON.parse(Graph.parseToAstJson('Entity "A"')).declarations).toHaveLength(1);
  expect(Graph.resolveApplicationContractJson(entry, sourcesJson))
    .toBe(canonicalContractGolden);
});
```

WASM uses `wasm_bindgen_test` with the same entry/source-map strings and golden
JSON. Each suite also asserts malformed source-map diagnostics.

- [ ] **Step 2: Run RED per target**

Run: `just python-test`

Expected: FAIL only for the missing Python contract method.

Run: `just ts-test`

Expected: FAIL only for missing TypeScript AST/contract methods.

Run: `cargo test -p domainforge-core --features wasm --test wasm_tests`

Expected: FAIL only for the missing WASM contract method.

- [ ] **Step 3: Add delegation-only methods**

Each wrapper passes both strings unchanged into Rust. Python maps the diagnostic
array to `PyValueError`; napi maps it to `Error::from_reason`; WASM maps it to a
`JsValue` string. Do not parse, normalize, resolve, or pretty-print JSON in a
binding.

- [ ] **Step 4: Run GREEN per target**

Run: `just python-test`

Expected: PASS.

Run: `just ts-test`

Expected: PASS.

Run: `cargo test -p domainforge-core --features wasm --test wasm_tests`

Expected: PASS.

- [ ] **Step 5: Commit the binding packet**

```bash
git add domainforge-core/src/python/graph.rs domainforge-core/src/typescript/graph.rs domainforge-core/src/wasm/graph.rs tests/test_parser.py typescript-tests/native-binding.test.ts domainforge-core/tests/wasm_tests.rs
git commit -m "feat(bindings): expose application contract JSON"
```

---

### Task 15: Prove the Milestone 0 stop gate

**Files:**

- Modify: `domainforge-core/tests/application_contract_tests.rs`
- Modify: `domainforge-core/tests/application_schema_tests.rs`
- Modify: `domainforge-core/tests/application_compatibility_tests.rs`
- Modify: `domainforge-core/tests/semantic_pack_compat_tests.rs`
- Modify: `docs/reference/sea-application-contract.md`

**Interfaces:**

- Consumes: all prior packets.
- Produces: one explicit coverage matrix and final evidence that the accepted
  contract is implemented without generator work.

- [ ] **Step 1: Add final cross-source determinism test**

```rust
#[test]
fn fixed_logical_source_set_is_byte_deterministic() {
    let sources = flagship_sources_json();
    let first = resolve_application_contract_json(
        "flagship/query-read.sea", &sources).unwrap();
    let second = resolve_application_contract_json(
        "flagship/query-read.sea", &sources).unwrap();
    assert_eq!(first.as_bytes(), second.as_bytes());
    assert_eq!(document(&first).semantic_closure_hash,
        document(&second).semantic_closure_hash);
}
```

Add a table in the test module mapping reference §§2, 4–10, and 13 to exact
test names. Add the final APP015 persisted-artifact vectors and D9 contract
assertions: defaulted status, failure mapping, policy binding, state/effect,
strategy order metadata, restart-stable IDs, and public query output fields.

- [ ] **Step 2: Run focused Milestone 0 suites**

Run: `cargo test -p domainforge-core --features cli,json-schema --test application_parser_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli,json-schema --test application_contract_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli,json-schema --test application_schema_tests`

Expected: PASS.

Run: `cargo test -p domainforge-core --features cli,json-schema --test application_compatibility_tests`

Expected: PASS with pre-change oracle hashes.

Run: `cargo test -p domainforge-core --features cli,json-schema --test semantic_pack_compat_tests`

Expected: PASS with pre-change pack hashes/signatures.

- [ ] **Step 3: Run repository parity and quality gates**

Run: `just all-tests`

Expected: PASS for Rust, Python, and TypeScript.

Run: `cargo clippy -p domainforge-core --all-targets --features cli,python,typescript,wasm,json-schema -- -D warnings`

Expected: PASS with no warnings.

Run: `cargo fmt --all -- --check`

Expected: PASS.

- [ ] **Step 4: Reconcile the normative reference with implemented facts**

Change only statements that identify future Milestone 0 work as unimplemented;
do not alter accepted semantics. Add exact implementation/test paths beside the
existing normative sections. Run `git diff --check` and inspect the full diff.

- [ ] **Step 5: Commit final gate evidence**

```bash
git add domainforge-core/tests/application_contract_tests.rs domainforge-core/tests/application_schema_tests.rs domainforge-core/tests/application_compatibility_tests.rs domainforge-core/tests/semantic_pack_compat_tests.rs docs/reference/sea-application-contract.md
git commit -m "test(application): prove Milestone 0 contract parity"
```

## Milestone 0 human review gate

Stop after Task 15. Present:

- all 15 commit hashes;
- the focused and repository-wide command results;
- AST/formatter compatibility oracle values;
- semantic-pack hash/signature oracle values;
- the canonical D9 `ApplicationContractDocument` and
  `CanonicalSemanticEnvelopeDocument` hashes;
- any accepted-document wording updated to match implementation.

The reviewer must confirm:

- [ ] both flagship sources parse, resolve, validate, and emit strict documents;
- [ ] APP001–APP015 and every closed reason/invariant have focused proof;
- [ ] Graph and contract share resolved existing-concept IDs;
- [ ] all three bindings return byte-identical canonical contract JSON;
- [ ] existing AST, formatter, Graph, policy, and semantic-pack behavior remains
      compatible;
- [ ] no Adapter IR, provider, generator, proof runner, Axum, SQLite, approval,
      or skill implementation entered the diff.

Only explicit human acceptance of this gate authorizes the Milestone 1 plan.

## Requirement-to-task traceability

| Accepted requirement | Task(s) |
|---|---|
| Exact contextual grammar and partial operation AST | 1–2 |
| Additive AST v3 and schema conversion | 3 |
| Canonical formatter/printer and round-trip | 4 |
| D3 shared import/identity/visibility closure | 5, 13 |
| Exact public Rust contract types | 6 |
| D5 types, fields, defaults, enums, keys, constraints | 7–8 |
| D6 operation, effect, transaction, failure, strategy semantics | 8 |
| Closed policy context and fail-closed enforcement | 9 |
| APP001–APP015 diagnostics | 5, 7–10, 12 |
| D10 source/pack/contract hashes and typed normalization | 11 |
| Canonical semantic envelope and closure hash | 12 |
| Python, TypeScript, and WASM JSON parity | 14 |
| D9 fixtures and deterministic contract proof | 1, 8, 15 |
| Legacy AST/formatter and semantic-pack compatibility | 1–4, 11, 15 |

## Known implementation risks and stop rules

| Risk | Stop rule and response |
|---|---|
| A contextual token changes an existing AST or formatted file. | Stop at Task 2 or 4; classify the token as breaking and invoke ADR-013's migration contingency. |
| Native registry and source-map resolution produce different symbols. | Stop at Task 5; keep one resolver engine and repair the native adapter. |
| Adding `RoleReference` changes existing policy normalization/evaluation. | Stop at Task 9; preserve old expression results and keep role resolution application-owned. |
| A public type needs a non-additive AST v3 shape. | Stop at Task 3; seek an ast-v4 decision. |
| Envelope conversion cannot classify a semantic AST variant. | Fail compilation through exhaustive matching; add a normative classification before proceeding. |
| An application canonicalization change alters a semantic-pack hash/signature. | Stop at Task 11; remove the coupling instead of updating pack goldens. |
| Cross-binding JSON differs. | Stop at Task 14; remove binding-side processing and compare raw Rust-produced bytes. |
| D9 runtime behavior seems necessary to prove language semantics. | Keep only contract-level assertions; defer execution, persistence, and restart proof to the provider/generator milestones. |
