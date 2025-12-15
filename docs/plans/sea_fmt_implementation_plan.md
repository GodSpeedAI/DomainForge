# SEA Format (`sea fmt`) Implementation Plan

**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Draft

---

## 1. Overview

Implement a code formatter for SEA-DSL files that standardizes whitespace, indentation, and ordering while preserving semantics.

### Goals

- Pretty-print SEA files with consistent formatting
- Preserve all semantic content and comments
- Support configurable indent style (spaces vs tabs, indent width)
- Idempotent output (formatting already-formatted code produces identical output)
- Fast enough for editor integration (format-on-save)

---

## 2. Current State

The `sea fmt` command currently:

- Parses the file to verify syntax validity
- Returns an error: "Formatting not yet implemented"

**File:** `sea-core/src/cli/format.rs` (23 lines)

---

## 3. Architecture

### Design Approach: AST-based Formatting

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Source Code │ ──► │    Parse    │ ──► │     AST     │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Output    │ ◄── │  Formatter  │ ◄── │  AST + Span │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Key Design Decisions

| Decision   | Choice                           | Rationale                               |
| ---------- | -------------------------------- | --------------------------------------- |
| Strategy   | AST-based (not regex/line-based) | Semantic awareness, handles edge cases  |
| Comments   | Preserve via span information    | Comments are critical for documentation |
| Ordering   | Preserve declaration order       | User-defined order is intentional       |
| Whitespace | Normalize to configured style    | Main formatter benefit                  |

---

## 4. Proposed Changes

### Phase 1: Core Formatter Module

#### [NEW] `sea-core/src/formatter/mod.rs`

```rust
pub mod config;
pub mod printer;

pub use config::FormatConfig;
pub use printer::Formatter;
```

#### [NEW] `sea-core/src/formatter/config.rs`

Formatter configuration:

```rust
pub struct FormatConfig {
    pub indent_style: IndentStyle,   // Spaces or Tabs
    pub indent_width: usize,          // Default: 4
    pub max_line_width: usize,        // Default: 100
    pub trailing_newline: bool,       // Default: true
    pub preserve_comments: bool,      // Default: true
}

pub enum IndentStyle {
    Spaces,
    Tabs,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces,
            indent_width: 4,
            max_line_width: 100,
            trailing_newline: true,
            preserve_comments: true,
        }
    }
}
```

#### [NEW] `sea-core/src/formatter/printer.rs`

Core formatter logic:

```rust
pub struct Formatter {
    config: FormatConfig,
    output: String,
    indent_level: usize,
}

impl Formatter {
    pub fn format(source: &str, config: FormatConfig) -> Result<String, FormatError>;

    fn format_file_header(&mut self, meta: &FileMetadata);
    fn format_declaration(&mut self, node: &AstNode);
    fn format_entity(&mut self, name: &str, version: Option<&str>, annotations: &HashMap, domain: Option<&str>);
    fn format_resource(&mut self, ...);
    fn format_flow(&mut self, ...);
    fn format_policy(&mut self, ...);
    fn format_expression(&mut self, expr: &Expression);
    // ... one method per declaration type
}
```

---

### Phase 2: Comment Preservation

#### [MODIFY] `sea-core/grammar/sea.pest`

Update COMMENT rule to capture (not discard):

```diff
-COMMENT = _{ "//" ~ (!"\n" ~ ANY)* }
+COMMENT = { "//" ~ (!"\n" ~ ANY)* }
```

#### [MODIFY] `sea-core/src/parser/ast.rs`

Add comment attachment to AST nodes:

```rust
pub struct CommentedNode<T> {
    pub node: T,
    pub leading_comments: Vec<String>,
    pub trailing_comment: Option<String>,
}
```

---

### Phase 3: CLI Integration

#### [MODIFY] `sea-core/src/cli/format.rs`

```rust
use crate::formatter::{FormatConfig, Formatter};

#[derive(Parser)]
pub struct FormatArgs {
    pub file: PathBuf,

    #[arg(long, default_value = "stdout")]
    pub out: Output,

    #[arg(long, default_value = "4")]
    pub indent_width: usize,

    #[arg(long)]
    pub use_tabs: bool,

    #[arg(long)]
    pub check: bool,  // Exit 1 if file would change (for CI)
}

pub fn run(args: FormatArgs) -> Result<()> {
    let source = read_to_string(&args.file)?;

    let config = FormatConfig {
        indent_width: args.indent_width,
        indent_style: if args.use_tabs { IndentStyle::Tabs } else { IndentStyle::Spaces },
        ..Default::default()
    };

    let formatted = Formatter::format(&source, config)?;

    if args.check {
        if source != formatted {
            anyhow::bail!("File would be reformatted: {}", args.file.display());
        }
    } else {
        match args.out {
            Output::Stdout => println!("{}", formatted),
            Output::File(path) => write(path, formatted)?,
        }
    }

    Ok(())
}
```

