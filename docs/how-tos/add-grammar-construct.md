# How-To: Add a Grammar Construct (Grammar-First Workflow)

This guide walks through extending the SEA Domain-Specific Language with new syntax, following DomainForge's mandatory **Grammar-First Workflow**.

---

## Goal
Add a new declaration or keyword to the SEA DSL syntax and lower it into the AST and Graph Store.

---

## Prerequisites
- Working Rust toolchain.
- Familiarity with Pest PEG syntax.

---

## The 5-Step Procedure

### Step 1: Update the PEG Grammar
Open `domainforge-core/grammar/sea.pest`.

1. Add your new rule to `declaration_inner`:
   ```pest
   declaration_inner = { ... | widget_decl }
   ```
2. Define the new syntax rule:
   ```pest
   widget_decl = {
       ^"widget" ~ name ~ ^"type" ~ string_literal
   }
   ```
3. If introducing a new declaration keyword, add it to `declaration_keyword` to prevent ambiguous identifier parsing.

### Step 2: Update the AST Model
Open `domainforge-core/src/parser/ast.rs`.

1. Add a variant to `AstNode`:
   ```rust
   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   #[serde(tag = "type")]
   pub enum AstNode {
       // ...
       Widget {
           name: String,
           widget_type: String,
           source_range: SourceRange,
       },
   }
   ```
2. In `parse_source()`, match your Pest pair:
   ```rust
   Rule::widget_decl => {
       // Extract inner pairs and construct AstNode::Widget
   }
   ```

### Step 3: Update AST-to-Graph Conversion
Open `domainforge-core/src/parser/ast_convert.rs`.

Handle `AstNode::Widget` by inserting the new concept or attribute into `Graph`:
```rust
AstNode::Widget { name, widget_type, .. } => {
    graph.add_widget(Widget::new(name, widget_type))?;
}
```

### Step 4: Add Parser Integration Tests
Create or edit `domainforge-core/tests/parser_tests.rs`:
```rust
#[test]
fn test_parse_widget_declaration() {
    let source = r#"
        @namespace "ui"
        widget "Button" type "clickable"
    "#;
    let ast = parse(source).expect("parsing should succeed");
    assert!(ast.declarations.iter().any(|d| matches!(d.node, AstNode::Widget { .. })));
}
```

### Step 5: Update Language Bindings (if public API touched)
If your change adds a new public primitive struct, you **must** update:
1. `domainforge-core/src/python/primitives.rs`
2. `domainforge-core/src/typescript/primitives.rs`
3. `domainforge-core/src/wasm/primitives.rs`

---

## Validation
Run the test suites across all three languages:
```bash
just all-tests
```
Verify formatting and clippy:
```bash
cargo fmt --check && cargo clippy --all-targets --features cli -- -D warnings
```
