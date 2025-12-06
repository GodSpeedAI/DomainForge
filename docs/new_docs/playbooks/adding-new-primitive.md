# Adding a New Primitive

This playbook outlines the steps required to add a new primitive (e.g., `Zone`, `Cluster`) to the DomainForge language.

**Warning**: This is a cross-cutting change that affects Core, Grammar, and all Bindings.

## Checklist

### 1. Rust Core (`sea-core`)

- [ ] **Grammar**: Update `sea-core/grammar/sea.pest`. Add the new keyword and syntax rules.
- [ ] **AST**: Update `sea-core/src/parser/ast.rs` to include the new node type.
- [ ] **Primitive Struct**: Create `sea-core/src/primitives/new_primitive.rs`. Implement `Debug`, `Clone`, `Serialize`.
- [ ] **Graph**: Update `sea-core/src/graph/mod.rs`. Add an `IndexMap` for the new primitive.
- [ ] **Parser Logic**: Update `sea-core/src/parser/mod.rs` to transform the AST node into the Primitive struct.

### 2. Bindings

- [ ] **Python**: Update `sea-core/src/python/`. Create a wrapper struct `#[pyclass]`. Register it in `lib.rs`.
- [ ] **TypeScript**: Update `sea-core/src/typescript/`. Create the napi-compatible struct.
- [ ] **WASM**: Update `sea-core/src/wasm/` if the primitive needs specific exposure.

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
