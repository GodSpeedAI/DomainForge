# Plan: DomainForge Enhancement Suite Implementation (Complete Specification)

**Document Version**: 1.0
**Created**: November 17, 2025
**Status**: Ready for Implementation
**Priority**: P0 (Critical for Production Readiness)

This plan implements 7 priority enhancements plus polish items to strengthen DomainForge's normative contracts, developer experience, and CI/CD guardrails. All implementation choices finalized per architectural review.

---

### Current Status

Step 1 is complete (SBVR/KG import implemented, validated, and merged). The RDF/Turtle and RDF/XML KG import paths now both work end-to-end; SBVR (XMI) import converts BusinessRule → Policy with modality and priority mapping; SHACL validation runs on KG imports and flags violations like sh:minExclusive (e.g., quantity > 0 checks). Automated tests for SBVR imports, KG imports (Turtle & RDF/XML), and SHACL validation were added and pass with the `shacl` feature enabled.

Step 2 is complete (operator precedence documented; three-valued NULL logic behind feature flag implemented and tested). Documentation for operator precedence and NULL semantics is in `docs/specs/semantics.md`. The three-valued logic implementation is gated behind the `three_valued_logic` feature flag, with comprehensive tests in `tests/null_handling_tests.rs` covering all operator combinations and aggregation behaviors.

Step 3 is not yet started (ICU collation with locale bundle and runtime discovery).

Step 4 is complete – Logical Namespace System with `.sea-registry.toml` and Glob Patterns

**Goal:** Automatically map files to logical namespaces so parser defaults remain
stable, even when files omit `in <namespace>` clauses.

**Status:** ✅ Implemented

### Deliverables

- Workspace-level `.sea-registry.toml` that maps glob patterns to namespaces.
- `NamespaceRegistry` loader in `sea_core` with glob validation, duplicate-match
  detection, and helper APIs for both per-file lookups and registry expansion.
- CLI support for validating directories: the `sea` binary now discovers the
  registry, expands all matching `.sea` files, applies the correct namespace
  defaults, and merges the resulting graphs before running validation.
- Documentation under `docs/reference/sea-registry.md` detailing the registry
  format and CLI workflow, plus sample `.sea` files in `examples/namespaces`.

---

### Implementation Strategy

**Approach**: Sequential implementation following TDD RED-GREEN-REFACTOR pattern established in Phases 0-17. Each step includes comprehensive tests, documentation, and cross-language parity verification (Rust, Python, TypeScript, WASM).

**Dependencies**: Phases 0-17 complete (core primitives, graph storage, policy engine, parser, language bindings, CALM integration).

**Timeline**: 15.5 weeks (assuming 1 developer, 30-40 hours/week); this matches the summed step durations and explicitly includes the cross-language parity verification work (Rust, Python, TypeScript, WASM) built into each phase.

---

### Steps

#### 1. Normative Projection Specifications with Bidirectional SBVR/KG Import

**Duration**: 2 weeks | **Priority**: P0 | **Complexity**: High

Create comprehensive projection mapping specifications in `sea-core/docs/specs/projections/` documenting:

- **CALM**: Entity→Node, Flow→Relationship (already implemented, document formally)
- **Knowledge Graph (RDF/SHACL)**:
  - Entity → `sea:Entity` class
  - Resource → `sea:Resource` with `sea:unit`, `sea:dimension` properties
  - Flow → `sea:Flow` with `sea:quantity`, `sea:from`, `sea:to` relationships
  - SHACL shapes for validation constraints
- **SBVR (Semantics of Business Vocabulary and Business Rules)**:
  - Entity → `BusinessThing` (SBVR vocabulary)
  - Resource → `Quantity` with unit-of-measure
  - Flow → `Movement` (action concept)
  - Policy → `BusinessRule` with **modalities**:
    - **Type-based priority defaults**: Obligation=5, Prohibition=5, Permission=1, Derivation=3, **Alethic=3, Deontic=4, Epistemic=2**
    - Override via `sea:priority` extension attribute
  - Defer synonyms, definitions, structured glossaries to Phase 19+

**Implementation**:

- **File**: `calm/sbvr_import.rs`
  - Use `roxmltree` (already in dev-dependencies) for XMI parsing
  - Implement minimal SBVR 1.6 subset: BusinessThing, Quantity, Movement, BusinessRule
  - Parse modality types (Obligation/Prohibition/Permission/Derivation/Alethic/Deontic/Epistemic)
  - Map `sea:priority` custom attribute to priority field

- **File**: `kg_import.rs`
  - Parse RDF Turtle and RDF/XML formats
  - Validate against SHACL shapes (use `oxigraph` crate)
  - Map RDF triples to SEA primitives with referential integrity checks

- **Tests**: `tests/projection_contracts_tests.rs`
  - Golden round-trip tests: DSL → CALM → DSL (verify no-op)
  - Golden round-trip tests: DSL → KG → SHACL validate → DSL
  - Golden round-trip tests: DSL → SBVR XMI → DSL
  - Test all SBVR modalities with priority mapping
  - Test ambiguous cases (multiple SBVR interpretations)

**Documentation**:

- Normative mapping tables with examples in README.md
- Migration guide for existing CALM/SBVR/KG assets
- Known limitations (SBVR vocabulary extensions deferred)

---

#### 2. Operator Precedence Documentation and Three-Valued NULL Logic

**Duration**: 1.5 weeks | **Priority**: P0 | **Complexity**: Medium

Document operator precedence and implement three-valued logic behind compile-time feature flag.

**Precedence Table** (in `docs/specs/semantics.md`):

