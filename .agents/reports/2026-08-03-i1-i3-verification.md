# SEA Interaction Interventions I1-I3 Verification

Date: 2026-08-03

## Scope and result

I1-I3 are implemented without adding an import grammar, journey-specific or
ordered-transition grammar, semantic-pack declaration loading, CMMN behavior,
or I4-I6. Existing application-contract types, diagnostics, module resolution,
graph abstractions, and policy evaluator paths are reused.

## Acceptance evidence

1. Typed graph contracts and concrete instance validation:
   `entity_instance_validation_tests.rs` (11 tests) covers preservation,
   required/optional and unknown fields, scalar and supported constraints,
   enum membership, list constraints, forward and dangling typed references,
   and key uniqueness.
2. Deterministic transitive filesystem closure:
   `cli_module_closure_tests.rs` (3 tests) proves same-namespace imported
   declarations resolve for validate and parse, and invalid imported enum data
   fails.
3. Policy exposure:
   `policy_entity_instance_tests.rs` (3 tests) proves parsed plural
   quantification, aggregate singular binding, and reserved-binding safety.
4. Backward compatibility:
   the bodyless compatibility test, application compatibility suite, all
   TypeScript/Python/WASM suites, and `just prove` pass. Untyped/bodyless graph
   serialization stays unchanged because empty contract indexes are omitted.
5. SEA Forge exercise:
   the original SEA Forge interaction model validates with zero violations.
   The faithful modular fixture tests (2 tests) prove the previously failing
   imported-instance topology succeeds and intentionally invalid enum and
   dangling-reference data fails.

## Commands and results

- `just ai-validate`: passed.
- `cargo test -p domainforge-core --features cli --test application_contract_tests --test application_compatibility_tests --test entity_instance_validation_tests --test cli_module_closure_tests --test policy_entity_instance_tests --test sea_forge_interaction_fixture_tests`: passed, 54 tests.
- `cargo check -p domainforge-core --all-features`: passed.
- `cargo clippy -p domainforge-core --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `just python-test`: passed (5 environment/feature skips, no failures).
- `just ts-test`: passed, 197 tests in 25 files.
- `just wasm-test`: passed, 21 tests.
- `just prove`: passed; evidence is in `evidence/latest/proof.{json,md}`.
- `target/debug/domainforge validate /home/sprime01/projects/sea-rs/.sea/interaction/interaction-model.sea`: passed with 0 violations.
- `target/debug/domainforge validate fixtures/sea_forge_interaction_i1_i3/valid-instances.sea`: passed with 0 violations.
- `target/debug/domainforge validate fixtures/sea_forge_interaction_i1_i3/invalid-instances.sea`: exited 1 as intended with invalid `Maturity` enum member `unknown` and dangling `CanonicalJourney` reference `MISSING`.

The proof gate reports a structural-only TLA result because Java and
`tla2tools.jar` are unavailable in this environment; all executable validation
gates pass and CI can supply the model-checker toolchain.

## Compatibility and deviations

Legacy graphs gain no serialized fields when they contain no typed entity
contracts. Existing application compatibility and projection suites pass.
Records and operations are deliberately not copied into graph contracts, so
unrelated application-only diagnostics cannot change legacy projection
behavior. There are no unresolved implementation failures or scope deviations.
