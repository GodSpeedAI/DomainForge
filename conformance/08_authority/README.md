# 08_authority — **Wired** (authority provenance golden)

**Goal (audit §5 item 8):** pin the full `AuthorityTrace` JSON — all seven hashes
(`ir_hash`, `pack_hashes`, `resolver_semantics_hash`, `specificity_profile_hash`,
`unknown_handling_config_hash`, `action_request_hash`, and the per-policy lineage)
plus `conflict_resolution_steps`, `candidate_policies`/`applicable_policies`, and
the `final_decision` — byte-stable.

## What is pinned

| File | Contents |
| --- | --- |
| `packs.json` | The two input packs: a prohibition (`shipping-policy`, priority 100) and a permission (`shipping-permission`, priority 50) of differing specificity. |
| `trace.json` | The full `AuthorityTrace` for a `ShipOrder` request on a credit-`Hold` order. The prohibition applies and **denies**; the permission is a structural *candidate* excluded by its `when` condition (`credit_status == Clear`). `pack_hashes` has two entries, both policies appear in `candidate_policies`, and `conflict_resolution_steps` records the resolution. |
| `decision.json` | The `AuthorityDecision` (`deny`, `reason_code = denied_by_block_credit_hold_shipping`). |
| `config.json`, `request.json`, `facts.json` | The shared cross-binding evaluation inputs. `facts.json` intentionally omits `expires_at` so the committed trusted fact does not drift into `stale_fact` as wall-clock time advances. |
| `manifest.json` | `command: "authority_trace"` — skipped by the `parse`/`validate` loaders (Rust corpus runner, Python and TS parity), like `10_kg_roundtrip` and `12_seaforge_fixture`. |

## How it is enforced

`sea-core/tests/authority_fixture_tests.rs` (1) validates `packs`/`trace`/`decision`
against `$defs/{pack,trace,decision}` in `schemas/seaforge-contract-v1.json`, and
(2) regenerates each from the builders, normalizes only the genuinely volatile
fields (wall-clock timestamps, random `decision_id`/`trace_ref`), and asserts
byte-equality with the committed files.

**Hashes are pinned byte-for-byte** — not normalized. This is possible because
authority hashing now flows through `canonical_json_string` (sorted-key
serialization, `sea-core/src/authority/types.rs`), making `compute_pack_hash` and
`AuthorityRequest::action_hash_input` deterministic regardless of `HashMap`
iteration order or the `serde_json` `preserve_order` feature. A drift in any hash,
policy, or decision field now fails the test.

Regenerate after a deliberate shape change:

```bash
cargo test --features cli --test authority_fixture_tests generate_fixtures -- --ignored
```

## Cross-binding parity (wired)

The shared evaluation inputs (`config.json`, `request.json`, `facts.json`) are
committed alongside the goldens so every binding drives byte-identical inputs
through the same core and byte-matches the same `trace.json`/`decision.json`:

| Surface | Binding entry point | Parity test |
| --- | --- | --- |
| Python | `AuthorityEnvironment(config).evaluate(request, facts)` | `tests/test_authority_parity.py` |
| TypeScript | `evaluateAuthority(config, request, facts)` | `typescript-tests/authority-parity.test.ts` |
| WASM | `evaluateAuthority(config, request, facts)` | `sea-core/tests/wasm_tests.rs::test_authority_08_trace_parity` |

Each test loads the committed inputs, calls the binding, parses the emitted
`trace`/`decision` JSON, normalizes only the genuinely volatile fields
(`created_at`, `decision_id`, `trace_ref`), and asserts structural equality
with the golden. All seven hashes and every policy/decision field stay pinned.
The shared trusted fact is non-expiring, so these parity checks are stable across
calendar time.