1. **Primary**: Literals, identifiers, member access (`.`), parentheses `()`
2. **Unary**: `-` (negation), `not`
3. **Multiplicative**: `*`, `/`
4. **Additive**: `+`, `-`
5. **Comparison**: `<`, `>`, `<=`, `>=`, `=`, `!=`, `contains`, `startswith`, `endswith`
6. **Logical AND**: `and`
7. **Logical OR**: `or`
8. **Implication**: `implies`

**NULL Truth Tables** (document SQL UNKNOWN equivalence):

| Operator | Example | Result |
|----------|---------|--------|
| `NULL AND true` | `NULL and true` | `NULL` |
| `NULL AND false` | `NULL and false` | `false` |
| `NULL OR true` | `NULL or true` | `true` |
| `NULL OR false` | `NULL or false` | `NULL` |
| `NOT NULL` | `not NULL` | `NULL` |
| `NULL IMPLIES true` | `NULL implies true` | `true` |
| `NULL = NULL` | `NULL = NULL` | `NULL` (not `true`!) |

**Implementation**:

- **File**: `policy/three_valued.rs`
  - Gate behind `cfg(feature = "three_valued_logic")` compile-time flag
  - Implement `ThreeValuedEvaluator` with `enum ThreeValuedBool { True, False, Null }`
  - NULL propagation in aggregations: `sum([1, NULL, 3])` → `NULL`
  - Provide `sum_nonnull()`, `avg_nonnull()`, `count_nonnull()` variants that skip NULLs

- **File**: `policy/core.rs`
  - Default mode (feature disabled): panic on NULL access with clear error
  - Feature enabled: delegate to `ThreeValuedEvaluator`
  - **No explicit NULL literal syntax** in Phase 18 (only from optional attributes)
  - Document: NULL-handling semantics apply **only** to missing attribute values
  - Add TODO: consider `NULL` keyword in Phase 19+ if user demand

- **Benchmarks**: `benches/null_overhead.rs`
  - Compare evaluation times: feature disabled vs enabled
  - Document 5-10% performance overhead
  - Test on 10K policy evaluations with varying NULL density

- **Tests**: `tests/null_handling_tests.rs`
  - All operator combinations with NULL operands
  - Aggregation NULL propagation vs `_nonnull()` variants
  - Nested expressions with mixed NULL/non-NULL
  - Edge case: `forall x in flows where x.attr = NULL: ...` (should match nothing)

**Documentation**:

- Precedence examples with parentheses overrides
- NULL semantics FAQ ("Why does `NULL = NULL` return `NULL`?")
- Performance guidance (when to enable three-valued logic)

---

#### 3. ICU Collation with Locale Bundle and Runtime Discovery

**Duration**: 1.5 weeks | **Priority**: P1 | **Complexity**: Medium

Implement full ICU collation support with minimal locale bundle and flexible data discovery.

**Locale Bundle** (~2MB for 5 locales):

- English (`en`)
- German (`de`) - handles umlauts, ß
- French (`fr`) - handles accented characters
- Chinese (`zh`) - CJK ideographs
- Arabic (`ar`) - RTL script

**Implementation**:

- **Build Script**: `build.rs`
  - Add `icu_datagen` build dependency
  - Generate locale-specific data files at build time
  - Embed in binary or write to `target/icu_data/`

- **Dependencies**: `Cargo.toml`

  ```toml
  [dependencies]
  icu = "1.5"
  icu_collator = "1.5"

  [build-dependencies]
  icu_datagen = "1.5"
  ```

- **File**: `policy/collation.rs`
  - Wrap ICU `Collator` with thread-safe caching
  - **UCA root collation fallback** with warning for unsupported locales
  - Log supported locales in error message: "Requested 'pt-BR' not in bundled locales [en, de, fr, zh, ar]. Using root UCA collation."
  - **Environment variable `SEA_ICU_DATA_DIR` discovery**:
    - Absolute path (e.g., `/opt/icu_data`)
    - Relative to workspace (e.g., `.sea-icu/`)
    - Colon-separated search path (e.g., `./icu:/usr/local/share/icu`)
  - Fallback order: `$SEA_ICU_DATA_DIR` → `<workspace>/.sea-icu/` → bundled data
  - Log discovery path at DEBUG level: "ICU data loaded from: /workspace/.sea-icu/"
  - **No hot-reload**: changing env var mid-execution logs warning, requires restart
  - Document restart requirement in error message

- **Grammar**: `sea.pest`

  ```pest
  file_header = { ... | collation_directive }
  collation_directive = { "@collation" ~ string_literal }
  ```

  - Example: `@collation "en-US:ci"` (case-insensitive)
  - Example: `@collation "de:phonebook"` (German phonebook ordering)

- **Operators**: `policy/core.rs`
  - Add case-insensitive variants: `icontains`, `istartswith`, `iendswith`
  - Grammar:

    ```pest
    comparison_op = { "=" | "!=" | ... | "icontains" | "istartswith" | "iendswith" }
    ```

  - Unicode normalization (NFC) preprocessing before comparison

- **Slim Build**: Document in README.md

  ```bash
  # Without ICU (no collation support)
  cargo build --no-default-features --features core

  # Binary size: ~5MB vs ~7MB with ICU
  ```

- **Tests**: `tests/collation_tests.rs`
  - German umlauts: `"Müller" icontains "muller"` → `true`
  - French accents: `"Société" istartswith "societe"` → `true`
  - CJK characters: `"北京" = "北京"` (normalized)
  - Arabic RTL: `"الشركة" = "الشركة"` (bidirectional)
  - Mixed scripts: `"ABC-αβγ-123" contains "abc"` (case-insensitive)
  - Unsupported locale fallback: `@collation "pt-BR:ci"` → warning + root UCA

**Documentation**:

- Supported locales and sensitivity levels (`:ci`, `:phonebook`, etc.)
- Custom ICU data directory setup guide
- Slim build instructions for embedded systems
- Performance impact (1-3% overhead for collation-aware comparisons)

