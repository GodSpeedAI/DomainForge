# Time-Sensitive Conformance Fixtures

Avoid committing conformance inputs that expire relative to wall-clock time.

Observed failure:

- `conformance/08_authority/facts.json` contained an `expires_at` value from 2026-06-14.
- On 2026-06-16, Python, TypeScript, and WASM authority parity loaded the fixture literally and produced `reason: "stale_fact"` instead of the golden trusted-fact trace.
- Rust fixture regeneration still passed because it built fresh facts with `Utc::now() + 1 hour` and normalized volatile timestamp fields.

Preferred pattern:

- Use non-expiring facts when a fixture is meant to stay trusted indefinitely.
- If expiry semantics are the behavior under test, pin the test's evaluation clock or make the stale/active boundary explicit.
- Keep fixture builders aligned with committed fixtures so regeneration does not reintroduce time drift.
