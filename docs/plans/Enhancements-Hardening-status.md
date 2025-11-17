# SEA DSL Enhancement: Hardening and Feature Implementation Plan
This document tracks the implementation of the comprehensive enhancement plan for the SEA DSL framework.
## Implementation Status
### Phase 1: Grammar and Parser Enhancement ✅ COMPLETE
#### Task 1.1: File-Level Metadata Support ✅ COMPLETE
- [x] Grammar updated with `file_header`, `annotation`, `annotation_name` rules
- [x] Parser AST updated with `FileMetadata` struct
- [x] Parser extracts header metadata
- [x] Graph builder applies file defaults
#### Task 1.2: Unit and Dimension Declaration Syntax ✅ COMPLETE
- [x] Grammar updated with `dimension_decl` and `unit_decl` rules
- [x] Grammar supports quantity literals with unit suffixes (`quantity_literal`)
- [x] Parser AST updated with dimension/unit node types
- [x] Parser registers custom dimensions/units in registry
- [x] Units module supports dynamic registration
#### Task 1.3: Policy Modality and Priority Syntax ✅ COMPLETE
- [x] Grammar extended with `policy_kind`, `policy_modality` rules
- [x] Grammar supports `policy_annotation` for rationale and tags
- [x] Grammar supports inline priority specification
- [x] Parser AST updated with `PolicyMetadata` struct
- [x] Policy core wired to populate metadata fields
#### Task 1.4: Aggregation Comprehension Syntax ✅ COMPLETE
- [x] Grammar supports `aggregation_comprehension` and `aggregation_simple`
- [x] Grammar allows inline filtering: `sum(f in flows where f.resource="Money": f.quantity)`
- [x] Grammar supports unit coercion: `as "USD"`
- [x] Parser AST updated with comprehension variant
- [x] Policy expression evaluation implements filtering logic
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