---

#### 4. Logical Namespace System with `.sea-registry.toml` and Glob Patterns

**Duration**: 2 weeks | **Priority**: P0 | **Complexity**: High

Build module/import system with logical namespace resolution and hexagonal adapter architecture.

**Grammar**: `sea.pest`

```pest
file = { SOI ~ file_header* ~ namespace_decl? ~ import_decl* ~ declaration* ~ EOI }
namespace_decl = { "namespace" ~ qualified_name }
import_decl = { "import" ~ qualified_name ~ ("as" ~ identifier)? }
export_decl = { "export" ~ (entity_decl | resource_decl | flow_decl | policy_decl) }
qualified_name = { identifier ~ ("." ~ identifier)* }
```

**Registry Format**: `.sea-registry.toml` (TOML table structure)

```toml
[namespace."com.acme"]
path = "./vendor/acme/"
version = "1.2.3"  # metadata-only in Phase 18

[namespace."com.acme.finance"]
path = "./vendor/acme/finance/"
version = "2.0.0"

# Glob patterns
[namespace."com.acme.*"]
path = "./vendor/acme/*/"
```

**Implementation**:

- **Hexagonal Architecture**: `src/module/`

  - **Port**: `ports.rs`

    ```rust
    pub trait ModuleResolver: Send + Sync {
        fn resolve(&self, namespace: &str) -> Result<ModuleSource, ResolveError>;
        fn list_modules(&self) -> Vec<String>;
    }

    pub enum ModuleSource {
        FileSystem(PathBuf),
        Embedded(String),  // For standard library
        Remote(Url),       // Phase 19+
    }
    ```

  - **Adapter**: `adapters/filesystem.rs`
    - Default mapping: `com.acme.finance` → `<workspace>/com/acme/finance.sea`
    - Configurable root directory

  - **Adapter**: `adapters/registry.rs`
    - Load `.sea-registry.toml` from workspace root
    - Parse TOML with `toml` crate
    - **Glob pattern matching** with `glob` crate
    - **Longest prefix wins**: If both `com.acme.*` and `com.acme.finance` match, use `com.acme.finance`
    - Version field stored as metadata (not enforced in Phase 18)

  - **Adapter**: `adapters/remote.rs`
    - Scaffold for Phase 19+ (HTTP/Git registry)
    - Return `Err(Unsupported)` for now

  - **Core**: `resolver.rs`
    - `ImportResolver` with circular dependency detection via topological sort
    - Two-pass parsing: collect exports, resolve imports
    - Cache resolved modules to avoid redundant parsing

- **ConceptId Extension**: `concept_id.rs`

  ```rust
  pub struct ConceptId {
      uuid: Uuid,
      namespace: String,
      module_path: Option<String>,  // e.g., "com.acme.finance"
      name: String,
  }

  impl ConceptId {
      pub fn qualified_name(&self) -> String {
          match &self.module_path {
              Some(path) => format!("{}.{}", path, self.name),
              None => self.name.clone(),
          }
      }
  }
  ```

- **Parser Integration**: `parser/ast.rs`
  - Extend `FileMetadata` with `namespace: Option<String>`, `imports: Vec<Import>`
  - Import resolution phase after parsing
  - Generate `ConceptId` with module_path from namespace context

- **Schema**: `schemas/sea-registry.schema.json`
  - JSON Schema for `.sea-registry.toml` validation
  - Validate path patterns, version format (semver)

- **Documentation**: `docs/specs/namespace_resolution.md`
  - **Precedence rules**:
    1. Exact match (`com.acme.finance`)
    2. Longest glob prefix (`com.acme.*` > `com.*`)
    3. Filesystem fallback (default mapping)
  - Clarify: "Longest glob prefix" refers to the longest literal (non-wildcard) left-to-right path segment prefix. Example: `com.acme.` outranks `com.` for `com.acme.finance`. When two patterns tie on literal prefix length but both match (e.g., `com.` vs `.acme` for `com.acme`), flag the configuration as ambiguous and surface a validation error instead of picking based on lexicographic order or discovery time.
  - Version field usage (metadata-only, defer enforcement to Phase 19+)
  - Remote registry preparation (document future API)

- **Tests**: `tests/module_system_tests.rs`
  - Cross-file imports: `import com.acme.finance` → loads `com/acme/finance.sea`
  - Circular dependency detection: A imports B imports A → error
  - Glob pattern precedence: `com.acme.*` vs `com.acme.finance` → use specific
  - Ambiguous patterns: `com.*` and `*.acme` both match `com.acme` with equal literal prefix lengths → validation error (no silent tie-breaking)
  - Missing modules: `import nonexistent.module` → clear error with "did you mean?"
  - Export/import round-trip: export from A, import in B, verify UUIDs stable

**Documentation**:

- Namespace best practices (avoid deep nesting, use domain-driven structure)
- `.sea-registry.toml` examples for monorepos
- Migration guide from single-file to multi-module projects

---

#### 5. Violation Severity Mapping with Deterministic Rules

**Duration**: 1 week | **Priority**: P0 | **Complexity**: Low

Standardize violation severity mapping with explicit precedence and CLI exit codes.

**Mapping Rules**: `policy/violation.rs`

```rust
pub fn compute_severity(policy: &Policy) -> Severity {
    // Explicit override takes precedence
    if let Some(explicit) = policy.severity_override {
        return explicit;
    }

    // Computed mapping
    match (policy.kind, policy.modality, policy.priority) {
        (PolicyKind::Constraint, Modality::Prohibition, p) if p >= 5 => Severity::Error,
        (PolicyKind::Constraint, Modality::Obligation, p) if p >= 5 => Severity::Error,
        (_, _, p) if p >= 3 && p < 5 => Severity::Warning,
        _ => Severity::Info,
    }
}
```