---

## 5. Formatting Rules

### File Structure

```
// Header annotations (in order: namespace, version, owner, profile)
@namespace "com.example"
@version "1.0.0"

// Imports (sorted alphabetically by path)
import { Entity } from "common.sea"
import { Resource } from "resources.sea"

// Declarations (preserved order)
Entity "Name" in domain

Resource "Name" units in domain

Flow "Resource" from "A" to "B" quantity 100
```

### Indentation Rules

| Context                 | Indentation         |
| ----------------------- | ------------------- |
| Top-level declarations  | 0                   |
| Instance body           | 1 level             |
| Relation fields         | 1 level             |
| Mapping/Projection body | 1 level             |
| Policy expression       | continuation indent |

### Spacing Rules

| Pattern          | Rule              |
| ---------------- | ----------------- |
| After keywords   | Single space      |
| Around operators | Single space      |
| After colons     | Single space      |
| Before braces    | Single space      |
| Empty lines      | Max 1 consecutive |

---

## 6. Implementation Phases

### Phase 1: Basic Formatting (MVP)

- [ ] Create formatter module structure
- [ ] Implement `FormatConfig`
- [ ] Implement basic `Formatter` with entity, resource, flow support
- [ ] Wire up CLI

### Phase 2: Full Declaration Support

- [ ] Add all declaration types (14 total)
- [ ] Add expression formatting
- [ ] Handle complex nested expressions

### Phase 3: Comment Preservation

- [ ] Modify grammar to capture comments
- [ ] Attach comments to AST nodes
- [ ] Preserve comments in output

### Phase 4: Polish

- [ ] Add `--check` flag for CI
- [ ] Add idempotency tests
- [ ] Performance optimization
- [ ] Editor integration docs

---

## 7. Verification Plan

### Unit Tests

Create `sea-core/tests/formatter_tests.rs`:

```rust
#[test]
fn test_format_entity_basic() {
    let input = "Entity   \"Foo\"  in    bar";
    let expected = "Entity \"Foo\" in bar\n";
    assert_eq!(format(input), expected);
}

#[test]
fn test_format_preserves_comments() {
    let input = "// Header\nEntity \"Foo\"";
    let expected = "// Header\nEntity \"Foo\"\n";
    assert_eq!(format(input), expected);
}

#[test]
fn test_format_idempotent() {
    let input = include_str!("fixtures/payment.sea");
    let once = format(input);
    let twice = format(&once);
    assert_eq!(once, twice);
}
```

**Run tests:**

```bash
cargo test -p sea-core formatter
```

### Integration Tests

Test against example files:

```bash
# Format and verify output is valid
sea fmt examples/basic.sea > /tmp/formatted.sea
sea validate /tmp/formatted.sea

# Check mode for CI
sea fmt --check examples/basic.sea
```

### Manual Verification

1. Format a complex model file
2. Compare before/after visually
3. Verify semantics unchanged via:
   ```bash
   sea project --format calm before.sea > before.json
   sea fmt before.sea --out after.sea
   sea project --format calm after.sea > after.json
   diff before.json after.json  # Should be identical
   ```

---

## 8. Risks and Mitigations

| Risk                            | Mitigation                                        |
| ------------------------------- | ------------------------------------------------- |
| Comment preservation complexity | Start without comments in Phase 1, add in Phase 3 |
| Breaking existing workflows     | Add `--check` mode for gradual adoption           |
| Performance on large files      | Benchmark, optimize only if needed                |
| Grammar changes break parser    | Extensive parser tests before modifying           |

---

## 9. Open Questions

1. **Should imports be auto-sorted?** (Proposed: Yes, alphabetically by path)
2. **Should declaration order be enforced?** (Proposed: No, preserve user order)
3. **Configuration file support?** (Proposed: Future - `.seafmt.toml`)

---

## 10. Related Documents

- [Grammar Spec](../reference/grammar-spec.md)
- [CLI Commands](../reference/cli-commands.md)
- [ADR-001: SEA-DSL as Semantic Source of Truth](../specs/ADR-001-sea-dsl-semantic-source-of-truth.md)

---

## 11. Acceptance Criteria

- [ ] `sea fmt model.sea` outputs formatted code to stdout
- [ ] `sea fmt model.sea --out formatted.sea` writes to file
- [ ] `sea fmt --check model.sea` returns exit code 0 if already formatted, 1 otherwise
- [ ] Formatting is idempotent (format twice = format once)
- [ ] All 14 declaration types are supported
- [ ] Comments are preserved (Phase 3)
- [ ] Unit tests achieve >80% coverage of formatter module
