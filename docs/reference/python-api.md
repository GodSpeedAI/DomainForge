# SEA DSL - Python Bindings

Python bindings for the Semantic Enterprise Architecture (SEA) Domain Specific Language.

## Installation

```bash
# From source (PyPI package coming soon)
pip install maturin
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
maturin develop

# Or build wheel
maturin build --release
pip install target/wheels/sea_dsl-*.whl
```

## Quick Start

### Creating Primitives

```python
import sea_dsl

# Create entities - use new() for default namespace, new_with_namespace() for explicit
warehouse = sea_dsl.Entity.new("Warehouse")  # Default namespace
factory = sea_dsl.Entity.new_with_namespace("Factory", "manufacturing")

# Namespace is always a string (not None), defaults to "default"
print(warehouse.namespace())  # "default"
print(factory.namespace())    # "manufacturing"

# Create resources
cameras = sea_dsl.Resource.new("Cameras", "units")

# Create flows - pass ConceptId values (clone before passing)
flow = sea_dsl.Flow.new(
    cameras.id().clone(),
    warehouse.id().clone(),
    factory.id().clone(),
    100.0
)
```

### Building a Graph

```python
import sea_dsl
from decimal import Decimal

# Create and populate a graph
graph = sea_dsl.Graph()

# Entities with constructor patterns
warehouse = sea_dsl.Entity.new("Warehouse")
factory = sea_dsl.Entity.new_with_namespace("Factory", "manufacturing")

# Resources with units
cameras = sea_dsl.Resource.new("Cameras", "units")

graph.add_entity(warehouse)
graph.add_entity(factory)
graph.add_resource(cameras)

# Flow with ConceptId clones and Decimal quantity
flow = sea_dsl.Flow.new(
    cameras.id().clone(),
    warehouse.id().clone(),
    factory.id().clone(),
    Decimal("100.0")
)
graph.add_flow(flow)

print(f"Graph has {graph.entity_count()} entities")
print(f"Graph has {graph.flow_count()} flows")
```

### Parsing DSL Source

```python
import sea_dsl

# Supports multiline strings with """ syntax
source = '''
    Entity "Warehouse" in logistics
    Entity """Multi-line
    Factory Name""" in manufacturing
    Resource "Cameras" units
    Flow "Cameras" from "Warehouse" to "Multi-line\nFactory Name" quantity 100
'''

graph = sea_dsl.Graph.parse(source)
print(f"Parsed {graph.entity_count()} entities")
print(f"Parsed {graph.flow_count()} flows")

# Query the graph
warehouse_id = graph.find_entity_by_name("Warehouse")
flows = graph.flows_from(warehouse_id)
for flow in flows:
    print(f"Flow: {flow.quantity()} units")
```

### Working with Attributes

```python
import sea_dsl

# Use new() for default namespace
entity = sea_dsl.Entity.new("Warehouse")
entity.set_attribute("capacity", 10000)
entity.set_attribute("location", "New York")

print(entity.get_attribute("capacity"))  # 10000
print(entity.get_attribute("location"))  # "New York"

# Namespace is always present (not None)
print(entity.namespace())  # "default"
```

### Formatting Source Code

```python
import sea_dsl

# Format SEA-DSL source with default settings
source = '''Entity   "Foo"  in    bar'''
formatted = sea_dsl.format_source(source)
print(formatted)  # Entity "Foo" in bar

# Format with custom options
formatted = sea_dsl.format_source(
    source,
    indent_width=2,
    use_tabs=False,
    preserve_comments=True,
    sort_imports=True
)

# Check if source is already formatted
is_formatted = sea_dsl.check_format(source)
print(is_formatted)  # False

# Check with formatted content
is_formatted = sea_dsl.check_format('Entity "Foo" in bar\n')
print(is_formatted)  # True
```

## API Reference

### Formatter Functions

- `format_source(source, indent_width=4, use_tabs=False, preserve_comments=True, sort_imports=True)`: Format SEA-DSL source code
- `check_format(source, indent_width=4, use_tabs=False)`: Check if source is already formatted

### Classes

- `Entity`: Represents business entities (WHO)
- `Resource`: Represents quantifiable resources (WHAT)
- `Flow`: Represents resource movement between entities
- `Instance`: Represents physical instances of resources
- `Graph`: Container with validation and query capabilities (uses IndexMap for deterministic iteration)
- `Expression`: Policy expression AST wrapper with normalization support
- `NormalizedExpression`: Canonical form of an expression with stable hashing

### Constructor Patterns (November 2025)

**Entities:**