**Implementation**:

- **Grammar**: `sea.pest`

  ```pest
  policy_decl = { "@severity" ~ severity_level ~ policy_header ~ policy_expr }
  severity_level = { "Error" | "Warning" | "Info" }
  ```

- **File**: `policy/severity.rs`

  ```rust
  pub struct SeverityResolver {
      rationale: HashMap<ConceptId, String>,  // Debug info
  }

  impl SeverityResolver {
      pub fn resolve(&self, policy: &Policy) -> (Severity, String) {
          let severity = compute_severity(policy);
          let rationale = match policy.severity_override {
              Some(_) => "Explicit override".to_string(),
              None => format!("Computed from kind={:?}, modality={:?}, priority={}",
                             policy.kind, policy.modality, policy.priority),
          };
          (severity, rationale)
      }
  }
  ```

- **CLI**: `bin/sea.rs`

  ```rust
  fn main() -> ExitCode {
      let result = validate(input);

      let has_errors = result.violations.iter()
          .any(|v| v.severity == Severity::Error);
      let has_warnings = result.violations.iter()
          .any(|v| v.severity == Severity::Warning);

      match (has_errors, has_warnings) {
          (true, _) => ExitCode::from(2),    // Errors present
          (false, true) => ExitCode::from(1), // Warnings only
          (false, false) => ExitCode::SUCCESS, // Clean
      }
  }
  ```

- **Tests**: `tests/severity_mapping_tests.rs`
  - All combinations of (kind, modality, priority) → expected severity
  - Explicit override: `@severity Error` on priority=1 policy → Error
  - CLI exit code verification: errors=2, warnings=1, clean=0
  - Rationale tracking: verify debug messages explain severity derivation

**Documentation**:

- Severity mapping table in `docs/specs/policy_semantics.md`
- CLI exit code contract for CI/CD integration
- Best practices: when to use explicit severity overrides

---

#### 6. Enhanced Diagnostics with Fuzzy Matching and Formatters

**Duration**: 1.5 weeks | **Priority**: P1 | **Complexity**: Medium

Enhance error diagnostics with "did you mean?" suggestions and structured output formats.

**Implementation**:

- **Error Extension**: `error.rs`

  ```rust
  pub struct ValidationError {
      pub code: ErrorCode,           // NEW: E001, E002, etc.
      pub range: SourceRange,        // NEW: line:col start/end
      pub message: String,
      pub hint: Option<String>,      // Enhanced with fuzzy matches
      pub severity: Severity,
  }

  pub struct SourceRange {
      pub start: Position,  // line, column
      pub end: Position,
  }

  pub enum ErrorCode {
      E001_UndefinedEntity,
      E002_UndefinedResource,
      E003_UnitMismatch,
      E004_TypeMismatch,
      // ... 50+ error codes
  }
  ```

- **Fuzzy Matching**: `error/fuzzy.rs`

  ```rust
  pub fn levenshtein_distance(a: &str, b: &str) -> usize {
      // Wagner-Fischer algorithm (O(mn) dynamic programming)
      // ...
  }

  pub fn suggest_similar(
      target: &str,
      candidates: &[String],
      threshold: usize  // Max edit distance (default 2)
  ) -> Vec<String> {
      candidates.iter()
          .filter(|c| levenshtein_distance(target, c) <= threshold)
          .map(|s| s.clone())
          .collect()
  }
  ```

  - Apply to `UndefinedEntity`, `UndefinedResource`, `UnknownIdentifier` errors
  - Example: `Entity "Warehous"` → "Did you mean 'Warehouse'?"

- **Formatters**: `error/diagnostics.rs`

  ```rust
  pub trait DiagnosticFormatter {
      fn format(&self, error: &ValidationError) -> String;
  }

  pub struct JsonDiagnostic;  // For CI tools
  pub struct HumanDiagnostic; // For developers (color + snippets)
  pub struct LspDiagnostic;   // For IDEs (LSP protocol)
  ```

  - **JSON**: Structured for machine parsing

    ```json
    {
      "code": "E001",
      "severity": "error",
      "range": {"start": {"line": 10, "col": 5}, "end": {"line": 10, "col": 15}},
      "message": "Undefined entity 'Warehous'",
      "hint": "Did you mean 'Warehouse'?"
    }
    ```

  - **Human**: Color-coded with source snippets (use `annotate-snippets` crate)

    ```
    error[E001]: Undefined entity 'Warehous'
      --> example.sea:10:5
       |
    10 |     Flow "Steel" from "Warehous" to "Factory" quantity 100
       |                       ^^^^^^^^^^ not found
       |
       = hint: Did you mean 'Warehouse'?
    ```

  - **LSP**: Compatible with Language Server Protocol

    ```json
    {
      "range": {...},
      "severity": 1,  // Error
      "code": "E001",
      "source": "sea-dsl",
      "message": "Undefined entity 'Warehous'\n\nDid you mean 'Warehouse'?"
    }
    ```

- **Snapshot Tests**: `tests/diagnostics_tests.rs`
  - Use `insta` crate for snapshot testing
  - One test per error code with expected output
  - Example:

    ```rust
    #[test]
    fn test_e003_unit_mismatch_diagnostic() {
        let error = ValidationError::unit_mismatch(
            Dimension::Mass, Dimension::Currency, "line 15"
        );

        let output = HumanDiagnostic.format(&error);
        insta::assert_snapshot!(output);
    }
    ```

**Documentation**:

- Error code catalog in `docs/specs/error_codes.md`
- Diagnostic format examples for CI integration
- IDE extension guide (LSP protocol implementation)

