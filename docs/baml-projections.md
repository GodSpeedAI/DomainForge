# BAML Projection (`--format baml`)

`domainforge project --format baml --recipe recipe.json model.sea out/` compiles
a `.sea` model **plus its authority environment** into a
[BAML](https://docs.boundaryml.com/) project: typed enums/classes for the
model's closed domains, one typed policy-decision function whose prompt embeds
the governing policy semantics, and a `test` block for every resolver-grounded
authority case. The LLM client is left as a documented placeholder so the
generated code is vendor-neutral.

The projection is deterministic: a fixed `--created-at` yields byte-identical
output, and every id is minted through the shared `projection::ids` kernel.

## AI-representation operator

BAML is the AI-representation operator of the projection family: it renders the
canonical model as a typed, testable LLM capability. It is part of the AI family
(with DSPy and ZenML) and reuses the ai-learning `AiLearningContext` — the same
resolver-grounded `AuthorityCase`s and `PolicyInfo` the ai-learning datasets are
built from. BAML never re-derives authority semantics; its examples are the
resolver's own decisions.

## What gets generated

Output is a directory (the conventional BAML `baml_src/` layout plus a README):

| File | Contents |
| --- | --- |
| `baml_src/domain.baml` | `enum`s for the closed entity/role/resource/operation domains and a fixed `AuthorityDecision` enum; the `AuthorityRequest` (input) and `AuthorityRuling` (output) `class`es |
| `baml_src/functions.baml` | the `function DecideAuthority(request: AuthorityRequest) -> AuthorityRuling` capability; its prompt lists the governing policies |
| `baml_src/tests.baml` | one `test` block per resolver-grounded authority case, seeded with the exact `(role, operation, resource_type)` tuple and the resolver's decision label |
| `baml_src/clients.baml` | commented-out `client<llm>` and `generator` placeholders |
| `README.md` | layout and the two edits needed to make the project runnable |

### Model → BAML mapping

| Model element | BAML construct |
| --- | --- |
| Roles | `enum ActorRole` variants |
| Resources | `enum ResourceType` variants |
| Entities | `enum DomainEntity` variants |
| Authority pack actions | `enum Operation` variants |
| Resolver decision set | `enum AuthorityDecision` (Allow/Deny/Escalate/Unknown/Reject) |
| Policy-decision capability | `function DecideAuthority` (input = authority request shape, output = decision + rationale + policy refs) |
| Authority policies | prompt-embedded governing-policy list |
| Authority cases (resolver decisions) | `test` blocks (the examples) |

By default only cases with an applicable policy (allow/deny/escalate/reject) become
tests; set the recipe's `baml.include_unknown` to also emit tests for tuples the
resolver labels `unknown`.

## Recipe

BAML reuses the one ai-learning recipe system — there is no second recipe format.
Its section is minimal:

```json
{
  "name": "my_baml_v0",
  "authority_config": "../authority/environment.json",
  "projections": {
    "baml": {
      "enabled": true,
      "function_name": "DecideAuthority",
      "include_unknown": false
    }
  }
}
```

`function_name` must be a valid BAML identifier (validated). The authority
environment may instead be supplied on the CLI with `--authority-config`; one of
the two is **required** because the function and its tests are resolver-grounded.

## No vendor lock-in

The `function` references a client named `DomainForgeAuthorityClient` that is
deliberately **not defined** — `baml_src/clients.baml` carries it (and a
`generator` block) as commented-out placeholders. You choose a provider
(OpenAI, Anthropic, Google, a local Ollama model, …), fill in the client, and
supply credentials through an environment variable. No provider or key is ever
baked into the generated code.

## Determinism

- All domains are enumerated in sorted order; policy lines iterate a `BTreeMap`;
  tests are sorted by `(decision, role, operation, resource_type)`.
- No wall-clock or randomness enters file contents — only the caller-supplied
  `created_at`.
- Every id (the capability id, per-test ids) is minted through
  `projection::ids::element_id` under the `baml` family tag, so renaming a model
  element changes exactly the ids derived from it.

## Validation

The pinned target BAML syntax revision is recorded in `BAML_TARGET_VERSION`
(`domainforge-core/src/projection/baml/ir.rs`). Bumping it is a constant change
plus a green `verify-baml` CI run — the same policy as `LEAN_TOOLCHAIN`.

**What is live-validated:** structural well-formedness of the emitted `.baml`
(`schemas/baml/check_baml.py`): balanced `{}`/`[]`/`()`, matched `#"`/`"#` raw
strings, known top-level block keywords, every function-referenced type resolves
to a defined class/enum or builtin, and every `test` binds an existing function.

**What is NOT live-validated (and why):** BAML's own `baml-cli generate` is not
run in CI. Because the generated client is an intentional placeholder (for
vendor-neutrality), `baml-cli generate` would fail on the undefined client, and
baking a specific vendor into the fixture purely to satisfy the generator would
defeat the goal. This is the same substitution the OTel projection makes for
`weaver` (see [OTel SemConv Projection](otel-projections.md) and
`schemas/baml/VENDORED.md`). Prompt-template semantics and actual client
generation are the user's responsibility once they fill in a client.

To validate locally:

```bash
domainforge project --format baml --recipe fixtures/baml/basic/recipes/baml.json \
  --created-at 2026-07-02T00:00:00+00:00 fixtures/baml/basic/domain/model.sea out/
python3 schemas/baml/check_baml.py out/baml_src
```

## Non-goals (v1)

- **No client/provider generation.** Vendor selection is the user's; the
  projection stays provider-agnostic.
- **No prompt tuning.** The prompt states the policies and the decision contract;
  optimizing it (or evaluating outputs) is downstream work — see the DSPy
  projection for optimization.
- **No new DSL syntax.** BAML consumes the graph and authority environment as-is.

## Implementation

- IR: `domainforge-core/src/projection/baml/ir.rs` (`AICapabilityIR`).
- Renderer: `domainforge-core/src/projection/baml/render.rs`.
- `emit` / `project_baml_in_memory`: `domainforge-core/src/projection/baml/mod.rs`.
- Recipe section: `baml` in `domainforge-core/src/projection/ai_learning/recipe.rs`.
- Fixture: `fixtures/baml/basic/` (reuses the manufacturing-quality authority
  environment).
- Tests: `domainforge-core/tests/baml_projection_tests.rs`, `tests/test_baml.py`,
  `typescript-tests/baml.test.ts`.
- CI: the `verify-baml` job in `.github/workflows/ci.yml`.
