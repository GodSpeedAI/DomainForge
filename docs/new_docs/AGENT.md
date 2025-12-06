# DomainForge Documentation Generation Prompt

## Objective

Populate all placeholder files in `docs/new_docs/` with production-ready documentation for the DomainForge project. The documentation must be accurate, comprehensive, and follow the Diataxis framework conventions already established in each folder's README.

## Source Materials (Priority Order)

### Primary Sources (Internal)

1. **Existing documentation** — `docs/` folder:
   - `docs/plans/` — Implementation plans, roadmaps, feature specs
   - `docs/specs/` — Error codes, technical specifications
   - `docs/reference/` — Existing reference material (especially `calm-mapping.md`)
   - `docs/guides/` — Any existing guides or tutorials

2. **Code as documentation**:
   - `sea-core/grammar/sea.pest` — Authoritative grammar specification
   - `sea-core/src/primitives/` — Entity, Resource, Flow, Instance, Policy definitions
   - `sea-core/src/policy/expression.rs` — Policy expression evaluation
   - `sea-core/src/calm/` — CALM export/import implementation
   - `sea-core/src/kg.rs`, `sea-core/src/kg_import.rs` — RDF/Turtle knowledge graph
   - `sea-core/src/python/`, `sea-core/src/typescript/`, `sea-core/src/wasm/` — Binding implementations
   - `sea-core/src/validation_error.rs` — Error code definitions

3. **Project configuration**:
   - `.github/copilot-instructions.md` — Architecture overview, conventions, workflows
   - `README.md` (root) — Project overview, installation, quick start
   - `justfile` — Developer commands and workflows
   - `Cargo.toml`, `pyproject.toml`, `package.json` — Dependencies and build config

4. **Examples**:
   - `examples/` — DSL example files
   - `sea-core/examples/` — Rust usage examples
   - `sea-core/tests/` — Test files as usage documentation (60+ test files)

5. **Test files** (for API behavior):
   - `tests/test_*.py` — Python binding usage patterns
   - `typescript-tests/*.test.ts` — TypeScript binding usage patterns

### External Sources

1. **FINOS CALM (Architecture-as-Code)**:
   - Repository: https://github.com/finos/architecture-as-code
   - Specification: https://github.com/finos/architecture-as-code/tree/main/calm
   - Use for: CALM mapping reference, export/import how-tos, architecture concepts

2. **Pest Parser**:
   - Documentation: https://pest.rs/book/
   - Use for: Grammar specification reference, extending grammar how-to

3. **Binding libraries**:
   - PyO3: https://pyo3.rs/
   - napi-rs: https://napi.rs/
   - wasm-bindgen: https://rustwasm.github.io/wasm-bindgen/
   - Use for: Cross-language binding explanations and API references

## Files to Populate

### Tutorials (`docs/new_docs/tutorials/`)

| File | Content Requirements |
|------|---------------------|
| `getting-started.md` | End-to-end first experience: install CLI → write first .sea file → parse → view output. Target: 15-minute completion. Include expected output at each step. |
| `first-sea-model.md` | Build a complete domain model (e.g., e-commerce with Customer, Order, Product entities). Cover: entities, resources, flows, instances. Include full .sea file and explain each section. |
| `python-binding-quickstart.md` | Install via pip → parse .sea file → access entities programmatically → modify and re-export. Include pytest example. |
| `typescript-binding-quickstart.md` | Install via npm → parse .sea file → access entities → integrate with Node.js project. Include Vitest example. |
| `wasm-in-browser.md` | Load WASM in browser → parse .sea content from textarea → render results. Include complete HTML+JS example. |

### How-Tos (`docs/new_docs/how-tos/`)

| File | Content Requirements |
|------|---------------------|
| `install-cli.md` | All installation methods: cargo install, GitHub releases (Linux/macOS/Windows), from source. Include verification steps. |
| `parse-sea-files.md` | CLI parsing, programmatic parsing (Rust/Python/TS), handling parse errors, batch processing multiple files. |
| `export-to-calm.md` | Step-by-step CALM JSON export. Cover: CLI command, programmatic API, customizing output, validating against CALM schema. Reference FINOS CALM spec. |
| `import-from-calm.md` | Import existing CALM JSON into SEA format. Cover: CLI, API, handling import warnings, round-trip validation. |
| `generate-rdf-turtle.md` | Generate RDF/Turtle output for knowledge graph integration. Cover: CLI, API, namespace configuration, integration with triple stores. |
| `define-policies.md` | Write policy expressions: syntax, operators, three-valued logic, quantifiers (forall, exists). Include practical examples (access control, validation rules). |
| `create-custom-units.md` | Define custom units of measure. Cover: built-in units, unit syntax, unit conversion, using units in resources. |
| `run-cross-language-tests.md` | Run all test suites: `just all-tests`, individual suites, CI configuration, debugging test failures. |
| `extend-grammar.md` | Add new syntax to SEA: modify sea.pest → update AST → add parser tests → update projections. Include concrete example (e.g., adding a new keyword). |

