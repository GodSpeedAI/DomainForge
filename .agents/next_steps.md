# Next Steps

## 1. Create and execute a Milestone 0 remediation plan

Turn every Critical and High finding in
`.agents/reports/2026-07-19-m0-human-review-gate.md` into test-first tasks,
preserving ADR-013 semantics. Expected outcome: strict field/default
validation, shared resolved identities, normalized canonical envelopes, strict
schemas, complete persisted-artifact validation, and bounded public inputs.

## 2. Produce independent compatibility and parity evidence

Restore or explicitly adjudicate the pre-change oracles and `std/core.sea`
visibility, exercise non-empty signed semantic packs, fix the D9 contract and
envelope goldens, and compare rebuilt Rust/Python/TypeScript/WASM outputs from
the same inputs. Expected outcome: stable accepted oracle values and
byte-identical cross-target artifacts that cannot pass against stale bindings.

## 3. Re-run the Milestone 0 human gate

Pass every focused suite, `just all-tests`, Clippy, formatting, genuine WASM
validation, and `git diff --check`; present a finding-to-test/commit matrix and
all required hashes. Expected outcome: an evidence-backed accept/reject
decision and, only if explicitly accepted, authorization to begin Milestone 1.