---

#### 7. CLI Commands for Projection/Import/Validation/Testing

**Duration**: 1.5 weeks | **Priority**: P0 | **Complexity**: Medium

Implement comprehensive CLI with subcommands for all major operations.

**CLI Structure**: `src/bin/sea.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sea")]
#[command(about = "SEA DSL compiler and validator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export to architecture formats
    Project {
        #[arg(long)]
        format: ProjectFormat,  // calm, kg, sbvr

        #[arg(value_name = "FILE")]
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import from architecture formats
    Import {
        #[arg(long)]
        format: ProjectFormat,

        #[arg(value_name = "FILE")]
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate DSL files
    Validate {
        #[arg(value_name = "FILE")]
        input: PathBuf,

        #[arg(long)]
        strict: bool,  // Treat warnings as errors

        #[arg(long)]
        json: bool,  // JSON output for CI

        #[arg(long)]
        exit_code: bool,  // Use exit codes (0=ok, 1=warn, 2=error)

        #[arg(long)]
        round_trip: Option<ProjectFormat>,  // Test export → import
    },

    /// Format DSL files
    Format {
        #[arg(value_name = "FILE")]
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long, default_value = "4")]
        indent: usize,
    },

    /// Run policy validation tests
    Test {
        pattern: Option<String>,  // Glob pattern for test files

        #[arg(long)]
        verbose: bool,
    },
}
```

**Implementation**:

- **Module**: `src/cli/`
  - `project.rs` - Export handlers (CALM/KG/SBVR)
  - `import.rs` - Import handlers
  - `validate.rs` - Validation logic
  - `format.rs` - Pretty-printer
  - `test.rs` - Test runner
  - `validate_kg.rs` - SHACL validation (uses `oxigraph` crate)

- **SHACL Validation**: `cli/validate_kg.rs`

  ```rust
  pub fn validate_kg(turtle: &str) -> Result<Vec<Violation>, KgError> {
      let store = Store::new()?;
      store.load_from_read(Format::Turtle, turtle.as_bytes())?;

      // Load SHACL shapes
      let shapes = include_str!("../../schemas/sea-shapes.ttl");
      store.load_from_read(Format::Turtle, shapes.as_bytes())?;

      // Run SHACL validation
      let report = shacl_validate(&store)?;

      // Convert SHACL violations to SEA violations
      Ok(parse_shacl_report(report))
  }
  ```

- **Round-Trip Validation**:

  ```bash
  sea validate --round-trip calm input.sea
  # 1. Parse input.sea → Graph
  # 2. Export Graph → CALM JSON
  # 3. Validate CALM JSON against schema
  # 4. Import CALM JSON → Graph2
  # 5. Compare Graph vs Graph2 (semantics preserved?)
  ```

**Tests**: `tests/cli_tests.rs`

- Test each subcommand with valid/invalid inputs
- Verify exit codes (0, 1, 2)
- Check JSON output format
- Round-trip tests for all formats

**Documentation**:

- CLI reference in README.md and `sea --help`
- CI/CD integration examples (GitHub Actions, GitLab CI)
- Test runner usage guide

---

#### 8. Temporal Policies with Redb and Background Index Migration

**Duration**: 2 weeks | **Priority**: P2 | **Complexity**: High

Implement temporal policy support with hexagonal storage architecture and redb backend.

**Benchmark gating:** Before committing to Step 8, run the redb benchmark against a 1M-flow workload (simulate export from `sea-core/tests/temporal`). If the <50ms target holds, proceed. If not, escalate to building the PostgreSQL adapter within Phase 18 (or, if necessary, temporarily reduce the acceptance dataset) and log the decision criteria so the risk mitigation section records the fallback path.

**Grammar**: `sea.pest`

```pest
file_header = { ... | asof_directive }
asof_directive = { "@asof" ~ iso8601_timestamp }
iso8601_timestamp = { ... }  // "2025-11-17T12:00:00Z"

// Window syntax in policy expressions
window_expr = { "last" ~ integer ~ time_unit }
time_unit = { "days" | "hours" | "minutes" | "seconds" }
```

**Primitive Extension**: `primitives/flow.rs`

```rust
pub struct Flow {
    // ... existing fields ...
    pub timestamp: Option<DateTime<Utc>>,  // NEW: Optional temporal info
}
```

**Hexagonal Architecture**: `src/temporal/`

- **Port**: `ports.rs`

  ```rust
  pub trait TemporalStore: Send + Sync {
      fn insert_flow(&mut self, flow: &Flow) -> Result<(), TemporalError>;

      fn query_range(
          &self,
          start: DateTime<Utc>,
          end: DateTime<Utc>,
          resource_filter: Option<ConceptId>,
      ) -> Result<Vec<Flow>, TemporalError>;

      fn aggregate_window(
          &self,
          resource: ConceptId,
          window: Duration,
          aggregation: AggregationType,  // Sum, Avg, Count, Min, Max
      ) -> Result<Decimal, TemporalError>;

      // NEW: Adapter capabilities
      fn capabilities(&self) -> TemporalCapabilities;
  }

  pub struct TemporalCapabilities {
      pub supports_concurrent_writes: bool,
      pub supports_transactions: bool,
      pub max_retention_days: Option<u32>,
  }
  ```

- **Adapter**: `adapters/redb.rs`
  - Use `redb` crate (pure Rust, ACID, embedded)
  - **Versioned database header**: `sea_temporal_v1`
  - Table schema: `timestamp` (indexed), `resource_id`, `from_id`, `to_id`, `quantity`
  - **Auto-migration** for backward-compatible changes (e.g., new nullable column)
  - **Background index rebuild** on first access after schema change
    - Show progress bar in CLI: `Rebuilding index... [####----] 50%`
  - **Single-writer** constraint (redb default)
  - Document multi-writer requirement for PostgreSQL adapter (Phase 19+)
  - Capabilities: `{ concurrent_writes: false, transactions: true, retention: None }`

