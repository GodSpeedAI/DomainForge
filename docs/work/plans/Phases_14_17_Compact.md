# 🧭 Phase 14-17: REVISED - Determinism, Errors, Unicode, Export

## ✅ VALIDATION REPORT

**Status:** Issues Found & Auto-Fixed
**Date:** 2025-11-08
**Validator:** Context7 + Ref + Exa + VibeCheck

### Key Findings

1. **Phase 14**: ✅ **ALREADY IMPLEMENTED** - IndexMap already in use, just needs documentation
2. **Phase 15**: ✅ **MOSTLY COMPLETE** - Error types already exist, minimal work needed
3. **Phase 16**: ⚠️ **NEEDS VALIDATION** - Pest may already handle Unicode; verify before implementing
4. **Phase 17**: ❌ **DEFER TO POST-MVP** - High complexity, low immediate ROI, technical debt risk

---

## Phase 14: Policy Determinism Documentation (REVISED)

**Priority:** P2 (downgraded from P1) | **Duration:** 2 days (reduced from 7) | **Complexity:** Very Low

### Status: ✅ Core Implementation Already Complete

**Validation Results:**

- ✅ `graph/mod.rs` already uses `IndexMap` for entities, resources, flows, instances
- ✅ `validation_error.rs` already has `DeterminismError` variant
- ✅ Iteration order is already deterministic
- ⚠️ Missing: Documentation of evaluation order guarantees

### Objectives (REVISED)

Document existing deterministic behavior and add policy priority field for explicit ordering.

### TDD Cycles (SIMPLIFIED)

**Cycle 14A — Verify Existing Determinism**

```rust
#[test]
fn test_iteration_order_already_deterministic() {
    // This test should PASS with current implementation
    let graph = build_test_graph();

    let flows1: Vec<_> = graph.all_flows().map(|(id, _)| id).collect();
    let flows2: Vec<_> = graph.all_flows().map(|(id, _)| id).collect();

    // IndexMap guarantees same order
    assert_eq!(flows1, flows2);
}
```

**Implementation (REDUCED SCOPE):**

- ✅ IndexMap already used (no code changes needed)
- 📝 Document deterministic iteration guarantees in docs/specs/sds.md
- 📝 Add policy evaluation order section to docs/specs/api_specification.md
- ➕ Add optional `Policy::priority: Option<u32>` field for explicit ordering (NEW)

**Cycle 14B — Add Policy Priority Field**

```rust
// In sea-core/src/policy/mod.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Policy {
    // ... existing fields ...
    pub priority: Option<u32>,  // NEW: Explicit evaluation ordering
}

impl Policy {
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }
}

// In graph validation:
// Policies with explicit priority sort first, then insertion order
```

**Cycle 14C — Documentation Only**

- 📝 Add "Deterministic Evaluation Guarantees" section to SDS
- 📝 Document IndexMap usage rationale in ADR
- 📝 Update API docs with policy ordering behavior

**Success Criteria:**

- [x] All collection iteration uses IndexMap (ALREADY DONE)
- [ ] Policy evaluation order documented
- [ ] Policy::priority field implemented
- [ ] Examples in documentation

---

## Phase 15: Enhanced Error Messages (REVISED)

**Priority:** P2 | **Duration:** 3 days (reduced from 7) | **Complexity:** Very Low

### Status: ✅ Error Types Already Implemented

**Validation Results:**

- ✅ `UnitError` with suggestions - ALREADY EXISTS
- ✅ `ScopeError` with available_in - ALREADY EXISTS
- ✅ `DeterminismError` with hints - ALREADY EXISTS
- ✅ `TypeError` with suggestions - ALREADY EXISTS
- ✅ Source ranges (line/column) - ALREADY EXISTS
- ⚠️ Missing: Helper methods for common error patterns

### Objectives (REVISED)

Add convenience methods and improve error message formatting; core types already complete.

### Implementation (REDUCED SCOPE)

**Cycle 15A — Add Error Builder Helpers**

```rust
// Already exists, just add convenience methods
impl ValidationError {
    // NEW: Convenience constructors
    pub fn undefined_entity(name: &str, location: &str) -> Self {
        Self::UndefinedReference {
            reference_type: "Entity".to_string(),
            name: name.to_string(),
            location: location.to_string(),
            suggestion: Some(format!("Did you mean to define 'Entity \"{}\"'?", name)),
        }
    }

    pub fn unit_mismatch(expected: Dimension, found: Dimension, location: &str) -> Self {
        Self::UnitError {
            expected,
            found,
            location: location.to_string(),
            suggestion: Some(format!(
                "Expected {:?} but found {:?}. Consider using unit conversion.",
                expected, found
            )),
        }
    }
}
```

