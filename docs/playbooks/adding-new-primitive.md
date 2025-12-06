# Adding a New Primitive

This playbook outlines the steps required to add a new primitive (e.g., `Zone`, `Cluster`) to the DomainForge language.

**Warning**: This is a cross-cutting change that affects Core, Grammar, and all Bindings.

## Checklist

### 1. Rust Core (`sea-core`)

- [ ] **Grammar**: Update `sea-core/grammar/sea.pest`. Add the new keyword and syntax rules.
- [ ] **AST**: Update `sea-core/src/parser/ast.rs` to include the new node type.
- [ ] **Primitive Struct**: Create `sea-core/src/primitives/new_primitive.rs`. Implement `Debug`, `Clone`, `Serialize`.
- [ ] **Module Exports**: Add the new module to `sea-core/src/primitives/mod.rs` and export it (`pub mod new_primitive;` and `pub use` if needed).
- [ ] **Graph**: Update `sea-core/src/graph/mod.rs`. Add an `IndexMap` for the new primitive, expose it with `pub mod`/`pub use`, and wire the new collection into Graph.
- [ ] **Parser Logic**: Update `sea-core/src/parser/mod.rs` to transform the AST node into the Primitive struct, add the module to the `mod` list, and export it.

### 2. Bindings

- [ ] **Python**: Update `sea-core/src/python/lib.rs` and `sea-core/src/python/primitives.rs` to add the `#[pyclass]` wrapper and register it in `lib.rs`.
- [ ] **TypeScript**: Update `sea-core/src/typescript/primitives.rs` and `sea-core/src/typescript/index.ts` to add the napi-compatible struct and export it.
- [ ] **WASM**: Update `sea-core/src/wasm/primitives.rs` and `sea-core/src/wasm/lib.rs` if the primitive needs specific exposure.

### 3. Testing

- [ ] **Rust Tests**: Add a test case in `sea-core/tests/parser_tests.rs`.
- [ ] **Python Tests**: Add a test in `tests/test_primitives.py`.
- [ ] **TypeScript Tests**: Add a test in `typescript-tests/primitives.test.ts`.

### 4. Documentation

- [ ] Update `semantic-modeling-concepts.md`.
- [ ] Update `sea.pest` comments.

## Example: Adding "Zone"

1. **Grammar**:

   ```pest
   zone_decl = { "zone" ~ identifier ~ "{" ~ (property)* ~ "}" }
   ```

2. **Struct**:

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Zone {
       pub id: ConceptId,
       pub properties: IndexMap<String, Value>,
   }
   ```

3. **Graph**:

   ```rust
      pub struct Graph {
         pub zones: IndexMap<ConceptId, Zone>,
         // ... other primitive collections (entities, resources, flows, roles, relations, instances)
      }
   ```

## Verification

Run the full suite to ensure no regressions:

```bash
just all-tests
```
