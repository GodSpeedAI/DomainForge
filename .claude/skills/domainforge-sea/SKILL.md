---
name: domainforge-sea
description: Master DomainForge SEA DSL authoring and DomainForge CLI workflows. Use when writing, editing, validating, formatting, parsing, projecting, importing, evaluating, packaging, signing, or troubleshooting `.sea` files, semantic packs, policies, metrics, modules/imports, units, mappings, projections, or DomainForge CLI commands in any project.
---

# DomainForge SEA

## Overview

Use this skill for DomainForge's `.sea` DSL, not the separate SEA-Forge repository. Treat `.sea` as the DomainForge Semantic Enterprise Architecture DSL.

This skill is portable. It bundles the DomainForge SEA and CLI references under `references/` so agents can use it outside the DomainForge repository.

## Bundled References

Load only the reference needed for the task:

- `references/write-in-sea.md`: read when authoring or repairing `.sea` files by hand. It covers file shape, declarations, policy syntax, metrics, imports, mapping/projection contracts, semantic validation, and troubleshooting.
- `references/cli-commands.md`: read when running the DomainForge CLI, selecting flags, interpreting exit codes, projecting/importing formats, using semantic packs, or scripting CI.
- `references/sea-dsl-ai-cheatsheet.yaml`: read when an agent needs dense syntax, primitive, expression, API, error-code, projection, or anti-pattern details.

When working inside the DomainForge repository, also follow that repo's `AGENTS.md` and shared `.agents/` state before making material changes.

## Operating Loop

1. Classify the request: author `.sea`, fix validation, inspect graph/AST, project/import, evaluate policy, work with semantic packs, or change DomainForge implementation.
2. Read this file, then load the smallest bundled reference set needed for the task.
3. Inspect nearby `.sea` examples or user files before inventing style.
4. Draft or edit `.sea` with metadata first, declarations before references, and policies/metrics after graph elements.
5. Prefer the `domainforge` binary. If the user or older docs say `sea`, verify that `sea` exists locally before using it; otherwise translate to `domainforge`.
6. Validate with the CLI. Use JSON output when tooling or exact diagnostics matter.
7. Format with `domainforge fmt --check` or `domainforge fmt --out` when the formatter supports the file.
8. For projection/import/pack work, run the target command and validate generated outputs when the reference gives a validator.
9. Report changed files and exact proof commands with pass/fail outcomes.

## SEA Authoring Checklist

Use this checklist before writing or changing `.sea`:

- Put optional header annotations first: `@namespace`, `@version`, `@owner`, `@profile`. Header values are strings.
- Use current import forms only: `import { Name, Other as Alias } from "module"` or `import * as alias from "module"`.
- Use `export` by wrapping a normal declaration, for example `export Entity "PaymentService"`.
- Use `//` comments. Do not use `#` comments in `.sea`.
- Quote concept names: `Entity "Customer"`, `Resource "Money" USD`, `Flow "Money" from "Customer" to "Processor"`.
- Use identifiers for policy names, instance names, aliases, and `in` domains. Identifiers do not contain dots, hyphens, spaces, or slashes.
- Put dotted namespaces in `@namespace "finance.payments"`, not after `in`.
- Declare dimensions and units before resources that use them.
- Declare entities, roles, resources, and patterns before flows, relations, instances, policies, and metrics that reference them.
- Keep units unquoted in `Resource "Money" USD`, but quoted in unit declarations and casts such as `as "USD"`.
- Use decimal numbers only. Do not use scientific notation.
- Use time literals with timezone offsets.
- Compare aggregations explicitly in boolean policy contexts, for example `count(flows) > 0`.
- Treat policy evaluation as three-valued by default; missing data may produce `Unknown`.
- Use mapping/projection targets from the reference: `calm`, `kg`, `sbvr`, `protobuf`, or `proto`.

Minimal valid starting point:

```sea
@namespace "logistics"
@version "1.0.0"
@owner "architecture-team"
@profile "default"

Entity "Warehouse"
Entity "Factory"

Resource "Camera" units

Flow "Camera" from "Warehouse" to "Factory" quantity 100

Policy positive_flows as:
    forall f in flows: (f.quantity > 0)
```

## CLI Playbook

Install or build:

```bash
cargo install --path domainforge-core --features cli --force
domainforge --help
domainforge --version
```

Core commands:

