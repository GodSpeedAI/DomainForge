# tmp/cr.md Code Review Remediation

Date: 2026-06-14

## Scope

Remediate the two review prompts in `tmp/cr.md` by verifying each finding against
current code, fixing only still-valid issues, and validating the result.

## Findings

- `sea-core/tests/conformance_corpus_tests.rs` flow normalization depends on
  `serde_json::Map` key iteration preserving JSON parse order. This is valid:
  without `serde_json/preserve_order`, the map sorted UUID keys and could map
  random flow IDs to different positional placeholders.
- `sea-core/src/cli/validate.rs` serializes `"error": err` in policy evaluation
  JSON. This is not currently a valid behavioral issue because
  `Policy::evaluate_with_mode` returns `Result<EvaluationResult, String>`, so
  serde already serializes `err` as a JSON string.

## Implementation

- Add a regression test proving flow-key parse order is preserved before
  normalization.
- Enable `serde_json`'s `preserve_order` feature for `sea-core` dev builds.
- Leave the validate JSON error field unchanged because the review rationale does
  not apply to the current `String` error type.

## Validation

- Red: `cargo test -p sea-core --features cli --test conformance_corpus_tests serde_json_preserves_flow_key_parse_order_when_normalizing_uuid_maps -- --nocapture`
  failed before the Cargo feature change with sorted UUID keys.
- Green: the same targeted test passed after enabling `preserve_order`.
- Passed: `cargo test -p sea-core --features cli --test conformance_corpus_tests -- --nocapture`.
- Passed: `cargo test -p sea-core --features cli --test cli_tests -- --nocapture`.
- Passed: `rustfmt --check sea-core/tests/conformance_corpus_tests.rs`.
- Not passed: `cargo fmt --all -- --check` still reports a diff in the pre-existing
  modified file `sea-core/src/cli/validate.rs`, which this remediation did not
  rewrite.
