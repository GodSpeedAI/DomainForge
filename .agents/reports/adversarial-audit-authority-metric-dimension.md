# Adversarial Audit: Policy Authority, Metric, Dimension/Units, Flows

Date: 2026-07-10 · Branch: `agent/projection-targets`
Scope: `domainforge-core/src/authority/*`, `policy/*`, `units/mod.rs`, `primitives/metric.rs`, `projection/flows.rs`

Severity: 🔴 exploitable / correctness-critical · 🟠 debt that will bite · 🟡 inefficiency/polish

---

## 1. Policy Authority (`src/authority/`)

### 🔴 A1. Signatures are never verified — only presence-checked

`types.rs:258` (`is_satisfied_by`) and `fact_resolver.rs:182` check `fact.signature.is_none()`.
Any non-empty string satisfies `signature_required: true`. `"signature": "lol"` passes.
The entire attestation chain is decorative.
**Mitigate:** verify against a registered public key per `FactSource` (ed25519 over a canonical
JSON serialization of `{path, value, observed_at, source_id}`), or rename the field
`signature_present` and document loudly that it is not cryptographic. Do not ship the middle ground.

### 🔴 A2. Fact trust is default-open: envelopes self-declare `source_class`

`is_satisfied_by` trusts `envelope.effective_trust()` — the envelope's own claim. The registry
cross-check (`source_class_mismatch`) happens only in `FactResolver::resolve_trusted_facts`, and
only for facts routed through the `provided_trusted` parameter. Any envelope handed directly to
`AuthorityResolver::resolve()` in `facts` is trusted at whatever class it claims
(`resolver.rs:41-48` takes `facts: &[FactEnvelope]` with no provenance gate).
**Mitigate:** make `AuthorityResolver::resolve` accept only the output of `resolve_trusted_facts`
(newtype `TrustedFacts(Vec<FactEnvelope>)` so the type system enforces the pipeline), or re-run
the registry check inside `resolve`.

### 🔴 A3. Conditions read caller-supplied facts unless the policy opts in

`ConditionPredicates::evaluate` (`policy.rs:92`) looks facts up by path with **no source-class
filter**. A policy with `when: {"kyc.verified": true}` but no matching `requires_fact` is
satisfied by a caller who puts `kyc: {verified: true}` in the request context
(`wrap_context_as_caller_supplied` mints those envelopes). Forgetting `requires_fact` = trusting
the attacker. **Mitigate:** lint/compile error when a `when` path has no `requires_fact` entry,
or default conditions to only consume facts at `SystemVerified`+ unless explicitly opened.

### 🔴 A4. `required_transform` is parsed, stored, and never enforced

`compiler.rs:114` populates it; `is_satisfied_by` (`types.rs:238-293`) never reads it. A policy
author who writes `required_transform: "pii_redaction"` gets nothing, silently.
Same for `_derived_lineages` — `resolve()` accepts it and ignores it (`resolver.rs:47`).
**Mitigate:** enforce or reject at compile time ("unsupported field"). Silent acceptance of a
security field is the worst option.

### 🔴 A5. Compatibility lowering is a string hack with a Unicode panic

`compiler.rs:165-249` (`audit_expression`):

- Byte offsets computed on `expr.to_lowercase()` are applied to the **original** string
  (`expr[4..][..eq_pos]`, and the `" and "` splitter). For non-ASCII input where lowercasing
  changes byte length (e.g. `İ` → `i̇`), this slices at wrong/invalid boundaries → garbage keys or
  **panic**.
- `" or "` / `" and "` inside quoted values misfire: `role = "black or white"` is rejected as
  ambiguous; `name = "rock and roll"` splits into two bogus predicates.
- Only `=` is understood; `!=`, `<`, `>=` return `Ok(None)` — expression silently not lowered.
**Mitigate:** a 50-line real tokenizer (respect quotes, char-indices), and make `Ok(None)` an
error in strict mode. This is a policy compiler; "silently didn't compile your policy" is a
correctness hole, not a convenience.

### 🟠 A6. Modality precedence uses declared modality, not effective decision

`resolve_conflicts` (`resolver.rs:204-236`) ranks by `policy.modality` even when the policy's
result came from unknown-handling. A Prohibition with **missing facts** whose unknown-default is
`Escalate` (rank 4) beats a Permission whose conditions are **definitively true** (rank 2).
May be intended fail-closed, but it means one un-fetchable fact on any broad prohibition
escalates everything it touches. Document it or rank by effective decision.

### 🟠 A7. Diagnostics lie

- `classify_unknown_reason` (`resolver.rs:389`): the envelope in an unsatisfied fact-check is
  **always `None`** (`check_fact_requirements` only stores the envelope when satisfied), so the
  `caller_omission` branch is dead and every unknown reads `"missing"`.
- `classify_availability` returns `"caller_omission"` for a missing envelope — the exact opposite
  classification of its sibling function for the same state.