- **Adapter**: `adapters/memory.rs`
  - In-memory sorted Vec for testing
  - Fast queries, no persistence
  - Capabilities: `{ concurrent_writes: true, transactions: false, retention: None }`

- **Schema**: `temporal/schema.rs`

  ```rust
  // Redb table definitions
  pub const FLOW_TABLE: TableDefinition<(u64, [u8; 16]), [u8]> =
      TableDefinition::new("flows_v1");
  // Key: (timestamp_micros, flow_id_bytes)
  // Value: bincode-serialized Flow

  pub const SCHEMA_VERSION: u32 = 1;

  pub fn migrate_v0_to_v1(db: &Database) -> Result<(), MigrationError> {
      // Example migration
  }
  ```

- **CLI Commands**: `cli/temporal.rs`

  ```bash
  sea temporal migrate --from-version 0 --to-version 1
  sea temporal reindex --force  # Immediate rebuild
  sea temporal stats            # Show database size, index status
  ```

- **Policy Integration**: `policy/quantifier.rs`

  ```rust
  // New syntax: sum(f in flows where f.timestamp in last 30 days: f.quantity)
  pub enum TemporalFilter {
      Last { duration: Duration },
      Between { start: DateTime<Utc>, end: DateTime<Utc> },
      At { instant: DateTime<Utc> },
  }
  ```

**Feature Flag**: `Cargo.toml`

```toml
[features]
temporal = ["redb", "chrono"]

[dependencies]
redb = { version = "2.0", optional = true }
```

**Tests**: `tests/temporal_policies_tests.rs`

```rust
#[cfg(feature = "temporal")]
mod tests {
    #[test]
    fn test_windowed_aggregation() {
        let mut store = RedbAdapter::new("test.db")?;

        // Insert flows with timestamps
        for day in 0..60 {
            let flow = Flow::new_with_timestamp(
                resource_id.clone(),
                from_id.clone(),
                to_id.clone(),
                Decimal::new(100, 0),
                Utc::now() - Duration::days(day),
            );
            store.insert_flow(&flow)?;
        }

        // Query: sum of last 30 days
        let result = store.aggregate_window(
            resource_id,
            Duration::days(30),
            AggregationType::Sum,
        )?;

        assert_eq!(result, Decimal::new(3000, 0));  // 30 flows × 100
    }

    #[test]
    fn test_schema_migration() {
        // Test v0 → v1 migration
    }
}
```

**Documentation**: `docs/specs/temporal_storage.md`

- Adapter interface guide with RocksDB/PostgreSQL examples:

  ```rust
  // Example: PostgreSQL adapter for multi-writer scenarios
  pub struct PostgresAdapter {
      pool: PgPool,
  }

  impl TemporalStore for PostgresAdapter {
      fn capabilities(&self) -> TemporalCapabilities {
          TemporalCapabilities {
              supports_concurrent_writes: true,  // PostgreSQL MVCC
              supports_transactions: true,
              max_retention_days: Some(365),
          }
      }
      // ...
  }
  ```

- Migration strategy (versioning, backward compatibility)
- Performance tuning (index optimization, query patterns)

---

#### 9. Quantity Rendering, Type Inference, Lint, and AST Export

**Duration**: 1 week | **Priority**: P1 | **Complexity**: Low

Polish quantity rendering, enforce type safety, add linting, and export AST schema.

**Quantity Rendering**: `primitives/quantity.rs`

```rust
use icu::decimal::FixedDecimalFormatter;
use icu::locid::Locale;

pub struct QuantityFormatter {
    formatter: FixedDecimalFormatter,
    locale: Locale,
}

impl QuantityFormatter {
    pub fn format(&self, quantity: &Quantity) -> String {
        let decimal = self.formatter.format_to_string(&quantity.value);
        format!("{} \"{}\"", decimal, quantity.unit)
    }
}

// Examples:
// en-US: 1500.00 "USD" → "1,500.00 \"USD\""
// de-DE: 1500.00 "EUR" → "1.500,00 \"EUR\""
// fr-FR: 1500.00 "EUR" → "1 500,00 \"EUR\""
```

**Type Inference**: `policy/type_inference.rs`

```rust
pub enum ExprType {
    Quantity { dimension: Dimension },  // NEW: Separate from Numeric
    Numeric,
    String,
    Boolean,
    Collection(Box<ExprType>),
}

// Enforce explicit casts in comparisons
pub fn check_comparison(left: &ExprType, right: &ExprType) -> Result<(), TypeError> {
    match (left, right) {
        (ExprType::Quantity { dim: d1 }, ExprType::Quantity { dim: d2 }) => {
            if d1 != d2 {
                return Err(TypeError::UnitMismatch {
                    expected: d1.clone(),
                    found: d2.clone(),
                    hint: format!("Convert using 'as \"{}\"'", d1.default_unit()),
                });
            }
        }
        (ExprType::Quantity { .. }, ExprType::Numeric) => {
            return Err(TypeError::MixedQuantityNumeric {
                hint: "Quantities require explicit units. Did you mean 'value as \"unit\"'?",
            });
        }
        // ...
    }
    Ok(())
}
```

**Linter**: `parser/lint.rs`

```rust
pub struct Linter {
    keywords: HashSet<String>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            keywords: ["Entity", "Resource", "Flow", "Policy", "Unit", "Dimension"]
                .iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn check_identifier(&self, name: &str, quoted: bool) -> Result<(), LintError> {
        if !quoted && self.keywords.contains(name) {
            return Err(LintError::KeywordCollision {
                name: name.to_string(),
                hint: format!("Use quoted identifier: \"{}\"", name),
            });
        }
        Ok(())
    }
}

// Example: Entity Policy → Error ("Policy" is keyword, use "Policy" with quotes)
```

