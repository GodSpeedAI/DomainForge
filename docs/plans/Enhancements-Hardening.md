# SEA DSL Enhancement: Hardening and Feature Implementation Plan

This document tracks the implementation of the comprehensive enhancement plan for the SEA DSL framework.

## Implementation Status

### Phase 1: Grammar and Parser Enhancement ✅ IN PROGRESS

#### Task 1.1: File-Level Metadata Support ✅ GRAMMAR COMPLETE
- [x] Grammar updated with `file_header`, `annotation`, `annotation_name` rules
- [ ] Parser AST updated with `FileMetadata` struct
- [ ] Parser extracts header metadata
- [ ] Graph builder applies file defaults

#### Task 1.2: Unit and Dimension Declaration Syntax ✅ GRAMMAR COMPLETE  
- [x] Grammar updated with `dimension_decl` and `unit_decl` rules
- [x] Grammar supports quantity literals with unit suffixes (`quantity_literal`)
- [ ] Parser AST updated with dimension/unit node types
- [ ] Parser registers custom dimensions/units in registry
- [ ] Units module supports dynamic registration

#### Task 1.3: Policy Modality and Priority Syntax ✅ GRAMMAR COMPLETE
- [x] Grammar extended with `policy_kind`, `policy_modality` rules
- [x] Grammar supports `policy_annotation` for rationale and tags
- [x] Grammar supports inline priority specification
- [ ] Parser AST updated with `PolicyMetadata` struct
- [ ] Policy core wired to populate metadata fields

#### Task 1.4: Aggregation Comprehension Syntax ✅ GRAMMAR COMPLETE
- [x] Grammar supports `aggregation_comprehension` and `aggregation_simple`
- [x] Grammar allows inline filtering: `sum(f in flows where f.resource="Money": f.quantity)`
- [x] Grammar supports unit coercion: `as "USD"`
- [ ] Parser AST updated with comprehension variant
- [ ] Policy expression evaluation implements filtering logic

### Phase 2: Determinism and Type Safety ⏳ PENDING
- [ ] Task 2.1: Explicit Quantifier Expansion Logging
- [ ] Task 2.2: Static Type Inference Pass
- [ ] Task 2.3: Unit Dimension Validation

### Phase 3: Projection Formalization ⏳ PENDING
- [ ] Task 3.1: CALM Normative Mapping Table
- [ ] Task 3.2: Knowledge Graph SHACL Shapes
- [ ] Task 3.3: SBVR Business Rules Export

### Phase 4: Command-Line Interface ⏳ PENDING
- [ ] Task 4.1: CLI Framework Setup
- [ ] Task 4.2: Validate Command
- [ ] Task 4.3: Project Command
- [ ] Task 4.4: Test Command

### Phase 5: Technical Debt Elimination ⏳ PENDING
- [ ] Task 5.1: Parser Refactoring
- [ ] Task 5.2: Projection Module Abstraction
- [ ] Task 5.3: Configuration-Driven Unit Registry
- [ ] Task 5.4: Expression Optimization
- [ ] Task 5.5: Enhanced Test Coverage

### Phase 6: Documentation and CI Integration ⏳ PENDING
- [ ] Task 6.1: Grammar Documentation
- [ ] Task 6.2: CI Validation Gates
- [ ] Task 6.3: API Documentation

## Grammar Changes Summary

### New Rules Added

```pest
// File-level metadata
file_header = { annotation+ }
annotation = { "@" ~ annotation_name ~ string_literal }
annotation_name = { ^"namespace" | ^"version" | ^"owner" }

// Dimension and Unit declarations
dimension_decl = { ^"dimension" ~ string_literal }
unit_decl = { ^"unit" ~ string_literal ~ ^"of" ~ string_literal ~ ^"factor" ~ number ~ ^"base" ~ string_literal }

// Enhanced policy syntax
policy_decl = {
    ^"policy" ~ identifier ~ 
    (^"per" ~ policy_kind ~ policy_modality ~ ^"priority" ~ number)? ~ 
    policy_annotation* ~ 
    (^"v" ~ version)? ~ 
    ^"as" ~ ":" ~ expression
}
policy_kind = { ^"Constraint" | ^"Derivation" | ^"Obligation" }
policy_modality = { ^"Obligation" | ^"Prohibition" | ^"Permission" }
policy_annotation = { "@" ~ policy_annotation_name ~ (string_literal | string_array) }
policy_annotation_name = { ^"rationale" | ^"tags" }
string_array = { "[" ~ string_literal ~ ("," ~ string_literal)* ~ "]" }

// Aggregation comprehension
aggregation_expr = { aggregate_fn ~ "(" ~ (aggregation_comprehension | aggregation_simple) ~ ")" }
aggregation_comprehension = {
    identifier ~ ^"in" ~ collection ~ ^"where" ~ expression ~ ":" ~ expression ~ (^"as" ~ string_literal)?
}
aggregation_simple = { collection ~ ("." ~ identifier)? ~ (^"where" ~ expression)? }

// Quantity literals
quantity_literal = { number ~ string_literal }
literal = { multiline_string | string_literal | quantity_literal | number | boolean }
```

### Example Syntax Enabled

```sea
// File-level metadata
@namespace "com.acme.finance"
@version "2.2.0"
@owner "team-payments"

// Dimension and unit declarations
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"

// Entity with namespace
Entity "Vendor" in procurement

// Resource with unit
Resource "Money" unit "USD" in finance

// Flow with quantity literal
Flow "Payment" from "AP" to "Vendor" quantity 1_500 "USD"

// Policy with full metadata
Policy ap_cap per Constraint Obligation priority 5
  @rationale "Limit vendor exposure"
  @tags ["sox", "payments"]
as:
  sum(f in flows where f.resource="Money" and f.to.name="VendorX": f.quantity as "USD") <= 5_000 "USD"
```

## Next Steps

1. **Complete AST Module Updates**
   - Add `FileMetadata` struct to AST
   - Add `AstNode::Dimension` and `AstNode::UnitDeclaration` variants
   - Add `PolicyMetadata` struct and update `AstNode::Policy`
   - Add aggregation comprehension support to expression parsing

2. **Update Graph Builder**
   - Extract file metadata from AST
   - Apply namespace/version defaults to entities/resources
   - Register custom dimensions/units during graph construction

3. **Extend Policy Core**
   - Wire policy metadata from parser
   - Add support for priority-based evaluation
   - Add violation severity mapping from modality

4. **Implement Type System**
   - Create `sea-core/src/type_system/mod.rs`
   - Implement type inference pass
   - Add dimension validation for expressions

5. **Build CLI Tool**
   - Create `sea-cli` workspace member
   - Implement `validate`, `project`, `test`, `check` commands
   - Add colorized error reporting

## References

- Full plan: `docs/plans/Enhancements-Hardening.md` (generated by plan_task tool)
- Original analysis: User-provided current state assessment
- Grammar file: `sea-core/grammar/sea.pest`
- Parser module: `sea-core/src/parser/`