```python
# Default namespace
entity = Entity.new("Warehouse")  # namespace() returns "default"

# Explicit namespace
entity = Entity.new_with_namespace("Warehouse", "logistics")
```

**Resources:**

```python
resource = Resource.new("Cameras", "units")  # Default namespace
resource = Resource.new_with_namespace("Cameras", "units", "inventory")
```

**Flows:**

```python
# Takes ConceptId values (not references) - clone before passing
flow = Flow.new(
    resource_id.clone(),
    from_id.clone(),
    to_id.clone(),
    Decimal("100.0")
)
```

### Graph Methods

- `add_entity(entity)`: Add an entity to the graph
- `add_resource(resource)`: Add a resource to the graph
- `add_flow(flow)`: Add a flow to the graph (validates references)
- `add_instance(instance)`: Add an instance to the graph
- `entity_count()`: Get number of entities
- `resource_count()`: Get number of resources
- `flow_count()`: Get number of flows
- `instance_count()`: Get number of instances
- `find_entity_by_name(name)`: Find entity ID by name
- `find_resource_by_name(name)`: Find resource ID by name
- `flows_from(entity_id)`: Get all flows from an entity
- `flows_to(entity_id)`: Get all flows to an entity
- `all_entities()`: Get all entities
- `all_resources()`: Get all resources
- `all_flows()`: Get all flows
- `all_instances()`: Get all instances
- `Graph.parse(source)`: Parse DSL source into a graph
- `Graph.parse_to_ast_json(source)`: Parse DSL source into AST JSON string
- `export_calm()`: Export graph to CALM JSON format
- `Graph.import_calm(json_str)`: Import graph from CALM JSON
- `add_policy(policy)`: Add a policy to the graph
- `add_association(owner_id, owned_id, rel_type)`: Add ownership/association relation between two entities (owner/owned)

### Expression API (December 2025)

```python
from sea_dsl import Expression, BinaryOp

# Factory methods
expr = Expression.binary(
    BinaryOp.And,
    Expression.variable("b"),
    Expression.variable("a")
)

# Normalization
normalized = expr.normalize()
print(str(normalized))  # "(a AND b)" (commutative sorting)
print(normalized.stable_hash())  # Stable hash for caching
```

**Breaking Changes:**

### NamespaceRegistry (Workspace)

```python
import sea_dsl

reg = sea_dsl.NamespaceRegistry.from_file('./.sea-registry.toml')
files = reg.resolve_files()
for binding in files:
    print(binding.path, '=>', binding.namespace)

ns = reg.namespace_for('/path/to/file.sea')
print('Namespace:', ns)

# You can also pass `True` as an optional second argument to make resolution fail on ambiguity:
try:
    reg.namespace_for(str('/path/to/file.sea'), True)
except Exception as e:
    print('Ambiguity detected:', e)
```

- `namespace()` now returns `str` instead of `Optional[str]` (always returns "default" if unspecified)
- Constructors split: `new()` for default namespace, `new_with_namespace()` for explicit
- `Resource.new(name, unit)` now routes through `new_with_namespace(..., "default")` so `namespace()` never returns `None` even when a namespace is not supplied
- Flow constructor takes `ConceptId` values (not references) - must clone before passing

- Multiline string support in parser: `Entity """Multi-line\nName"""`
- ValidationError helpers: `undefined_entity()`, `unit_mismatch()`, etc.
- CALM integration: `export_calm()` and `import_calm()` for architecture-as-code
- IndexMap storage ensures deterministic iteration (reproducible results)

## Semantic Pack API

The `sea_dsl` module exposes semantic pack types and functions for building, validating, signing, and inspecting packs programmatically.

### Enums

```python
from sea_dsl import (
    SemanticTruth,        # Valid, Invalid, Unknown
    DiagnosticSeverity,   # Error, Warning, Info, Hint
    ValidationMode,       # Off, Warn, Strict
    ApprovalState,        # Candidate, Approved, Rejected
    SignatureState,       # Unsigned, Signed, InvalidSignature
    ConceptStatus,        # Active, Proposed, Deprecated, Rejected, ExternalOnly
    ConceptKind,          # Entity, Resource, Role, Flow, Policy, Metric, Dimension, Unit, External
    AliasStatus,          # Approved, Deprecated, Ambiguous, Blocked
    UnknownConceptPolicy, # Ignore, Warning, Error
    DeprecatedPolicy,     # Allow, Warn, ErrorInStrict, ErrorAlways
    SemanticValidationStatus,  # Passed, Failed, Unknown, Blocked
)
```

### SemanticPack Class

