# authority — trust, provenance & decision traces

The trust surface: turns semantic packs + facts into auditable, hash-anchored
decisions. Strongest-proven module in the crate (within Rust).

## Files

| File | Role |
|------|------|
| `types.rs` (617) | Core authority data types |
| `resolver.rs` (446) | `AuthorityResolver`: conflict resolution → explicit `ConflictResolutionStep`s; errors on `SpecificityConflict` (never silently picks) |
| `compiler.rs` | Compiles packs/policies into the resolver's evaluable form |
| `policy.rs` | Authority-policy representation |
| `fact_resolver.rs` | `FactResolver`/`FactSourceRegistry`; `signature_required` sources reject unsigned facts (`missing_signature`) |
| `trace.rs` | `AuthorityTrace`: hashes the full decision context (see below) + `derived_fact_lineage` |
| `transform.rs` | `DerivedFactEngine`/`FactTransformRegistry` (derived facts) |
| `environment.rs` | `AuthorityEnvironment(Config)` — runtime config |
| `pack.rs` | `AuthorityPack`, `compute_pack_hash` |
| `error.rs` | `AuthorityError`/`AuthorityErrorCode` |

## Trace = the contract

`AuthorityTrace` hashes: `ir_hash`, `pack_hashes`, `resolver_semantics_hash`,
`specificity_profile_hash`, `unknown_handling_config_hash`, `action_request_hash`,
plus `derived_fact_lineage` and `conflict_resolution_steps`. This is what downstream
(SEA-Forge) must be able to replay byte-for-byte.

## Gotchas

- Conflicts **halt or trace, never silently resolve** — preserve this when editing
  the resolver.
- Trace byte-stability across binding serialization is **unproven** (audit Phase 6);
  do not assume a binding produces identical trace JSON yet.
- Packs self-verify via `validate_hash()`; tampering → `pack_hash_mismatch`.
- Related: `../semantic_pack/` produces the packs this module consumes.
- CLI entry: `cli/authority.rs` (`sea authority config.json request.json --facts … --json`).
