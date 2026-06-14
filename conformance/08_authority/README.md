# 08_authority (roadmap — not yet wired)

**Goal (audit §5 item 8):** extend `fixtures/semantic_packs/acme_procurement` with
two conflicting policies from sources of different specificity and one fact from a
`signature_required` source, then pin the full `AuthorityTrace` JSON — all seven
hashes (`ir_hash`, `pack_hashes`, `resolver_semantics_hash`,
`specificity_profile_hash`, `unknown_handling_config_hash`, `action_request_hash`,
`derived_fact_lineage`), the `conflict_resolution_steps`, and
`unknown_handling_applied` — byte-stable.

**Why it is not wired yet:** the `sea authority` oracle needs hand-authored
`config.json` / `request.json` / `facts.json` inputs (an `AuthorityEnvironmentConfig`
schema), and the high-value proof for it — trace byte-stability **across binding
serialization** — is audit **Phase 6 (Provenance hardening)**, after the
cross-language parity matrix (Phases 2–3) exists. The authority module is already
`proven` within Rust (`tests/authority_conformance_tests.rs`).

This directory has no `manifest.json`, so the conformance harness skips it. Add one
(pointing at an `authority` oracle) when the golden-trace fixture lands.
