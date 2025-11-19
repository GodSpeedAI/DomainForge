# SEA DSL Enhancement: Hardening and Feature Implementation
## Overview
This plan implements all recommendations from the current state analysis to harden the SEA DSL framework. The plan addresses determinism, policy metadata, richer operators, aggregation improvements, unit/dimension handling, projection scaffolds, and comprehensive error handling.
### Key Objectives
1. **Enhance DSL surface syntax** with file-level metadata, unit/dimension declarations, and policy modality annotations
2. **Strengthen determinism** through explicit quantifier expansion and stable evaluation order
3. **Improve aggregation functions** with inline filtering and comprehension syntax
4. **Formalize projection mappings** with normative tables and SHACL shapes
5. **Build production-grade CLI** with validation, projection, and testing commands
6. **Eliminate technical debt** identified in parser, evaluation, and projection modules
---
## Task Breakdown
### Phase 1: Grammar and Parser Enhancement
#### Task 1.1: File-Level Metadata Support
**Objective**: Add header annotations for namespace, version, and ownership to reduce repetition and improve scoping.
**Status**: ✅ Implemented (parser AST exposes `FileMetadata`, `parse_source` extracts annotations, and `ast_to_graph` applies defaults). 【F:sea-core/src/parser/ast.rs†L18-L213】【F:sea-core/src/parser/ast.rs†L958-L1112】

**Changes**:
- **Grammar** (`sea-core/grammar/sea.pest`): ✅
  - Add `file_header` rule for `@namespace`, `@version`, `@owner` annotations
  - Update `program` rule to parse optional header before declarations
  - Add `annotation` rule pattern: `"@" ~ annotation_name ~ string_literal`
- **Parser AST** (`sea-core/src/parser/ast.rs`): ✅
  - Add `FileMetadata` struct with `namespace`, `version`, `owner` fields
  - Add `Ast::metadata` field
  - Update `parse_source()` to extract header metadata
  - Implement resolution rule: declaration-level overrides file defaults
- **Graph Builder**: ✅
  - Apply file metadata defaults when creating entities/resources/policies
  - Preserve explicit declaration-level metadata
**Example Syntax**:
```
@namespace "com.acme.finance"
@version "2.2.0"
@owner "team-payments"
Entity "Vendor" in procurement
Resource "Money" unit "USD" in finance
```
#### Task 1.2: Unit and Dimension Declaration Syntax
**Objective**: Allow authors to declare custom dimensions and units in DSL files.
**Status**: ✅ Implemented (AST nodes cover `Dimension`/`UnitDeclaration`, registry accepts dynamic entries, and graph registration occurs during parsing). 【F:sea-core/src/parser/ast.rs†L44-L116】【F:sea-core/src/units/mod.rs†L288-L354】【F:sea-core/src/parser/ast.rs†L976-L1014】

**Changes**:
- **Grammar** (`sea-core/grammar/sea.pest`): ✅
  - Add `dimension_decl` rule: `Dimension string_literal`
  - Add `unit_decl` rule: `Unit string_literal of string_literal factor number base string_literal`
  - Update `declaration` to include `dimension_decl | unit_decl`
  - Add optional unit suffix to `number`: `number ~ identifier?` for quantity literals
- **Parser AST** (`sea-core/src/parser/ast.rs`): ✅
  - Add `AstNode::Dimension { name }`
  - Add `AstNode::UnitDeclaration { symbol, dimension, factor, base_unit }`
  - Add `QuantityLiteral { value, unit }` variant
- **Parser to Graph** (`sea-core/src/parser/mod.rs`): ✅
  - Register custom dimensions/units in global registry during graph construction
  - Validate unit literals against registry
- **Units Module** (`sea-core/src/units/mod.rs`): ✅
  - Add `UnitRegistry::register_dimension(name)`
  - Add validation for duplicate dimension/unit names
**Example Syntax**:
```
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"
Resource "Money" unit "USD" in finance
Flow "Payment" from "AP" to "Vendor" quantity 1_500 "USD"
```
#### Task 1.3: Policy Modality and Priority Syntax
**Objective**: Expose policy metadata (modality, kind, priority, rationale, tags) in DSL syntax.
**Status**: ✅ Implemented (policy parser fills metadata, core policies retain rationale/tags, and graphs store policy declarations). 【F:sea-core/src/parser/ast.rs†L314-L416】【F:sea-core/src/policy/core.rs†L20-L161】【F:sea-core/src/graph/mod.rs†L7-L335】

**Changes**:
- **Grammar** (`sea-core/grammar/sea.pest`): ✅
  - Extend `policy_decl` rule: `Policy identifier per policy_kind policy_modality priority number annotations? as: expression`
  - Add `policy_kind` rule: `Constraint | Derivation | Obligation`
  - Add `policy_modality` rule: `Obligation | Prohibition | Permission`
  - Add `policy_annotation` rule: `@rationale string_literal | @tags string_list`
- **Parser AST** (`sea-core/src/parser/ast.rs`): ✅
  - Add `PolicyMetadata` struct with `kind`, `modality`, `priority`, `rationale`, `tags`
  - Update `AstNode::Policy` to include metadata fields
- **Policy Core** (`sea-core/src/policy/core.rs`): ✅
  - Already has fields; wire parser to populate them
  - Add `Policy::with_metadata()` builder
**Example Syntax**:
```
Policy ap_cap per Constraint Obligation priority 5
  @rationale "Limit vendor exposure"
  @tags ["sox", "payments"]
as:
  sum(f in flows where f.resource="Money": f.quantity) <= 5_000
```
#### Task 1.4: Aggregation Comprehension Syntax
**Objective**: Add inline filtering to aggregation functions with comprehension notation.
**Status**: ✅ Implemented (parser, expression evaluation, and tests cover comprehension syntax with unit coercion). 【F:sea-core/src/parser/ast.rs†L622-L888】【F:sea-core/src/policy/expression.rs†L1-L200】【F:sea-core/tests/aggregation_integration_tests.rs†L55-L197】

**Changes**:
- **Grammar** (`sea-core/grammar/sea.pest`): ✅
  - Extend `aggregation_expr` rule: `aggregate_fn "(" identifier "in" collection "where" expression ":" expression ")"`
  - Support legacy syntax: `aggregate_fn "(" collection ("." identifier)? ")"`
- **Parser AST** (`sea-core/src/parser/ast.rs`): ✅
  - Add `AggregationExpression::Comprehension { function, variable, collection, predicate, projection }`
  - Maintain backward compatibility with existing `Aggregation` variant
- **Policy Expression** (`sea-core/src/policy/expression.rs`): ✅
  - Add `Expression::AggregationComprehension` variant
  - Implement evaluation: filter collection by predicate, project field, apply function
**Example Syntax**:
```
sum(f in flows where f.resource = "Money" and f.to.name = "VendorX": f.quantity as "USD")
count(e in entities where e.domain = "logistics")
avg(r in resources where r.dimension = "Currency": r.quantity)
```
---
### Phase 2: Determinism and Type Safety
#### Task 2.1: Explicit Quantifier Expansion Logging
**Objective**: Make quantifier expansion transparent and auditable for debugging.
**Changes**:
- **Quantifier Module** (`sea-core/src/policy/quantifier.rs`):
  - Add `QuantifierExpansionTrace` struct to capture expansion steps
  - Add `expand_with_trace()` method that returns `(Expression, QuantifierExpansionTrace)`
  - Log collection size, substituted variables, and generated sub-expressions
- **Validation Error** (`sea-core/src/validation_error.rs`):