- `incomparable_policies` in `ResolverOutput` is always `vec![]` even when the
  `specificity_incomparable` step fires (`resolver.rs:316`).
- `unknown_decision.fact_source_ids` is always empty (`resolver.rs:98`).
Operators debugging a denial will be actively misled. **Mitigate:** store the *nearest rejected*
envelope (path match, requirement failed) in `check_fact_requirements` and derive reasons from it;
populate `incomparable_policies` from the non-dominated set.

### 🟠 A8. Obligations evaluate to plain `Allow`

`modality_to_decision` maps `Obligation → Allow` and `obligation_spec` (action, deadline) never
appears in `ResolverOutput`. The caller cannot know an obligation was incurred, which is the whole
point of the modality. **Mitigate:** add `obligations: Vec<ObligationSpec>` to the output.

### 🟠 A9. `parse_duration` foot-guns (`compiler.rs:122`)

- Bare number defaults to **hours** (`"300"` = 300 h) — surprising; seconds or an error is safer.
- Negative durations accepted (`"-5h"` → every fact is "fresh").
- `"10ms"` hits the `s` branch, becomes `"10m"`, fails to parse → confusing error.
**Mitigate:** reject bare numbers and negatives; match suffixes longest-first. ~10 lines.

### 🟡 A10. Misc

- `StructuralPredicates::matches`: unknown keys silently fall through to `metadata` — a typo like
  `"resource.typ"` makes the policy permanently inert with no diagnostic. Warn on unmatched
  structural-looking keys at compile time.
- A predicate expecting `null` matches an *absent* actor role (`policy.rs:40-45`) — "no role"
  is matchable, probably unintended.
- Specificity is a raw predicate count per dimension (`policy.rs:71-82`) — gameable by stuffing
  redundant `resource.*` metadata keys to win ties.
- `validate_versions` is string equality; no semver range logic despite "compatibility" naming.
- Specificity vectors are recomputed inside the O(n²) domination loop (`resolver.rs:280-285`) —
  compute once per policy before the loop.
- Duplicate fact paths: `find` takes the first envelope; ordering between raw and trusted facts is
  load-bearing (`resolve_trusted_facts` replaces in place — OK today, fragile).

---

## 2. Metric (`primitives/metric.rs`)

### 🟠 M1. `unit: Option<String>` is stringly-typed and never validated

Nothing ties `Metric.unit` to the `UnitRegistry` at parse/validate time. Combined with U1 below,
a typo'd unit flows all the way to projections as a fake `Count` unit. **Mitigate:** resolve the
unit against the registry in `validate` and error on unknown symbols.

### 🟠 M2. Threshold/target/severity carry no direction or semantics

`threshold: Option<Decimal>` — above or below? `target` vs `threshold` relationship? Nothing
enforces `severity` requires `threshold`. Every projection target must guess, and they will guess
differently. **Mitigate:** `threshold: Option<(Comparator, Decimal)>` or a documented invariant
checked in validation.

### 🟡 M3. `refresh_interval`/`window` use `chrono::Duration` (signed) — negative windows are

representable and unvalidated.

---

## 3. Dimension / Units (`units/mod.rs`)

### 🔴 U1. `unit_from_string` silently converts typos into real units

`units/mod.rs:447-462`: unknown symbol → fabricate a `Count` unit with factor 1. Two typo'd units
convert 1:1 with no error; one typo'd unit vs a real one yields `IncompatibleDimensions` blaming
the wrong thing. In a domain-modeling tool whose value proposition is semantic rigor, this is the
single most corrosive silent default in the audited surface. **Mitigate:** return
`Result<Unit, UnitError>`; add `unit_from_string_lossy` only if some call site truly needs it.

### 🟠 U2. Temperature dimension exists but is unimplementable

`Dimension::Temperature` is declared; no temperature units are registered, and the linear
`value * base_factor` model (`mod.rs:134-142`) cannot express affine conversions (°C↔°F needs an
offset). First user to add "F" via `register_from_json` gets silently wrong conversions.
**Mitigate:** add an `offset` field to `Unit` (one line in the math).

### 🟠 U3. Registration APIs bypass invariants

- `Unit::new` (public) accepts `base_factor: 0` — division by zero in `convert_from_base`.
  Only the JSON path checks `ZeroBaseFactor`.
- `register` never checks that `base_unit` exists or that its dimension matches — you can register
  `"furlong"` with base `"kg"` in `Length` and `convert` will happily multiply factors.
- `register_dimension` inserts an **empty-string** base unit (`or_default`), poisoning
  `base_units()` consumers.
**Mitigate:** validate in `register` (nonzero factor, base unit exists or is self, dimensions
agree). ~15 lines, closes all three.

### 🟡 U4. Global mutable registry

`UnitRegistry::global()` RwLock singleton means one model's custom units leak into every other
model in the same process (LSP/server usage). Fine for CLI; a landmine for the WASM/python
bindings. Note it in a comment now; thread a registry handle when it first bites.

