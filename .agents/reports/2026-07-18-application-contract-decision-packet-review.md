# Review: SEA Application Contract Decision Packet

Date: 2026-07-18

Artifact:
`.agents/reports/2026-07-18-application-contract-decision-packet.md`

## Verdict

Accept the overall direction, but do not accept Human Gate A unchanged. D2 and
D10 are acceptable as written. D1, D3, D4, D5, D6, D7, D8, and D9 need the
amendments below before approval is recorded.

**Disposition:** Complete. On 2026-07-18 the DomainForge repository maintainer
explicitly instructed the agent to accept the recommendations with these
amendments. The decision packet now records all ten accepted decisions and
Human Gate A is closed.

## Required amendments

1. **D1:** Split the recommendation into two explicit decisions: entities have
   optional typed state bodies; operation DTOs use a mandatory, identityless,
   flat `record` declaration. Remove “if maintainers want.”
2. **D3:** Define one module-resolution output and symbol table from which both
   Graph and `ApplicationContract` are built. Specify aliases, exports,
   qualified IDs, cycles, collisions, origins, and deterministic closure order.
3. **D4:** Define the policy evaluation context and fail-closed result mapping.
   Restrict orphan diagnostics to application-scoped policy references; legacy
   or general policies remain valid. Keep authentication outside v0.1.
4. **D5:** Replace “Option 1 plus enum” with one actual closed option. Define
   enum encoding/evolution, constraint applicability, string/regex
   normalization, list/reference nullability, and distinguish concept identity,
   aggregate key, DTO fields, and foreign references. Either add a unit-bearing
   quantity type or remove the flagship's typed-quantity claim.
5. **D6:** Replace unnamed closed sets with exhaustive v0.1 values and
   invariants for effects, transactions, failures, concurrency, idempotency,
   evidence, lifecycle, and `not_applicable` eligibility.
6. **D7:** Follow the repository's additive AST policy unless ADR-013 explicitly
   changes it: retain `ast-v3` for additive variants, introduce
   `domainforge-application-contract/v1`, and state the exact Rust and
   Python/TypeScript/WASM JSON/typed API matrix.
7. **D8:** Do not equate reserved keywords with additive compatibility. Use
   contextual, full-declaration lookahead and exhaustive collision tests, or
   classify the change as breaking with migration. Define regression as the
   same pre/post formatter output, not byte equality with authored input.
8. **D9:** Keep order management, but replace external payment authorization
   with a local deterministic policy. Add a field-by-field acceptance table for
   every Section 5.1 item, exact command/query payloads and results, failures,
   transaction and concurrency behavior, persisted rows, rollback/restart,
   projection status/reasons, and canonical expected outputs.

## Recommended flagship adjustment

Use `place_order` with a local `order_total_within_limit` precondition. Model
restart-safe idempotency by `client_order_id`: same key and same canonical input
returns the stored result; same key with different canonical input returns
`duplicate_order`. Keep `get_order_status` public only for an opaque order ID
and a response containing no sensitive fields. This exercises policy,
conflict, persistence, and restart behavior without importing an authentication
or external payment-authority system into v0.1.

## Review evidence

The packet was checked against the governing spec, SEA language evolution
policy, grammar ambiguity around `declaration_keyword`, AST v3 schema and
versioning documentation, current Graph/module resolution surfaces, policy
evaluation API, bindings, and semantic-pack canonicalization. Two independent
fresh-context reviews converged on the same D1, D4-D9 issues; one additionally
identified D3's unresolved symbol-table boundary. Cross-model review was
offered and not invoked.
