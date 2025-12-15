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

### Phase 1: Basic Formatting (MVP) ✅ COMPLETE

- [x] Create formatter module structure
- [x] Implement `FormatConfig`
- [x] Implement basic `Formatter` with entity, resource, flow support
- [x] Wire up CLI

### Phase 2: Full Declaration Support ✅ COMPLETE

- [x] Add all declaration types (14 total)
- [x] Add expression formatting
- [x] Handle complex nested expressions

### Phase 3: Comment Preservation

- [ ] Modify grammar to capture comments
- [ ] Attach comments to AST nodes
- [ ] Preserve comments in output

### Phase 4: Polish ✅ COMPLETE

- [x] Add `--check` flag for CI
- [x] Add idempotency tests
- [ ] Performance optimization (deferred - not needed currently)
- [ ] Editor integration docs (deferred)

### Phase 5: Language Bindings (Future)

Expose formatter API to Python, TypeScript, and WASM for programmatic access.

**Use Cases:**

- VS Code extension format-on-save
- Jupyter notebook cell formatting
- Browser-based playground
- CI/CD pipeline integration without shelling out

#### [NEW] `sea-core/src/python/formatter.rs`

```rust
use crate::formatter::{format, FormatConfig};
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (source, indent_width=4, use_tabs=false))]
pub fn format_source(source: &str, indent_width: usize, use_tabs: bool) -> PyResult<String> {
    let config = FormatConfig {
        indent_width,
        indent_style: if use_tabs { IndentStyle::Tabs } else { IndentStyle::Spaces },
        ..Default::default()
    };
    format(source, config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}
```

**Python usage:**

```python
from sea_dsl import format_source

formatted = format_source("""Entity   "Foo"  in    bar""")
print(formatted)  # Entity "Foo" in bar
```

#### [NEW] `sea-core/src/typescript/formatter.rs`

```rust
use crate::formatter::{format, FormatConfig};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn format_source(source: String, indent_width: Option<u32>, use_tabs: Option<bool>) -> napi::Result<String> {
    let config = FormatConfig {
        indent_width: indent_width.unwrap_or(4) as usize,
        indent_style: if use_tabs.unwrap_or(false) { IndentStyle::Tabs } else { IndentStyle::Spaces },
        ..Default::default()
    };
    format(&source, config).map_err(|e| napi::Error::from_reason(e.to_string()))
}
```

**TypeScript usage:**

```typescript
import { formatSource } from "@domainforge/sea";

const formatted = formatSource(`Entity   "Foo"  in    bar`);
console.log(formatted); // Entity "Foo" in bar
```

#### [NEW] `sea-core/src/wasm/formatter.rs`

```rust
use crate::formatter::{format, FormatConfig};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn format_source(source: &str, indent_width: Option<usize>, use_tabs: Option<bool>) -> Result<String, JsValue> {
    let config = FormatConfig {
        indent_width: indent_width.unwrap_or(4),
        indent_style: if use_tabs.unwrap_or(false) { IndentStyle::Tabs } else { IndentStyle::Spaces },
        ..Default::default()
    };
    format(source, config).map_err(|e| JsValue::from_str(&e.to_string()))
}
```

**Browser usage:**

```javascript
import { formatSource } from "@domainforge/sea-wasm";

const formatted = formatSource(`Entity   "Foo"  in    bar`);
document.getElementById("output").textContent = formatted;
```

#### Phase 5 Tasks

- [ ] Add `format_source` function to Python bindings
- [ ] Add `formatSource` function to TypeScript bindings
- [ ] Add `format_source` function to WASM bindings
- [ ] Update binding documentation (python-api.md, typescript-api.md, wasm-api.md)
- [ ] Add cross-language formatter tests

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

### Binding Tests

```python
# tests/test_formatter.py
def test_format_source():
    from sea_dsl import format_source
    result = format_source('Entity   "Foo"')
    assert result == 'Entity "Foo"\n'
```

```typescript
// typescript-tests/formatter.test.ts
import { formatSource } from "@domainforge/sea";
import { expect, test } from "vitest";

test("formatSource normalizes whitespace", () => {
  expect(formatSource('Entity   "Foo"')).toBe('Entity "Foo"\n');
});
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

| Risk                            | Mitigation                                         |
| ------------------------------- | -------------------------------------------------- |
| Comment preservation complexity | Start without comments in Phase 1, add in Phase 3  |
| Breaking existing workflows     | Add `--check` mode for gradual adoption            |
| Performance on large files      | Benchmark, optimize only if needed                 |
| Grammar changes break parser    | Extensive parser tests before modifying            |
| Binding API divergence          | Share core `format()` function across all bindings |

---

## 9. Open Questions

1. **Should imports be auto-sorted?** (Proposed: Yes, alphabetically by path)
2. **Should declaration order be enforced?** (Proposed: No, preserve user order)
3. **Configuration file support?** (Proposed: Future - `.seafmt.toml`)
4. **Should bindings support config objects?** (Proposed: Start with simple args, add config object later)

---

## 10. Related Documents

- [Grammar Spec](../reference/grammar-spec.md)
- [CLI Commands](../reference/cli-commands.md)
- [ADR-001: SEA-DSL as Semantic Source of Truth](../specs/ADR-001-sea-dsl-semantic-source-of-truth.md)
- [Python API](../reference/python-api.md)
- [TypeScript API](../reference/typescript-api.md)
- [WASM API](../reference/wasm-api.md)

---

## 11. Acceptance Criteria

### Core (Phases 1-4)

- [ ] `sea fmt model.sea` outputs formatted code to stdout
- [ ] `sea fmt model.sea --out formatted.sea` writes to file
- [ ] `sea fmt --check model.sea` returns exit code 0 if already formatted, 1 otherwise
- [ ] Formatting is idempotent (format twice = format once)
- [ ] All 14 declaration types are supported
- [ ] Comments are preserved (Phase 3)
- [ ] Unit tests achieve >80% coverage of formatter module

### Bindings (Phase 5)

- [ ] Python: `format_source()` function available in `sea_dsl` module
- [ ] TypeScript: `formatSource()` function exported from `@domainforge/sea`
- [ ] WASM: `format_source()` function available in browser bundle
- [ ] All bindings produce identical output for same input
- [ ] Cross-language formatter tests pass