```bash
domainforge validate --format human model.sea
domainforge validate --format json model.sea
domainforge explain --format human model.sea
domainforge parse model.sea --format human
domainforge parse model.sea --ast --format json
domainforge fmt --check model.sea
domainforge fmt model.sea --out model.sea
domainforge graph model.sea
domainforge units model.sea
domainforge normalize "true AND x" --json
```

Projection and import:

```bash
domainforge project --format calm model.sea calm.json
domainforge project --format rdf --created-at 2026-01-01T00:00:00Z model.sea rdf_out/
domainforge project --format bpmn --created-at 2026-01-01T00:00:00Z model.sea bpmn_out/
domainforge project --format cmmn --created-at 2026-01-01T00:00:00Z model.sea cmmn_out/
domainforge project --format lean --created-at 2026-01-01T00:00:00Z model.sea lean_out/
domainforge project --format protobuf --include-services --package com.example model.sea api.proto
domainforge import --format calm calm.json --out restored.sea
domainforge import --format kg graph.ttl
domainforge import --format sbvr vocabulary.xmi
domainforge validate-kg rdf_out/model.ttl
```

Semantic pack commands:

```bash
domainforge pack build --source "models/**/*.sea" --org acme --domain logistics \
  --version 1.0.0 --meaning-version 1.0.0 --out packs/candidate.json

domainforge pack validate --pack packs/approved.json --mode strict models/**/*.sea
domainforge pack inspect --format json packs/approved.json
domainforge pack diff --old packs/v1.json --new packs/v2.json
domainforge pack sign packs/approved.json --key keys/private.pem --out packs/signed.json
domainforge pack verify packs/signed.json --key keys/public.pem
```

Use `--format json` or `--format jsonl` when another tool will consume output. Use `SEA_LOG=debug` for verbose troubleshooting and `SEA_REGISTRY` or `--registry .sea-registry.toml` for module resolution.

## Debugging Validation Failures

Move from syntax to semantics:

1. Run `domainforge validate --format json <file>.sea` and inspect the first parser or semantic error.
2. Run `domainforge explain --format human <file>.sea` when suggestions matter.
3. Run `domainforge parse --ast --format json <file>.sea` to see whether the parser accepted the structure you intended.
4. Check declaration order, quoted names, namespace defaults, import registry, unit dimensions, ambiguous unqualified references, and aggregate comparisons.
5. If CLI behavior is surprising in the DomainForge repository, inspect `domainforge-core/src/cli/mod.rs` and the relevant file under `domainforge-core/src/cli/commands/`.

Common fixes:

- Unknown entity/resource: add the declaration before use, fix spelling, or resolve namespace ambiguity.
- Syntax near a concept name: quote the concept name.
- Unexpected `as`: use `as:` for policy/metric bodies and `as "Unit"` only for casts.
- Unit conversion failure: declare both units in the same dimension and verify base units.
- Import failure: check `.sea-registry.toml`, exported symbols, aliases, and dependency cycles.
- Formatter mismatch: run `domainforge fmt <file> --out <file>` and then re-run `domainforge fmt --check <file>`.

## Changing DomainForge Itself

When the user asks to change SEA syntax or runtime behavior in the DomainForge repository, follow the repository contract:

- Grammar changes start in `domainforge-core/grammar/sea.pest`.
- Parser AST changes follow in `domainforge-core/src/parser/ast.rs`.
- Add or update parser tests under `domainforge-core/tests/parser_*.rs`.
- If primitives change, update Rust core plus Python, TypeScript, and WASM bindings as required by `AGENTS.md`.
- Use `IndexMap`, not `HashMap`, for policy-relevant deterministic collections.
- Flows reference IDs, not entity/resource objects.
- Run the narrowest relevant tests first, then `just all-tests` when behavior touches bindings or shared contracts.

## Evidence Requirements

Do not claim `.sea` work is done without observed proof. Choose the smallest command set that covers the claim:

- Authoring or editing `.sea`: `domainforge fmt --check <file>.sea` and `domainforge validate --format human <file>.sea`.
- Parser/AST inspection: add `domainforge parse --ast --format json <file>.sea`.
- Projection: run the `domainforge project` command and validate generated output when a validator exists.
- Import/round trip: run import, then validate the restored `.sea`.
- Packs: run the relevant `domainforge pack` command and record exit code behavior for breaking changes, signatures, or strict validation.
- Core implementation changes: run repo tests from `AGENTS.md`, usually `just all-tests` for cross-binding impact.
