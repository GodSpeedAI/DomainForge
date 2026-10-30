# parser — Pest Grammar Front End, AST, and Printer

## Overview
`SeaParser` is the Pest-derived parser for `grammar/sea.pest`. This module turns
source text into an `Ast` (`parse`) or directly into a `Graph`
(`parse_to_graph`, `parse_to_graph_with_options`), and pretty-prints ASTs back to
source. When a file declares entity bodies, graph construction is delegated to
`crate::application::resolve_application_graph`.

## Interface
- `[mod.rs:SeaParser]` — `#[derive(Parser)] #[grammar = "grammar/sea.pest"]`; the generated Pest parser.
- `[mod.rs:ParseOptions]` — `default_namespace`, `namespace_registry`, `entry_path`, `active_profile`, `tolerate_profile_warnings`.
- `[mod.rs:parse]` — `parse(source: &str) -> ParseResult<Ast>`.
- `[mod.rs:parse_to_graph]` — `parse_to_graph(source) -> ParseResult<Graph>` using default options.
- `[mod.rs:parse_to_graph_with_options]` — honors `ParseOptions`; uses `ModuleResolver` when both a registry and entry path are supplied.
- `[ast.rs:parse_source]`, `[ast.rs:parse_expression_from_str]`, `[ast.rs:ast_to_graph]`, `[ast.rs:ast_to_graph_with_options]` — AST parsing and lowering.
- `[ast.rs:Ast]`, `[ast.rs:Spanned<T>]`, `[ast.rs:AstNode]`, `[ast.rs:FileMetadata]` — AST root and node model.
- `[ast.rs:ImportDecl]`, `[ast.rs:ImportSpecifier]`, `[ast.rs:ImportItem]`, `[ast.rs:PolicyMetadata]`, `[ast.rs:PolicyKind]`, `[ast.rs:PolicyModality]`, `[ast.rs:MetricMetadata]`, `[ast.rs:TargetFormat]`, `[ast.rs:MappingRule]`, `[ast.rs:ProjectionOverride]` — declaration metadata types.
- `[ast.rs:EntityBody]`, `[ast.rs:FieldDecl]`, `[ast.rs:FieldType]`, `[ast.rs:FieldTypeRef]`, `[ast.rs:FieldConstraintDecl]`, `[ast.rs:RecordDecl]`, `[ast.rs:EnumDecl]`, `[ast.rs:EnumMember]`, `[ast.rs:OperationDecl]`, `[ast.rs:OperationClause]`, `[ast.rs:PolicyBinding]` — typed application declarations.
- `[ast_schema.rs:Ast]`, `[ast_schema.rs:SpannedAstNode]` — serde/JSON-schema mirror of the AST (`cfg_attr(feature = "json-schema", derive(JsonSchema))`); exports the same declaration types as `ast.rs`.
- `[ast_schema.rs:ImportDecl]`, `[ast_schema.rs:ImportSpecifier]`, `[ast_schema.rs:ImportItem]`, `[ast_schema.rs:FileMetadata]`, `[ast_schema.rs:PolicyKind]`, `[ast_schema.rs:PolicyModality]`, `[ast_schema.rs:PolicyMetadata]`, `[ast_schema.rs:Severity]`, `[ast_schema.rs:MetricMetadata]`, `[ast_schema.rs:TargetFormat]`, `[ast_schema.rs:MappingRule]`, `[ast_schema.rs:ProjectionOverride]`, `[ast_schema.rs:WindowSpec]`, `[ast_schema.rs:BinaryOp]`, `[ast_schema.rs:UnaryOp]`, `[ast_schema.rs:Quantifier]`, `[ast_schema.rs:AggregateFunction]`, `[ast_schema.rs:Expression]`, `[ast_schema.rs:EntityBody]`, `[ast_schema.rs:FieldDecl]`, `[ast_schema.rs:FieldType]`, `[ast_schema.rs:FieldTypeRef]`, `[ast_schema.rs:FieldConstraintDecl]`, `[ast_schema.rs:RecordDecl]`, `[ast_schema.rs:EnumDecl]`, `[ast_schema.rs:EnumMember]`, `[ast_schema.rs:OperationDecl]`, `[ast_schema.rs:OperationClause]`, `[ast_schema.rs:PolicyBinding]` — JSON-schema AST surface.
- `[error.rs:ParseResult]`, `[error.rs:ParseError]` — error type with constructors `from_pest`, `syntax_error`, `undefined_entity`, `undefined_entity_no_loc`, `undefined_resource`, `undefined_resource_no_loc`, `undefined_variable`, `duplicate_declaration`, `duplicate_declaration_no_loc`, `type_error`, `namespace_not_found`, `module_not_found`, `symbol_not_exported`, `circular_dependency`.
- `[printer.rs:PrettyPrinter]` (`+new/with_trailing_commas/print`) — AST-to-source printing.
- `[profiles.rs:Profile]` (`+new/is_feature_allowed/name/description/allowed_features`), `[profiles.rs:ProfileRegistry]` (`+get/list_names/global`) — dialect/profile gating.
- `[lint.rs:LintError]`, `[lint.rs:Linter]` (`+new/check_identifier`) — identifier linting.
- `[string_utils.rs:unescape_string]` — SEA string-literal unescaping.
- `[ast_convert.rs]` — AST lowering helpers; `[mod.rs]` re-exports `parse_expression_from_str`, `parse_source`, `Ast`, `AstNode`, `ParseError`, `ParseResult`, `lint::*`, `PrettyPrinter`, `Profile`, `ProfileRegistry`, `unescape_string`.

## Invariants
- `[mod.rs:parse_to_graph_with_options]` — a namespace registry supplied without an entry path is an error (`"Namespace registry provided without entry path"`); an entry path without a registry skips module resolution and only logs a warning.
- `[mod.rs:graph_from_parsed_source]` — if any declaration (including inside an `Export`) is an `Entity` with a body, graph construction goes through `application::resolve_application_graph` instead of `ast_to_graph_with_options`; otherwise the AST path is used.
- `[mod.rs:graph_from_parsed_source]` — when the source has no `@namespace`, a synthetic `@namespace` line is prepended using `options.default_namespace` (falling back to `"default"`) before application resolution.
- `[error.rs:ParseError]` — every parser failure is reported as a typed `ParseError`, not a panic; Pest errors are converted via `from_pest`.
- `[profiles.rs:ProfileRegistry::global]` — the registry is a process-wide `OnceLock` singleton, so profile definitions are shared for the process lifetime.
- `[printer.rs:PrettyPrinter::with_trailing_commas]` — trailing-comma emission is a printer option, to be applied consistently across a print run.

## Side Effects
- `[mod.rs:parse_to_graph_with_options]` — when a registry and entry path are both present it constructs a `ModuleResolver`, which walks and reads the filesystem.
- `[ast_schema.rs]` — the `#[ignore]`d `generate_ast_schema` test writes `schemas/ast-v3.schema.json` with `std::fs::write`; test-only, not part of the runtime surface.
- `[mod.rs:parse_to_graph_with_options]` — emits a `log::warn!` when an entry path is given without a registry.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
