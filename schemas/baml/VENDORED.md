# BAML validation assets

## `check_baml.py` — structural well-formedness checker

DomainForge's `--format baml` projection emits a BAML project whose
`client<llm>` block is a **documented, commented-out placeholder** so the
generated code is vendor-neutral (no provider or credentials baked in). Because
the referenced client is intentionally undefined, BAML's own
`baml-cli generate` cannot run against the output as-shipped — it fails on the
missing client.

Rather than bake a specific LLM vendor into the fixture purely to satisfy the
generator (which would defeat the vendor-neutrality goal), CI validates the
emitted `.baml` **structurally** with `check_baml.py`:

- balanced `{}`, `[]`, `()` (ignoring string, comment, and raw-prompt content),
- matched `#"` / `"#` raw-string delimiters,
- every top-level block opens with a known BAML keyword,
- every type the function references resolves to a defined `class`/`enum` or a
  builtin, and
- every `test` binds a function that exists.

This is the same substitution pattern documented for the OTel projection
(`weaver` → JSON-Schema check; see `docs/otel-projections.md`). What is **not**
live-validated: prompt-template semantics and actual LLM client generation —
both require a user-chosen provider and are the user's responsibility once they
fill in `baml_src/clients.baml`.

## Pinned target syntax

The generated source targets the BAML syntax revision recorded in
`BAML_TARGET_VERSION` (`domainforge-core/src/projection/baml/ir.rs`) and
`docs/baml-projections.md`. Bumping the target is a constant change plus a green
`verify-baml` CI run.
