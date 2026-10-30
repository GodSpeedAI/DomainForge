# examples — DSL Samples and a Runnable Parser Demo

## Overview
Cargo example target plus hand-written `.sea` samples. `parser_demo.rs` is a
compilable example binary; `basic.sea` and `evolution_semantics.sea` are DSL
inputs demonstrating the annotation/versioning surface. `cli/` holds shell
walkthroughs of the CLI.

## Interface
- `[parser_demo.rs:main]` — parses an inline camera supply-chain DSL with `domainforge_core::parse_to_graph`, then prints entity/resource/flow counts, entities grouped by namespace, resource units, and inbound/outbound flow analysis for one entity using `Graph::flows_to`/`Graph::flows_from`/`Graph::get_entity`/`Graph::get_resource`.
- `basic.sea` — minimal sample: `@namespace "auth"`, `@version "1.0.0"`, one `Entity`, one `Resource`, one `Flow`.
- `evolution_semantics.sea` — versioning sample using `Entity … v1.0.0`, `@replaces`, `@changes`, and a `ConceptChange` with `@from_version`/`@to_version`/`@migration_policy`/`@breaking_change`.
- Child contracts: `./cli/EGM.md` — shell walkthroughs of the `domainforge` CLI.

## Invariants
- `[parser_demo.rs:main]` — the demo treats `parse_to_graph` failure as the error branch of a `match`; it does not unwrap, so a broken sample degrades to an error message instead of a panic.
- `basic.sea` — the sample relies on the `@namespace` file header to qualify unqualified declarations.
- `evolution_semantics.sea` — `@changes` must be a string array and `@breaking_change` a boolean, matching `grammar/sea.pest`'s `annotation_value` alternatives (`string_array` / `boolean`).

## Side Effects
- `[parser_demo.rs:main]` — writes the model summary to stdout via `println!`, and references a missing flow endpoint to stderr via `eprintln!`. No filesystem writes.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
