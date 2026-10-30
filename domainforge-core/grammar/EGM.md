# grammar — Pest Grammar for the SEA DSL

## Overview
`sea.pest` is the single Pest grammar compiled into `parser::SeaParser` via
`#[grammar = "grammar/sea.pest"]`. It defines the token layer, file header and
import/export modifiers, and the full declaration and expression language
(175 rules).

## Interface
- `[sea.pest:WHITESPACE]` — `_{ " " | "\t" | "\r" | "\n" }`; implicit whitespace.
- `[sea.pest:COMMENT]` — `_{ "//" ~ (!"\n" ~ ANY)* }`; implicit line comment.
- `[sea.pest:in_keyword]` — `@{ ^"in" ~ WHITESPACE+ }`; the keyword rule requiring trailing whitespace.
- `[sea.pest:program]` — `{ SOI ~ file_header? ~ declaration* ~ EOI }`; the grammar entry point.
- `[sea.pest:file_header]` — `{ annotation* ~ import_decl* }`.
- `[sea.pest:annotation]` — `{ "@" ~ annotation_name ~ string_literal }`; `[sea.pest:annotation_name]` restricts to `namespace`, `version`, `owner`, `profile`.
- `[sea.pest:import_decl]` — `{ ^"import" ~ import_specifier ~ ^"from" ~ string_literal }`.
- `[sea.pest:import_specifier]` — `{ import_named | import_wildcard }`.
- `[sea.pest:import_named]` — `{ "{" ~ import_item ~ ("," ~ import_item)* ~ ","? ~ "}" }`.
- `[sea.pest:import_item]` — `{ identifier ~ (^"as" ~ identifier)? }`.
- `[sea.pest:import_wildcard]` — `{ "*" ~ ^"as" ~ identifier }`.
- `[sea.pest:export_decl]` — `{ ^"export" ~ declaration_inner }`.
- `[sea.pest:declaration]` — `{ export_decl | declaration_inner }`.
- `[sea.pest:declaration_inner]` — the ordered alternation over all 28 declaration forms: `dimension_decl`, `unit_decl`, `record_decl`, `enum_decl`, `operation_decl`, `entity_decl`, `resource_decl`, `flow_decl`, `pattern_decl`, `role_binding_decl`, `role_decl`, `relation_decl`, `instance_decl`, `policy_decl`, `concept_change_decl`, `metric_decl`, `mapping_decl`, `projection_decl`, `cell_decl`, `system_dependency_decl`, `runtime_decl`, `tool_decl`, `dependency_set_decl`, `service_decl`, `mount_decl`, `endpoint_decl`, `network_flow_decl`, `credential_decl`.
- `[sea.pest:expression]` — `{ or_expr }`, descending through `or_expr`, `and_expr`, `not_expr`, `comparison_expr`, `additive_expr`, `multiplicative_expr`, `unary_expr`, `cast_expr`, `primary_expr`.
- `[sea.pest:annotation_value]` — `{ object_literal | string_array | string_literal | boolean | number }`.
- `[sea.pest:literal]` — `{ time_literal | multiline_string | string_literal | interval_literal | quantity_literal | array_literal | number | boolean }`.
- `[sea.pest:identifier]` — `@{ (LETTER | "_") ~ (LETTER | NUMBER | "_")* }`.
- `[sea.pest:generic_annotation]` — `{ "@" ~ identifier ~ annotation_value }`, shared by the cell-environment declaration family.

## Invariants
- `[sea.pest:WHITESPACE]` and `[sea.pest:COMMENT]` — both are `_` (silent) rules, so they never appear in the produced parse tree.
- `[sea.pest:program]` — the grammar is anchored at both ends (`SOI … EOI`), so trailing unparsed input is a parse error rather than being ignored.
- `[sea.pest:declaration_inner]` — declaration alternatives are ordered; earlier alternatives win on a shared prefix, so keyword rules that are prefixes of others must appear after the longer form.
- `[sea.pest:in_keyword]` — the `in` keyword rule consumes its own trailing whitespace (`WHITESPACE+`), distinguishing it from an identifier that merely starts with "in".

## Side Effects
- None observed in source. This is a declarative grammar file with no runtime behavior of its own.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