```python
pack = sea_dsl.SemanticPack.from_json(pack_json_string)
print(pack.pack_id())              # "acme/logistics/1.1.0"
print(pack.schema_version())       # "0.3"
print(pack.approval_state())       # ApprovalState.Approved
print(pack.signature_state())      # SignatureState.Signed
print(pack.concept_count())        # 42
print(pack.alias_count())          # 15
print(pack.meaning_version())      # "1.1.0"
print(pack.meaning_fingerprint())  # "sha256:abc..."
print(pack.pack_content_hash())    # "sha256:def..."

json_str = pack.to_json()
```

Methods:

| Method                | Return Type         | Description                            |
|-----------------------|---------------------|----------------------------------------|
| `from_json(json)`     | `SemanticPack`      | Deserialize from JSON string.          |
| `to_json()`           | `str`               | Serialize to JSON string.              |
| `pack_id()`           | `str`               | Pack identifier.                       |
| `schema_version()`    | `str`               | Schema version.                        |
| `approval_state()`    | `ApprovalState`     | Pack approval state.                   |
| `signature_state()`   | `SignatureState`    | Pack signature state.                  |
| `concept_count()`     | `int`               | Number of concepts.                    |
| `alias_count()`       | `int`               | Number of aliases.                     |
| `meaning_version()`   | `str`               | Meaning version.                       |
| `meaning_fingerprint()` | `str`             | Meaning fingerprint hash.              |
| `pack_content_hash()` | `str`               | Content hash (excluding signature).    |

### SemanticValidationResult Class

```python
result = sea_dsl.SemanticValidationResult.from_json(result_json)
print(result.status())                        # SemanticValidationStatus.Passed
print(result.diagnostics_json())              # JSON array of diagnostics
print(result.unsigned_fixture_bypass_used())  # False
print(result.first_approved_version_bypass_used())  # False
```

### Functions

#### build_semantic_pack

Build a semantic pack from input JSON. Returns a tuple of `(pack_json, error_list)`.

```python
pack_json, errors = sea_dsl.build_semantic_pack(input_json)
if errors:
    for err in errors:
        print(err)
```

#### validate_semantic_pack

Validate a pack's internal consistency. Returns a list of diagnostic JSON strings.

```python
diagnostics = sea_dsl.validate_semantic_pack(pack_json)
for d in diagnostics:
    print(d)
```

#### validate_graph_with_pack

Validate a source file against a pack. Returns a JSON result string.

```python
result_json = sea_dsl.validate_graph_with_pack(
    pack_json,
    "source_uri",
    options_json
)
```

#### sign_pack

Sign a pack with an Ed25519 private key. Returns the signed pack JSON.

```python
signed_json = sea_dsl.sign_pack(pack_json, private_key_pem_string)
```

#### verify_pack_signature

Verify a pack's signature. Returns `True` or `False`.

```python
is_valid = sea_dsl.verify_pack_signature(pack_json, public_key_pem_string)
```

#### diff_packs

Compare two packs. Returns a JSON diff result.

```python
diff_json = sea_dsl.diff_packs(old_pack_json, new_pack_json)
```

#### compute_pack_hash

Compute the content hash of a pack.

```python
hash_str = sea_dsl.compute_pack_hash(pack_json)
```

#### normalize_lookup_key

Normalize a term for lookup (NFC, case-fold, whitespace collapse).

```python
normalized = sea_dsl.normalize_lookup_key("  Hello   World  ")  # "hello world"
```

#### resolve_concept

Resolve a term against a pack. Returns a JSON result with `resolved_concept_id`, `semantic_truth`, `diagnostic_code`, `message`, and `suggestions`.

```python
result_json = sea_dsl.resolve_concept(
    "Supplier",
    pack_json,
    options_json
)
```

## Development

### Building from Source

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Run tests
pytest
```

### Running Tests

```bash
pytest tests/
```

Quick start for tests in development (recommended):

```bash
# Requires just
just python-setup
just python-test
```

If you'd like to remove the local virtual environment and start fresh:

```bash
just python-clean
```

### Manual Python workflow (without just)

```bash
# Create a local virtual environment
python -m venv .venv

# Activate the environment
# Linux/macOS:
source .venv/bin/activate
# Windows (Command Prompt):
.\.venv\Scripts\activate
# Windows (PowerShell):
.\.venv\Scripts\Activate.ps1

# Install development dependencies
pip install -e .  # or `pip install -r requirements-dev.txt`

# Run the Python test suite
pytest tests/

# Clean up the virtual environment when you're done
deactivate
rm -rf .venv
Apache-2.0


```