**AST Schema**: `schemas/ast-v1.schema.json`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SEA DSL Abstract Syntax Tree",
  "version": "1.0.0",
  "type": "object",
  "properties": {
    "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
    "namespace": { "type": "string" },
    "imports": {
      "type": "array",
      "items": { "$ref": "#/definitions/Import" }
    },
    "entities": { ... },
    "resources": { ... },
    "flows": { ... },
    "policies": { ... }
  },
  "definitions": { ... }
}
```

**Pretty-Printer**: `parser/printer.rs`

```rust
pub struct PrettyPrinter {
    indent_width: usize,
    max_line_length: usize,
    trailing_commas: bool,
}

impl PrettyPrinter {
    pub fn print(&self, ast: &Ast) -> String {
        let mut output = String::new();

        // File header
        if let Some(ns) = &ast.namespace {
            writeln!(output, "namespace {}", ns).unwrap();
            writeln!(output).unwrap();
        }

        // Imports
        for import in &ast.imports {
            writeln!(output, "import {}", import.path).unwrap();
        }

        // Entities
        for entity in &ast.entities {
            writeln!(output, "Entity \"{}\"", entity.name).unwrap();
        }

        // ... format other primitives with configurable indentation

        output
    }
}
```

**Tests**:

- Quantity formatter locale tests: en/de/fr/zh/ar
- Type inference: mixed Quantity/Numeric → error
- Linter: keyword collision detection
- AST schema: validate against jsonschema crate
- Pretty-printer: parse → print → parse → compare ASTs

**Documentation**:

- Quantity rendering guide with locale examples
- Type system specification (Quantity vs Numeric)
- Linting rules and keyword list
- AST schema versioning policy

---

#### 10. CI/CD Guardrails with Semver Enforcement and Security Scanning

**Duration**: 1.5 weeks | **Priority**: P0 | **Complexity**: Medium

Implement comprehensive CI/CD guardrails for quality gates and semver enforcement.

**GitHub Actions**: `.github/workflows/guardrails.yml`

```yaml
name: CI Guardrails

on: [pull_request, push]

jobs:
  parse_check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Parse all DSL files
        run: |
          find . -name "*.sea" -exec sea validate {} \;

  type_check:
    runs-on: ubuntu-latest
    steps:
      - name: Type and unit validation
        run: sea validate --strict *.sea

  determinism_test:
    runs-on: ubuntu-latest
    steps:
      - name: Run determinism tests
        run: cargo test determinism --features temporal

  diagnostics:
    runs-on: ubuntu-latest
    steps:
      - name: Generate diagnostics report
        run: sea validate --json *.sea > diagnostics.json
      - uses: actions/upload-artifact@v3
        with:
          name: diagnostics
          path: diagnostics.json

  semver_check:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Semver diff analysis
        run: |
          # Export baseline from main branch
          git checkout main
          sea project --format calm main.sea > baseline.json

          # Export candidate from PR branch
          git checkout ${{ github.head_ref }}
          sea project --format calm main.sea > candidate.json

          # Compare and enforce semver
          sea semver diff baseline.json candidate.json --enforce

  security_audit:
    runs-on: ubuntu-latest
    steps:
      - name: Cargo deny (license/ban checks)
        run: cargo deny check licenses bans

      - name: Cargo audit (CVE scanning)
        run: cargo audit

  projection_drift:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - name: Check projection drift
        run: |
          # Fail if semantics changed without @version bump
          sea validate --round-trip calm main.sea || {
            echo "❌ Projection drift detected. Bump @version in file header."
            exit 1
          }
```

**Semver Diff**: `src/cli/semver.rs`

```rust
pub enum SemverChange {
    Major,  // Breaking: removed primitives, changed types
    Minor,  // Non-breaking: added primitives, new attributes
    Patch,  // Metadata: comments, formatting, @description changes
}

pub fn diff_asts(baseline: &Ast, candidate: &Ast) -> SemverChange {
    // Check for removed primitives
    let removed_entities = baseline.entities.iter()
        .filter(|e| !candidate.entities.iter().any(|c| c.id == e.id))
        .count();

    if removed_entities > 0 {
        return SemverChange::Major;
    }

    // Check for added primitives
    let added_entities = candidate.entities.iter()
        .filter(|e| !baseline.entities.iter().any(|b| b.id == e.id))
        .count();

    if added_entities > 0 {
        return SemverChange::Minor;
    }

    // Only metadata changed
    SemverChange::Patch
}

