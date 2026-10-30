# wasm — wasm-bindgen Bindings

## Overview
wasm-bindgen wrappers over the canonical Rust core for browser/edge use.
`mod.rs` re-exports the whole surface (`authority`, `formatter`, `graph`,
`policy`, `primitives`, `semantic_pack`, `units`); `lib.rs` installs the
`lol_alloc` global allocator only under
`all(feature = "lol_alloc", target_arch = "wasm32")`. Unlike `python`/`typescript`,
there is **no** `registry` module here.

## Interface
- `[mod.rs]` — module list: `authority`, `error`, `formatter`, `graph`, `policy`, `primitives`, `semantic_pack`, `units`; re-exports all except `error`.
- `[graph.rs:Graph]` — `#[wasm_bindgen]` wrapper with `#[wasm_bindgen(constructor)] new()`; JS-renamed methods `parse`, `parseToAstJson`, `resolveApplicationContractJson`, `domainModelIdentityJson`, `worldRefFromIdentityJson`, `verifyWorldRef`, `parseWorldRef`, `isEmpty`, `addEntity`, `hasEntity`, `getEntity`, `removeEntity`, `findEntityByName`, `entityContractJson`, `entityCount`, `allEntities`, and the analogous `Resource`/`Flow`/`Instance`/`Policy`/`Metric`/`Mapping` families.
- `[authority.rs:FinalDecision]`, `[authority.rs:PolicyModality]`, `[authority.rs:SourceClass]`, `[authority.rs:ClaimLevel]` — `#[wasm_bindgen]` enum mirrors.
- `[authority.rs:evaluate_authority]` — authority entry point.
- `[error.rs:to_js_error]` — converts a `ValidationError` into a `JsValue` error object.
- `[formatter.rs:format_source]` (js_name `formatSource`), `[formatter.rs:check_format]` (js_name `checkFormat`) — formatter entry points.
- `[policy.rs:AggregateFunction]`, `BinaryOp`, `UnaryOp`, `Quantifier`, `Severity`, `WindowSpec`, `Expression`, `NormalizedExpression`, `Violation`, `EvaluationResult` — policy wrappers; `Expression` exposes `literal`/`literal_bool`/`literal_number`/`literal_string`/`variable`/`quantity`/`time`/`interval`/`binary`/`unary`/`cast`/`quantifier_expr`/`member_access`/`aggregation`/`aggregation_comprehension`/`group_by`/`normalize`/`is_equivalent`/`to_string_repr`/`equals`/`into_inner`.
- `[primitives.rs:Entity]`, `[primitives.rs:Resource]`, `[primitives.rs:Flow]`, `[primitives.rs:Instance]` — `#[wasm_bindgen]` wrappers deriving `Serialize`.
- `[units.rs:Dimension]` (`+new/to_string`), `[units.rs:Unit]` (`+new` and getters `symbol`/`name`/`base_factor`/`base_unit`) — unit wrappers.
- `[semantic_pack.rs]` — the enum mirrors (`SemanticTruth`, `DiagnosticSeverity`, `ValidationMode`, `ApprovalState`, `SignatureState`, `ConceptStatus`, `ConceptKind`, `AliasStatus`, `SemanticValidationStatus`) plus `semanticPackBuild`, `semanticPackValidate`, `semanticPackValidateGraph`, `semanticPackSign`, `semanticPackVerify`, `semanticPackDiff`, `semanticPackHash`, `semanticNormalizeKey`, `semanticResolveConcept`.

## Invariants
- `[mod.rs]` — `policy` is annotated `#[allow(clippy::clone_on_copy)]` because `wasm_bindgen`'s `getter_with_clone` emits `.clone()` on `Copy` fields; the allow is scoped to that module only.
- `[graph.rs]` — the module carries `#![allow(clippy::new_without_default)]` because wasm-bindgen requires an explicit `new()` constructor for JS.
- `[error.rs:to_js_error]` — core validation failures surface as a serialized JS error object via `serde_wasm_bindgen`, never a Rust panic.
- `[semantic_pack.rs:semantic_pack_sign]`/`semantic_pack_verify` — the `#[cfg(not(feature = "signing"))]` variants return `Err(JsError::new("signing requires the 'signing' feature"))` instead of succeeding.
- `[mod.rs]` — this module holds no duplicated business logic; all behavior delegates to the Rust core.

## Side Effects
- None observed in source. There is no `registry` module here, so the WASM surface performs no filesystem access at all (unlike `python`/`typescript`).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