### Reference (`docs/new_docs/reference/`)

| File | Content Requirements |
|------|---------------------|
| `grammar-spec.md` | Complete grammar reference. Parse and document every rule in `sea.pest`. Include EBNF-style notation, examples for each construct, and edge cases. |
| `primitives-api.md` | Document all primitives: Entity, Resource, Flow, Instance, Policy. For each: fields, constructors, methods, relationships, serialization format. |
| `cli-commands.md` | All CLI commands with flags, options, examples, exit codes. Generate from actual CLI help output if available. |
| `python-api.md` | Complete Python API reference. Document every exposed class/function from PyO3 bindings. Include type hints, examples, exceptions. |
| `typescript-api.md` | Complete TypeScript API reference. Document every exposed type/function from napi-rs bindings. Include TypeScript types, examples, error handling. |
| `wasm-api.md` | WASM API reference. Document exposed functions, memory considerations, browser vs Node.js usage differences. |
| `error-codes.md` | All error codes from `validation_error.rs` and `docs/specs/error_codes.md`. Include: code, message, cause, resolution. |
| `calm-mapping.md` | Detailed mapping between SEA primitives and CALM JSON. Reference existing `docs/reference/specs/calm-mapping.md`. Include bidirectional mapping tables. |
| `configuration.md` | All configuration options: environment variables, config files, CLI flags, programmatic configuration. |

### Explanations (`docs/new_docs/explanations/`)

| File | Content Requirements |
|------|---------------------|
| `architecture-overview.md` | System architecture: Rust core as canonical, binding strategy, why this approach. Include architecture diagram (ASCII or Mermaid). Reference `.github/copilot-instructions.md`. |
| `semantic-modeling-concepts.md` | Explain domain modeling with SEA: what is semantic modeling, entities vs resources vs flows, when to use each primitive, modeling patterns. |
| `policy-evaluation-logic.md` | How policies are evaluated: expression parsing, evaluation order, variable binding, side effects. Include evaluation flow diagram. |
| `three-valued-logic.md` | Explain Kleene three-valued logic: True/False/Unknown, truth tables, why it's needed, practical implications for policy authors. |
| `graph-store-design.md` | Internal graph representation: why IndexMap, node/edge model, query patterns, performance characteristics. |
| `cross-language-binding-strategy.md` | Why and how bindings work: PyO3/napi-rs/wasm-bindgen, what's exposed, memory management, keeping bindings in sync. |
| `versioning-strategy.md` | Semantic versioning approach, breaking change policy, migration paths, deprecation process. |

### Playbooks (`docs/new_docs/playbooks/`)

| File | Content Requirements |
|------|---------------------|
| `adding-new-primitive.md` | Step-by-step: add to Rust core → update all bindings → add tests → update grammar if needed → update docs. Include checklist. |
| `releasing-beta.md` | Release process: version bump, changelog, CI checks, crates.io/PyPI/npm publishing, GitHub release. Reference `docs/plans/release_beta_plan.md`. |
| `debugging-parser-failures.md` | Diagnose parse errors: read error messages, common syntax mistakes, using pest debugger, reporting grammar bugs. |
| `migrating-schema-versions.md` | Handle breaking changes: detect version, migration scripts, backward compatibility strategies. |
| `onboarding-contributors.md` | New contributor guide: dev environment setup, codebase tour, first issue, PR process, testing requirements. |

## Quality Requirements

### Style

- Use active voice and direct language
- Code examples must be complete and runnable
- Include expected output for all examples
- Use consistent heading hierarchy (H1 = title, H2 = sections, H3 = subsections)
- No trailing whitespace, proper blank lines around lists and code blocks

### Accuracy

- All code examples must compile/run against current codebase
- API signatures must match actual implementation
- CLI commands must match actual CLI behavior
- Version numbers must be current

### Completeness

- Every placeholder file must have substantial content (minimum 200 lines for reference docs, 100 lines for how-tos)
- All cross-references must link to existing files
- Include "See also" sections linking related docs across categories

### Format

Follow the templates in `docs/new_docs/templates/`:

- `template_tutorial.md`
- `template_howto.md`
- `template_reference.md`
- `template_explanation.md`
- `template_playbook.md`

## Verification Steps

After generating documentation:

1. Run `just all-tests` to ensure code examples are valid
2. Verify all internal links resolve
3. Check that CLI commands match `--help` output
4. Validate CALM examples against FINOS schema
5. Review grammar-spec.md against actual `sea.pest`

## Output Format

For each file, provide the complete content ready to save. Use this format:

```markdown
<!-- filepath: docs/new_docs/{category}/{filename}.md -->
# Title

[Complete content here]
```

Process files in this order:

1. Reference docs (establishes terminology)
2. Explanations (provides context)
3. How-tos (practical tasks)
4. Tutorials (guided learning)
5. Playbooks (operational procedures)