**Cycle 15B — Improve Error Display**

```rust
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnitError { expected, found, location, suggestion } => {
                write!(f, "Unit mismatch at {}: expected {:?}, found {:?}",
                       location, expected, found)?;
                if let Some(hint) = suggestion {
                    write!(f, "\n  Hint: {}", hint)?;
                }
                Ok(())
            }
            // ... other variants with improved formatting
        }
    }
}
```

**Success Criteria:**

- [x] UnitError, ScopeError, DeterminismError exist (ALREADY DONE)
- [ ] Convenience constructor methods added
- [ ] Improved Display formatting with hints
- [ ] Examples in error handling documentation

---

## Phase 16: Unicode & Escape Sequences (REVISED)

**Priority:** P2 (downgraded from P1) | **Duration:** 5 days (reduced from 10) | **Complexity:** Low (downgraded from Medium)

### Status: ⚠️ NEEDS VALIDATION - May Already Work

**Validation Results:**

- ✅ Rust strings are UTF-8 by default - Unicode in string_literal should work
- ✅ Pest parser inherently handles UTF-8 correctly
- ⚠️ Unknown: Current escape sequence support in parser
- ❌ Missing: Multi-line strings (""" syntax)

### Objectives (REVISED - VALIDATE FIRST)

**BEFORE implementing, verify what already works:**

1. Test if Unicode identifiers already parse correctly
2. Test if basic escape sequences (\n, \t, \\, \") work
3. Only implement what's actually broken or missing

### TDD Cycles (VALIDATION-FIRST APPROACH)

**Cycle 16A — Validate Current Unicode Support**

```rust
#[test]
fn test_unicode_in_string_literals_works_now() {
    // TEST FIRST - This might already pass!
    let source = r#"
        Entity "Müller GmbH" in germany
        Entity "北京公司" in china
        Entity "Société Générale" in france
    "#;

    let result = parse_to_graph(source);

    // If this passes, Unicode support is DONE
    match result {
        Ok(graph) => {
            assert_eq!(graph.entity_count(), 3);
            println!("✅ Unicode already works! No implementation needed.");
        }
        Err(e) => {
            println!("❌ Unicode broken: {:?}", e);
            println!("   Implement escape_sequence in grammar");
        }
    }
}
```

**Cycle 16B — Validate Escape Sequences (IF NEEDED)**

```rust
#[test]
fn test_escape_sequences_validation() {
    let test_cases = vec![
        (r#"Entity "Quote: \"test\"" "#, "Quote: \"test\""),
        (r#"Entity "Line\nBreak" "#, "Line\nBreak"),
        (r#"Entity "Tab\there" "#, "Tab\there"),
    ];

    for (source, expected_name) in test_cases {
        match parse_to_graph(source) {
            Ok(graph) => println!("✅ Escape '{}' works", expected_name),
            Err(_) => println!("❌ Need to implement: {}", expected_name),
        }
    }
}
```

**Grammar Update (ONLY IF VALIDATION FAILS):**

```pest
// Add ONLY if tests show it's needed
string_literal = @{
    "\"" ~ string_content* ~ "\""
}

string_content = {
    escape_sequence | string_char
}

escape_sequence = @{
    "\\\\" |     // Backslash
    "\\\"" |     // Quote
    "\\n" |      // Newline
    "\\r" |      // Carriage return
    "\\t" |      // Tab
    ("\\u{" ~ ASCII_HEX_DIGIT{1,6} ~ "}")  // Unicode escape
}

string_char = { !("\"" | "\\") ~ ANY }
```

**Cycle 16C — Multi-Line Strings (LOW PRIORITY)**

```rust
// ONLY implement if users actually need this
#[test]
fn test_multiline_strings() {
    let source = r####"
        Policy description as: """
        Multi-line policy
        description here
        """
    "####;

    // Add to grammar if needed:
    // multiline_string = @{ "\"\"\"" ~ (!"\"\"\"" ~ ANY)* ~ "\"\"\"" }
}
```

**Success Criteria (REVISED):**

- [ ] **VALIDATION PHASE**: Run tests to see what works now
- [ ] Document current Unicode/escape support status
- [ ] Implement ONLY features that fail validation
- [ ] If everything works, mark phase as COMPLETE with no code changes

---

## Phase 17: SBVR & RDF Export - DEFERRED TO POST-MVP

**Priority:** P3 (downgraded from P2) | **Duration:** 21 days | **Complexity:** High
**Status:** ⛔ **RECOMMEND DEFERRAL** - High technical debt risk

### Validation Assessment: DEFER THIS PHASE

**Critical Issues Identified:**

1. **CALM Already Implemented** - Phase 10 provides architecture export (completed Nov 2025)
2. **Low Immediate ROI** - SBVR/RDF export benefits unclear vs cost
3. **High Complexity** - 21 days for export that may not be used
4. **Technical Debt Risk** - Round-trip conversion extremely difficult to maintain
5. **No User Demand** - PRD doesn't list SBVR/RDF as critical requirements
6. **Maintenance Burden** - Three export formats (CALM, SBVR, RDF) to keep in sync

### Why This Should Be Post-MVP

**From PRD Analysis:**

- PRD-014 specifies CALM interoperability (✅ DONE in Phase 10)
- No PRD requirement for SBVR export
- No PRD requirement for RDF/SHACL export
- Success metrics focus on model validation speed, not export formats

**From Vibe Check:**

- Question: "Does this plan directly address what the user requested?"
- Answer: ❌ NO - User hasn't requested SBVR/RDF export
- This solves a problem that may not exist yet

**From Best Practices (Exa/Ref research):**

- SHACL validation requires deep ontology design expertise
- SBVR XMI mappings are notoriously complex to maintain
- Round-trip conversion rarely preserves full semantics
- Multiple export formats increase testing burden exponentially

### Recommended Alternative: Defer to Phase 18+ (Post-MVP)

**Instead, focus on:**

1. User feedback on CALM export (Phase 10) - is ONE export enough?
2. Gather actual requirements for SBVR/RDF from real users
3. Validate use cases before building complex export machinery
4. Consider simpler JSON-LD export if semantic web integration needed

### IF Implemented Later (Post-MVP)

**Simplified Approach:**

**Phase 17 (Original Design) - NOT RECOMMENDED NOW**

```rust
#[test]
fn test_export_to_sbvr() {
    let graph = build_test_graph();
    let sbvr_xml = graph.export_sbvr().unwrap();

    // Validate against SBVR XMI schema
    assert!(sbvr_xml.contains("<sbvr:FactType"));
    assert!(sbvr_xml.contains("<sbvr:Obligation"));
}
```

**Implementation:**

```rust
pub mod sbvr {
    pub struct SbvrModel {
        vocabulary: Vec<Term>,
        facts: Vec<FactType>,
        rules: Vec<BusinessRule>,
    }

    impl SbvrModel {
        pub fn to_xmi(&self) -> Result<String, SbvrError> {
            // Generate SBVR 1.5 XMI
        }
    }
}

impl Graph {
    pub fn export_sbvr(&self) -> Result<String, SbvrError> {
        let model = SbvrModel::from_graph(self)?;
        model.to_xmi()
    }
}
```

**Mapping:**

- Entity → SBVR Term (General Concept)
- Resource → SBVR Term (Individual Concept)
- Flow → SBVR Fact Type
- Policy → SBVR Business Rule (Obligation/Prohibition)

**Cycle 17B — Knowledge Graph Export**

```rust
#[test]
fn test_export_to_rdf() {
    let graph = build_test_graph();
    let rdf_turtle = graph.export_rdf("turtle").unwrap();

    assert!(rdf_turtle.contains("sea:Entity"));
    assert!(rdf_turtle.contains("sea:hasResource"));
}
```

**Implementation:**

```rust
pub mod kg {
    pub struct KnowledgeGraph {
        triples: Vec<Triple>,
        shapes: Vec<ShaclShape>,
    }

    impl KnowledgeGraph {
        pub fn to_turtle(&self) -> String { /* ... */ }
        pub fn to_rdf_xml(&self) -> String { /* ... */ }
    }
}

impl Graph {
    pub fn export_rdf(&self, format: &str) -> Result<String, KgError> {
        let kg = KnowledgeGraph::from_graph(self)?;
        match format {
            "turtle" => Ok(kg.to_turtle()),
            "rdf-xml" => Ok(kg.to_rdf_xml()),
            _ => Err(KgError::UnsupportedFormat(format.to_string())),
        }
    }
}
```

**Ontology:**

```turtle
@prefix sea: <http://domainforge.ai/sea#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

sea:Entity a owl:Class ;
    rdfs:label "Entity" ;
    rdfs:comment "Business actor, location, or organizational unit" .

sea:Resource a owl:Class ;
    rdfs:label "Resource" ;
    rdfs:comment "Quantifiable subject of value" .

sea:Flow a owl:Class ;
    rdfs:label "Flow" ;
    rdfs:comment "Transfer of resource between entities" .

sea:hasResource a owl:ObjectProperty ;
    rdfs:domain sea:Flow ;
    rdfs:range sea:Resource .
```

**SHACL Shapes:**

```turtle
sea:FlowShape a sh:NodeShape ;
    sh:targetClass sea:Flow ;
    sh:property [
        sh:path sea:quantity ;
        sh:datatype xsd:decimal ;
        sh:minExclusive 0 ;
    ] ;
    sh:property [
        sh:path sea:hasUnit ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .
```

**Cycle 17C — Round-Trip Testing**

```rust
#[test]
fn test_sbvr_round_trip() {
    let original = build_test_graph();
    let sbvr_xml = original.export_sbvr().unwrap();
    let imported = Graph::import_sbvr(&sbvr_xml).unwrap();

    assert_eq!(original.entity_count(), imported.entity_count());
    assert_eq!(original.policy_count(), imported.policy_count());
}

#[test]
fn test_rdf_round_trip() {
    let original = build_test_graph();
    let rdf = original.export_rdf("turtle").unwrap();
    let imported = Graph::import_rdf(&rdf).unwrap();

    assert_eq!(original.entity_count(), imported.entity_count());
}
```

**Success Criteria:**

- [ ] SBVR XMI export validates against schema
- [ ] RDF export in Turtle and RDF/XML formats
- [ ] SHACL shapes generated for validation
- [ ] Round-trip conversion preserves semantics
- [ ] Documentation with normative mappings

---

## Summary: REVISED TIMELINE & PRIORITIES

### Original vs. Revised Estimates

| Phase | Original | Revised | Change | Reason |
|-------|----------|---------|--------|--------|
| 14    | 7 days (P1) | 2 days (P2) | ⬇️ -71% | IndexMap already implemented |
| 15    | 7 days (P2) | 3 days (P2) | ⬇️ -57% | Error types already exist |
| 16    | 10 days (P1) | 5 days (P2) | ⬇️ -50% | Validate-first approach |
| 17    | 21 days (P2) | **DEFERRED** | ⛔ 0 days | Post-MVP, technical debt risk |
| **TOTAL** | **45 days** | **10 days** | **⬇️ 78% reduction** | Focus on actual gaps |

### What Changed & Why

#### ✅ Wins From Validation

1. **Phase 14**: Core determinism already solved - just needs docs
2. **Phase 15**: Rich error types already implemented - just add helpers
3. **Phase 16**: May already work - test before building
4. **Phase 17**: Deferred to avoid premature complexity

#### 🎯 Recommended Execution Order

**Week 1: Quick Wins (5 days)**

- Days 1-2: Phase 14 - Document determinism, add Policy::priority
- Days 3-5: Phase 15 - Error builder helpers, improved Display

**Week 2: Validation & Completion (5 days)**

- Days 1-2: Phase 16 - Run validation tests
- Days 3-5: Phase 16 - Implement ONLY what's broken (if anything)

**Post-MVP: Phase 17 (IF needed)**

- Gather user feedback on export needs
- Validate SBVR/RDF use cases
- Design incremental export strategy

### Technical Debt Avoided

1. ❌ **Avoided**: Reimplementing IndexMap conversion (already done)
2. ❌ **Avoided**: Duplicating error types (already comprehensive)
3. ❌ **Avoided**: Complex SBVR/RDF export before proving necessity
4. ✅ **Result**: 78% time reduction, zero technical debt increase

### Alignment With Existing Architecture

- ✅ Follows TDD cycles as required
- ✅ Maintains cross-language binding updates
- ✅ Preserves documentation standards
- ✅ Respects MVP scope discipline
- ✅ Uses validation-first approach to avoid waste

### Next Steps

1. **Review** this revised plan with stakeholders
2. **Execute** Phases 14-16 in 10 days (vs 45 originally)
3. **Gather feedback** on CALM export usage (Phase 10)
4. **Decide** on Phase 17 based on real user needs, not speculation

**Total Estimated Duration (Revised):** 2 weeks (vs. 8-10 weeks originally)

---

## Appendix: Validation Methodology

**Tools Used:**

- **context7**: IndexMap documentation and best practices
- **ref**: Rust error handling patterns (thiserror/anyhow)
- **exa**: Unicode handling, policy evaluation order, SHACL validation patterns
- **vibecheck**: Questioned assumptions, identified premature optimization

**Key Insight:** "Validate what exists before building what might be needed."
