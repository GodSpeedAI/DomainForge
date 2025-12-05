# Feature 10: Module System Implementation Plan

This plan details the implementation of the Module System for DomainForge (SEA DSL), enabling `import` and `export` capabilities for better code organization.

## 1. Overview

The goal is to transition from a purely file-based namespace mapping (via `NamespaceRegistry`) to a true module system where files can explicitly import symbols from other namespaces and control their own exports.

### Key Concepts

- **Module**: A single `.sea` file, identified by its namespace (declared or inferred).
- **Export**: A declaration made available to other modules.
- **Import**: A declaration brought into the current scope from another module.
- **Namespace**: The logical grouping identifier (e.g., `com.acme.finance`).

## 2. Grammar Changes (`sea-core/grammar/sea.pest`)

We need to add syntax for imports (at the file level) and exports (at the declaration level).

```pest
// Update file_header to include imports
file_header = { annotation* ~ import_decl* }

// Import Declarations
import_decl = {
    ^"import" ~ import_specifier ~ ^"from" ~ string_literal
}

import_specifier = {
    import_named | import_wildcard
}

import_named = {
    "{" ~ import_item ~ ("," ~ import_item)* ~ "}"
}

import_item = {
    identifier ~ (^"as" ~ identifier)?
}

import_wildcard = { "*" }

// Export Modifier
export_decl = {
    ^"export" ~ declaration_inner
}

// Refactor declaration to support export wrapper
declaration = { export_decl | declaration_inner }

declaration_inner = {
    dimension_decl | unit_decl | entity_decl | resource_decl |
    flow_decl | pattern_decl | role_decl | relation_decl |
    instance_decl | policy_decl | concept_change_decl |
    metric_decl | mapping_decl | projection_decl
}
```

## 3. AST Updates (`sea-core/src/parser/ast.rs`)

We need to capture import statements and the export status of declarations.

```rust
// New AST Nodes
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub specifier: ImportSpecifier,
    pub from_module: String, // The namespace or path being imported
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportSpecifier {
    Named(Vec<ImportItem>),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

// Update FileMetadata
pub struct FileMetadata {
    pub namespace: Option<String>,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub imports: Vec<ImportDecl>, // Added
}

// Update AstNode
// Instead of wrapping every node, we can add an `exported` field to relevant nodes
// OR wrap them in an Export node. Wrapping is cleaner for the AST structure.
pub enum AstNode {
    // ... existing variants ...
    Export(Box<AstNode>), // New variant wrapping the exported declaration
}
```

## 4. Module Resolution Engine (`sea-core/src/module/`)

Create a new module `sea-core/src/module/` to handle the logic of loading and resolving modules.

### `mod.rs`

Public API for the module system.

### `resolver.rs`

Responsible for:

1. Taking an AST and a `NamespaceRegistry`.
2. Resolving `import` strings to actual file paths/namespaces.
3. Building a symbol table for the current scope.

```rust
pub struct ModuleResolver<'a> {
    registry: &'a NamespaceRegistry,
    // Cache of loaded modules to prevent cycles and re-parsing
    loaded_modules: HashMap<String, ModuleInfo>,
}

pub struct ModuleInfo {
    pub namespace: String,
    pub exports: HashMap<String, ConceptId>,
    pub ast: Ast,
}
```

### `loader.rs`

Handles the file I/O and parsing coordination.

- `load_module(path)`: Parses a file.
- `load_dependency_graph(entry_point)`: Recursively loads imports.

## 5. Graph Integration (`sea-core/src/graph/mod.rs`)

The `Graph` needs to be aware of which concepts are exported.

1. **Symbol Visibility**:
    - By default, concepts are "public" within their namespace (legacy behavior).
    - With the module system, we enforce that only `export`ed concepts can be imported by *other* namespaces.
    - Within the *same* namespace, everything is visible (package-private).

2. **Graph Structure**:
    - Add `exports: IndexMap<ConceptId, bool>` or similar metadata to track visibility if needed, though strictly this is a parser/resolver check, not necessarily a runtime graph property.
    - However, for downstream tools (like documentation generators), knowing what is part of the public API is useful.

## 6. CLI Updates (`sea-core/src/bin/sea.rs`)

1. **`sea validate`**:
    - Needs to support multi-file validation via imports.
    - Add `--include-path` or `-I` to specify where to look for imported modules if they aren't in the registry.

2. **`sea module`** (New Subcommand):
    - `sea module list`: Show all discoverable modules in the registry.
    - `sea module deps <file>`: Show the dependency tree of a file.
    - `sea module check <file>`: Verify imports resolve and no cycles exist.

## 7. Error Handling (`sea-core/src/validation_error.rs`)

Add new `ErrorCode` variants:

- `E200_ModuleNotFound`: Import could not be resolved.
- `E201_SymbolNotExported`: Importing a private symbol.
- `E202_CircularDependency`: A -> B -> A cycle detected.
- `E203_AmbiguousImport`: Symbol name conflict.
- `E204_InvalidExport`: Exporting something that doesn't exist or is invalid.

## 8. Bindings

### Python (`sea-core/src/python/`)

- Expose `imports` in `FileMetadata`.
- Allow inspecting exports of a parsed graph.

### TypeScript (`sea-core/src/typescript/`)

- Similar exposure for the JS/TS ecosystem.

## 9. Implementation Steps & Checklist

### Step 1: Grammar & AST

- [ ] Modify `sea.pest` to add `import` and `export`.
- [ ] Update `ast.rs` with `ImportDecl` and `AstNode::Export`.
- [ ] Update `parser/mod.rs` to handle the new nodes.
- [ ] Add unit tests in `parser_tests.rs`.

### Step 2: Module Resolution Logic

- [ ] Create `sea-core/src/module/` directory.
- [ ] Implement `resolver.rs` to handle import resolution against `NamespaceRegistry`.
- [ ] Implement cycle detection.
- [ ] Add tests for resolution logic.

### Step 3: Integration with Parser

- [ ] Update `parse_to_graph` to use the resolver.
- [ ] Implement the "load world" logic: when parsing a file, if it has imports, load them (or verify they exist).
- [ ] *Note*: For the first pass, we might just validate that imports *can* be resolved, without fully merging the graphs, or we might implement full multi-file graph merging.
- [ ] **Decision**: `parse_to_graph` should probably return a Graph containing *all* loaded dependencies merged, or we need a `Project` abstraction. For now, merging into a single Graph is the standard SEA pattern.

### Step 4: CLI & Error Reporting

- [ ] Add `ImportError` variants.
- [ ] Update `sea validate` to handle imports.
- [ ] Add `sea module` subcommand.

### Step 5: Bindings & Final Polish

- [ ] Update Python/TS bindings.
- [ ] Write documentation and examples.

## 10. Testing Strategy

1. **Unit Tests**:
    - `sea-core/tests/import_parsing_tests.rs`: Test grammar.
    - `sea-core/tests/module_resolution_tests.rs`: Test resolver logic (mocked registry).

2. **Integration Tests**:
    - Create a `tests/modules/` directory with real `.sea` files.
    - Test Case: `basic_import.sea` imports `utils.sea`.
    - Test Case: `cycle_a.sea` imports `cycle_b.sea` (should fail).
    - Test Case: `private_access.sea` tries to import non-exported symbol (should fail).

3. **Cross-Language**:
    - Verify Python can inspect imports.
