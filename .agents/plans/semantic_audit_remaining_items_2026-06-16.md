# Semantic Audit Remaining Items Plan

**Goal:** Resolve the two open items from the 2026-06-16 semantic infrastructure audit verification.

## Scope

1. Make `conformance/08_authority/facts.json` stable across wall-clock time so authority parity remains green after fixture generation day.
2. Resolve G8 by wiring breaking `ConceptChange` compatibility into canonical validation rather than leaving it as an opt-in API.

## Tasks

1. Add a red conformance item for breaking `ConceptChange` plus an active policy.
   - Expected outcome: the corpus expects a canonical validation error for breaking evolution, and current code fails before implementation.

2. Update canonical validation.
   - Expected outcome: `Graph::validate()` and `validate_to_canonical_json()` include a deterministic ERROR-severity violation when breaking concept changes coexist with active policies.

3. Stabilize `08_authority` facts.
   - Expected outcome: committed facts no longer expire; fixture pinning tests compare the same stable shape used by Python, TypeScript, and WASM parity tests.

4. Refresh audit state.
   - Expected outcome: the audit report and `.agents/current_state.md` mark the remaining blockers resolved with command evidence.

5. Run focused gates.
   - Expected outcome: Rust corpus/authority tests, Python parity, TypeScript parity, and WASM tests pass.
