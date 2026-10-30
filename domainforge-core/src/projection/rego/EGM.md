# Rego / OPA Policy Projection

## Overview
Projects a SEA model's `Policy` declarations into OPA/Rego v1 modules that reproduce the DomainForge evaluator's three-valued result per policy and aggregate those results into an allow / deny / escalate decision, shipped as an OPA bundle with fidelity and provenance reports.

## Interface
- `[mod.rs:PROJECTION_VERSION]` — the projection identity tag, `domainforge-rego/v1`.
- `[mod.rs:FIDELITY_SCHEMA]` — the fidelity report schema, `domainforge.rego.fidelity/v1`.
- `[mod.rs:PROVENANCE_SCHEMA]` — the provenance document schema, `domainforge.rego.provenance/v1`.
- `[mod.rs:MANIFEST_FILE]` — `bundle/.manifest` (OPA bundle manifest).
- `[mod.rs:DATA_FILE]` — `bundle/data.json` (policy metadata + model identity).
- `[mod.rs:LIB_FILE]` — `bundle/domainforge/sea/lib.rego` (three-valued helpers).
- `[mod.rs:DECISION_FILE]` — `bundle/domainforge/policy/decision.rego`.
- `[mod.rs:POLICIES_FILE]` — `bundle/domainforge/policy/policies.rego`.
- `[mod.rs:INPUT_SCHEMA_FILE]` / `[mod.rs:DECISION_SCHEMA_FILE]` — `contracts/input.schema.json` and `contracts/decision.schema.json`.
- `[mod.rs:TEST_FILE]` — `tests/decision_test.rego`.
- `[mod.rs:FIDELITY_FILE]` / `[mod.rs:PROVENANCE_FILE]` — `fidelity-report.json` and `provenance.json`.
- `[mod.rs:RegoProjectionError]` — typed errors: `Unrepresentable` (REGO001), `Model` (REGO002), `Write` (REGO003).
- `[mod.rs:model_hash]` — sha256 of the canonically formatted `.sea` source, shared with `cell.lock` identity.
- `[mod.rs:emit]` — writes the full bundle tree into an `ArtifactSink`.
- `[mod.rs:project_rego_in_memory]` — binding surface returning `BTreeMap<path, content>`.
- `[ir.rs:RegoPolicyIr]` / `[ir.rs:RegoPolicyIr::build]` — the narrow policy IR and its graph lowering.
- `[ir.rs:PolicyIr]` / `[ir.rs:CollectionIr]` / `[ir.rs:CmpOp]` / `[ir.rs:ValueIr]` / `[ir.rs:BoolIr]` / `[ir.rs:COLLECTIONS]` — the policy expression IR.
- `[render.rs:policies_module]` — renders the policy IR as Rego v1 terms.
- `[reference.rs:DECISION_CONTRACT]` — the aggregate decision contract, `domainforge.rego.decision/v1`.
- `[reference.rs:facts_from_graph]` / `[reference.rs:input_errors]` / `[reference.rs:evaluate_policy]` / `[reference.rs:decide]` — the Rust reference implementation the Rego mirrors.
- Static Rego assets: `rego/lib.rego` and `rego/decision.rego`, embedded at compile time via `include_str!` in `mod.rs`.

## Invariants
- `[ir.rs:RegoPolicyIr::build]` — a policy using any construct outside the IR fails the whole projection with `Unrepresentable`; nothing is approximated or dropped.
- `[ir.rs:RegoPolicyIr::build]` — a policy bound to an operation's `access policy_governed by` clause is rejected as unrepresentable.
- `[mod.rs]` (module doc) — `True` / `False` / `Unknown` and evaluation errors stay distinct end to end; Rego `undefined` never stands in for `Unknown`.
- `[reference.rs:DECISION_CONTRACT]` — the aggregate allow/deny/escalate decision is this projection's own contract, not an evaluator outcome; the fidelity report records it as `adapter`, not `isomorphic`.
- `[mod.rs:model_hash]` — canonical-source hashing means whitespace-only edits do not change model identity.
- Only `provenance.json` carries `created_at`, so the rest of the bundle is byte-identical for a fixed model.

## Side Effects
- `[mod.rs:emit]` — writes the whole artifact tree through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