pub fn enforce_version_bump(
    baseline: &Ast,
    candidate: &Ast,
    change: SemverChange,
) -> Result<(), SemverError> {
    let baseline_version = baseline.metadata.version;
    let candidate_version = candidate.metadata.version;

    match change {
        SemverChange::Major if candidate_version.major <= baseline_version.major => {
            Err(SemverError::MajorBumpRequired)
        }
        SemverChange::Minor if candidate_version.minor <= baseline_version.minor => {
            Err(SemverError::MinorBumpRequired)
        }
        SemverChange::Patch if candidate_version.patch <= baseline_version.patch => {
            Err(SemverError::PatchBumpRequired)
        }
        _ => Ok(()),
    }
}
```

**Validate JSON Output**: `src/cli/validate.rs`

```rust
pub fn validate_json(input: &Path, exit_code: bool) -> ExitCode {
    let result = parse_and_validate(input);

    // JSON output
    let json = serde_json::json!({
        "file": input.display().to_string(),
        "violations": result.violations.iter().map(|v| {
            serde_json::json!({
                "code": v.code,
                "severity": v.severity,
                "message": v.message,
                "range": v.range,
                "hint": v.hint,
            })
        }).collect::<Vec<_>>(),
        "summary": {
            "errors": result.violations.iter().filter(|v| v.severity == Severity::Error).count(),
            "warnings": result.violations.iter().filter(|v| v.severity == Severity::Warning).count(),
            "info": result.violations.iter().filter(|v| v.severity == Severity::Info).count(),
        }
    });

    println!("{}", serde_json::to_string_pretty(&json).unwrap());

    if exit_code {
        result.exit_code()
    } else {
        ExitCode::SUCCESS
    }
}
```

**Tests**: `tests/ci_guardrails_tests.rs`

- Semver diff: MAJOR (removed entity), MINOR (added resource), PATCH (comment change)
- Version enforcement: detect missing version bump
- JSON output: validate schema
- Projection drift: CALM round-trip failure detection

**Documentation**: `docs/specs/ci_cd_guardrails.md`

- GitHub Actions workflow setup
- Semver enforcement policy
- Security scanning configuration
- Projection drift detection strategy

---

### Cross-Cutting Concerns

#### Testing Strategy

**Test Pyramid**:

1. **Unit Tests** (70%): Individual functions, pure logic
   - `tests/*_tests.rs` files
   - Property-based tests with `proptest` for invariants
2. **Integration Tests** (20%): Multi-component interactions
   - `tests/*_integration_tests.rs` files
   - Cross-language parity (Rust ≡ Python ≡ TypeScript)
3. **End-to-End Tests** (10%): CLI workflows, round-trips
   - `tests/e2e_*.rs` files
   - Golden file tests with snapshots

**TDD RED-GREEN-REFACTOR**:

- RED: Write failing test first
- GREEN: Implement minimal code to pass
- REFACTOR: Improve code quality without changing behavior
- VALIDATE: Run full test suite + cross-language tests

#### Documentation Requirements

Each step must include:

1. **Normative Specifications**: Formal behavior definitions in specs
2. **API Documentation**: Rustdoc comments with examples
3. **User Guides**: Tutorial-style docs in `docs/guides/`
4. **Migration Guides**: Upgrade paths for breaking changes

#### Performance Benchmarks

Add benchmarks for:

- Three-valued logic overhead (target: 5-10%)
- ICU collation impact (target: 1-3%)
- Temporal query performance (target: <50ms for 1M flows)
- Redb index rebuild (target: 1M flows in <30s)

#### Cross-Language Parity

For each Rust feature, verify equivalence in:

- **Python** (PyO3 bindings): `tests/test_*.py`
- **TypeScript** (napi-rs bindings): `typescript-tests/*.test.ts`
- **WASM** (wasm-bindgen): `sea-core/tests/*_wasm_tests.rs`

---

### Success Criteria

**Phase 18 Complete When**:

1. ✅ All 10 steps implemented with passing tests
2. ✅ Documentation complete (specs + guides + API docs)
3. ✅ Cross-language parity verified (342+ tests passing)
4. ✅ Performance benchmarks meet targets
5. ✅ CI/CD guardrails enforced in GitHub Actions
6. ✅ No regressions in existing functionality
7. ✅ Code review completed by maintainers

**Acceptance Tests**:

- Complex enterprise model (1000+ entities, 5000+ flows) validates in <500ms
- Round-trip tests (DSL → CALM/KG/SBVR → DSL) preserve semantics
- Three-valued logic handles NULL correctly (20+ edge cases)
- Module system resolves imports from filesystem/registry/remote
- Temporal queries run <50ms on 1M historical flows
- CLI tools integrate seamlessly with CI/CD pipelines

---

### Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| ICU data size bloat | Medium | Medium | Minimal locale bundle, slim build option |
| Redb performance issues | Low | High | Benchmark early (1M-flow redb run); if <50ms target fails, escalate to the PostgreSQL adapter implementation or lower the acceptance scale and document the decision path. |
| SBVR XMI parsing complexity | Medium | Medium | Start with minimal subset, defer extensions |
| Three-valued logic bugs | High | High | Extensive property-based tests, truth table validation |
| Namespace collision | Medium | Low | Longest-prefix matching, clear error messages |
| Temporal schema migration failures | Medium | High | Versioned databases, auto-migration with rollback |

---

### Phase 19+ Roadmap (Future Work)

**Deferred Items**:

1. SBVR vocabulary extensions (synonyms, definitions, structured glossaries)
2. Remote namespace registry (HTTP/Git with version constraints)
3. Multi-writer temporal storage (PostgreSQL/CockroachDB adapters)
4. Explicit NULL literal syntax (`f.quantity = NULL`)
5. Advanced SBVR modalities (full Alethic/Deontic/Epistemic support)
6. Property graphs (Neo4j/ArangoDB native adapters)
7. Language Server Protocol (LSP) implementation
8. Visual DSL editor (web-based or VSCode extension)

---

### References

- **Phases 0-17**: Foundation complete (primitives, graph, policy, parser, bindings, CALM)
- **ADR-001**: Layered Architecture (Vocabulary → Facts → Rules)
- **ADR-003**: Graph-Based Domain Model (IndexMap determinism)
- **ADR-006**: CALM Interoperability (bidirectional conversion)
- **ADR-007**: Idiomatic Language Bindings (PyO3, napi-rs, wasm-bindgen)
- **PRD**: Product Requirements Document (success metrics, use cases)
- **SDS**: Software Design Specification (component architecture)

---

**END OF PLAN**

This plan is ready for implementation. All architectural decisions finalized, dependencies clarified, and success criteria defined. Follow TDD RED-GREEN-REFACTOR workflow, maintain cross-language parity, and document thoroughly.