### 🟡 U5. Currency is half-modeled: EUR/GBP have `base_factor 1` against themselves and the

Currency base unit is "USD". Correct today only because `convert` special-cases Currency; any new
code path using `base_factor` directly will convert EUR→USD at 1.0. Set them apart structurally
(e.g., each currency its own dimension) or guard in `convert_to_base`.

---

## 4. Flows / projection cell (`projection/flows.rs`)

Genuinely good module — small, blessed, loud on dangling references. Remaining gaps:

### 🟠 F1. The dangling-reference test doesn't test dangling references

`collect_flows_errors_on_dangling_reference` (`flows.rs:129-139`) admits in a comment that it
verifies the empty case instead. The load-bearing error path of the module has **zero coverage**.
**Mitigate:** build a `Graph` directly (bypass parser), insert a flow with a bogus resource id,
assert the error. ~8 lines.

### 🟡 F2. Display-name collisions: flows are resolved to *names*, and two distinct entities with

the same name (possible? depends on graph invariants) would merge silently in every projection.
If `Graph::add_entity` already rejects duplicate names per namespace, add a one-line comment here
citing that invariant so the next reader doesn't have to re-derive it.

### 🟡 F3. Errors are `String` — every one of the eight targets must invent its own wrapping

A tiny `enum FlowError { Dangling(..), MultiNamespace(..) }` with `Display` gives structured
handling for free.

---

## 5. Low-effort / outsized-reward list (the overlooked stuff)

Ranked by (value ÷ effort). Most are < 30 lines.

1. **Make `unit_from_string` fallible (U1).** One signature change; converts an entire class of
   silent data corruption into compile-guided error handling at every call site. The compiler
   does the migration work for you.
2. **Newtype `TrustedFacts` (A2).** ~20 lines. The type system then *proves* every decision went
   through the trust pipeline — eliminates the whole "someone called resolve() with raw facts"
   bug class forever, with zero runtime cost.
3. **Compile-time lint: `when` path without `requires_fact` (A3).** One loop in
   `PolicyCompiler::compile_one`. Turns the default-open trust gap into an authoring error.
4. **Reject unenforced security fields (A4).** If `required_transform` isn't implemented, error
   on it. 3 lines. "We refuse what we can't enforce" is a property auditors pay for.
5. **`cargo test -- --include-ignored` + `#[deny(dead_code)]`-style CI grep for `vec![]`
   placeholder fields** (A7): populate-or-delete `incomparable_policies` and `fact_source_ids`.
   Fields that always serialize empty are worse than absent — consumers write code against them.
6. **Fuzz the two hand-rolled string parsers** (`parse_duration`, `audit_expression`) with
   `cargo-fuzz` — 20 minutes of setup; A5's Unicode panic falls out immediately. Hand-rolled
   parsers at a trust boundary are exactly what fuzzing is for, and almost nobody fuzzes
   "little" internal parsers.
7. **Property test the resolver's conflict ladder** with `proptest`: generate random policy sets,
   assert invariants ("adding an irrelevant policy never changes the decision", "Deny never
   downgrades to Allow when facts are removed"). ~40 lines, and it's the kind of test that finds
   the bugs integration tests structurally cannot. Monotonicity-under-fact-removal is the
   fail-closed property regulators actually ask about.
8. **Snapshot-test `ResolverOutput` JSON** (`insta` crate) for 3–4 canonical scenarios. Decision
   traces are your audit artifact; today nothing pins their shape, so a refactor can silently
   change `reason_code` strings that downstream systems parse.
9. **`#[non_exhaustive]` on `FinalDecision`, `PolicyModality`, `UnitError`** — free forward
   compatibility for a published core crate; retrofitting it later is a semver break.
10. **One `debug_assert!` in `convert`** that `from.base_unit == to.base_unit` — catches U3's
    mismatched-base registrations in every dev/test run at zero release cost.

---

## Summary table

| Area | 🔴 | 🟠 | 🟡 |
|---|---|---|---|
| Authority | A1 sig-not-verified, A2 default-open trust, A3 caller-fact conditions, A4 unenforced fields, A5 lowering parser | A6 modality vs decision, A7 lying diagnostics, A8 obligations dropped, A9 durations | A10 misc |
| Metric | — | M1 unvalidated unit, M2 threshold semantics | M3 signed durations |
| Units | U1 silent Count fallback | U2 temperature, U3 invariant bypass | U4 global registry, U5 currency |
| Flows | — | F1 untested error path | F2, F3 |

The theme across all four areas: **silent defaults at trust and validation boundaries**
(unverified signatures, self-declared source classes, fabricated units, unlowered expressions,
unenforced fields). Each individually looks like pragmatism; together they mean the system's
strongest guarantees hold only on the happy path. The cheapest systemic fix is the pattern in
items 1–4: make the type system or the compiler refuse the bypass, rather than documenting it.
