# Import from SBVR

This guide explains how to import SBVR (Semantics of Business Vocabulary and Rules) XMI files into SEA graphs.

## Overview

SBVR is an OMG standard for expressing business vocabularies and rules in natural language with formal semantics. DomainForge can import SBVR XMI files and convert them to SEA primitives:

| SBVR Concept  | SEA Mapping         |
| ------------- | ------------------- |
| Noun Concept  | Entity              |
| Verb Concept  | Relation            |
| Fact Type     | Role-based Relation |
| Business Rule | Policy              |
| Definition    | Entity description  |

## Prerequisites

- `sea` CLI installed with `cli` feature
- An SBVR XMI file (e.g., exported from business rule tools)

## Quick Start

```bash
# Import SBVR XMI to SEA DSL
sea import --format sbvr vocabulary.xmi > model.sea

# Import with custom namespace
sea import --format sbvr --namespace my-domain vocabulary.xmi
```

## Example

### Input: SBVR XMI

```xml
<?xml version="1.0" encoding="UTF-8"?>
<sbvr:Vocabulary xmlns:sbvr="http://www.omg.org/spec/SBVR">
  <nounConcept xmi:id="nc1" name="Customer">
    <definition>A person or organization that purchases products</definition>
  </nounConcept>

  <nounConcept xmi:id="nc2" name="Order">
    <definition>A request to purchase products</definition>
  </nounConcept>

  <verbConcept xmi:id="vc1" name="places">
    <role name="placer" nounConceptRef="nc1"/>
    <role name="placed" nounConceptRef="nc2"/>
  </verbConcept>

  <factType xmi:id="ft1" reading="Customer places Order" verbConceptRef="vc1"/>

  <businessRule xmi:id="br1" name="OrderLimit">
    <statement>Each Customer must have at most 10 pending Orders</statement>
    <ruleType>OperativeRule</ruleType>
  </businessRule>
</sbvr:Vocabulary>
```

### Output: SEA DSL

```bash
sea import --format sbvr vocabulary.xmi
```

Produces:

```sea
Namespace "sbvr"

// Entities from Noun Concepts
Entity "Customer" {
    @description "A person or organization that purchases products"
}

Entity "Order" {
    @description "A request to purchase products"
}

// Roles from Verb Concepts
Role "placer"
Role "placed"

// Relations from Fact Types
Relation "places" from placer to placed {
    @reading "Customer places Order"
}

// Policies from Business Rules
Policy "OrderLimit" {
    severity: error
    expression: "Each Customer must have at most 10 pending Orders"
    @original_sbvr true
}
```

## SBVR Concept Mapping

### Noun Concepts → Entities

SBVR noun concepts become SEA entities. Concept hierarchies are preserved via the `@generalizes` attribute:

```xml
<nounConcept xmi:id="nc1" name="Person"/>
<nounConcept xmi:id="nc2" name="Customer" generalConcept="nc1"/>
```

Becomes:

```sea
Entity "Person"
Entity "Customer" {
    @generalizes "Person"
}
```

### Verb Concepts → Roles and Relations

Binary verb concepts become role definitions and relations:

```xml
<verbConcept name="manages">
    <role name="manager" nounConceptRef="Person"/>
    <role name="managed" nounConceptRef="Team"/>
</verbConcept>
```

Becomes:

```sea
Role "manager"
Role "managed"
Relation "manages" from manager to managed
```

### Business Rules → Policies

Business rules are converted based on their type:

| SBVR Rule Type | SEA Severity |
| -------------- | ------------ |
| StructuralRule | error        |
| OperativeRule  | warning      |
| DerivationRule | info         |

### Supported Statement Patterns

| SBVR Pattern               | SEA Expression             |
| -------------------------- | -------------------------- |
| "Each X must Y"            | `forall x in X: Y(x)`      |
| "At most one X per Y"      | Cardinality constraint     |
| "It is necessary that..."  | Policy with error severity |
| "It is obligatory that..." | Policy with error severity |
| "It is permitted that..."  | No policy (allowed)        |

## Programmatic Usage

### Rust

```rust
use sea_core::sbvr::SbvrModel;

let xmi = std::fs::read_to_string("vocabulary.xmi")?;
let sbvr_model = SbvrModel::from_xmi(&xmi)?;
let graph = sbvr_model.to_graph()?;

println!("Imported {} entities", graph.all_entities().count());
println!("Imported {} policies", graph.all_policies().count());
```

### Python

```python
from sea_dsl import Graph

# Import SBVR via CLI subprocess
import subprocess
result = subprocess.run(
    ["sea", "import", "--format", "sbvr", "vocabulary.xmi"],
    capture_output=True,
    text=True
)
sea_dsl = result.stdout

# Parse the resulting SEA DSL
graph = Graph.parse(sea_dsl)
print(f"Imported {len(graph.entities())} entities")
```

## Limitations

| Limitation           | Workaround                           |
| -------------------- | ------------------------------------ |
| Complex quantifiers  | Manual policy authoring after import |
| Computed derivations | Not supported (logged as warning)    |
| Temporal rules       | Limited support                      |
| Modal logic          | Mapped to severity levels            |

Some SBVR statements cannot be automatically converted to SEA expressions. These are imported with `@original_sbvr true` attribute for manual review.

## Post-Import Workflow

1. **Review imported policies**: Check for `@original_sbvr` markers indicating statements that need manual conversion
2. **Validate the model**: Run `sea validate model.sea` to check for semantic errors
3. **Refine expressions**: Convert natural language statements to formal SEA expressions
4. **Add missing details**: SBVR may not capture all attributes needed for your domain

```bash
# Validate imported model
sea validate model.sea

# Export to other formats
sea project --format calm model.sea architecture.json
sea project --format protobuf model.sea contracts.proto
```

## Troubleshooting

### Parse Error: Invalid XMI

Ensure your SBVR file is valid XMI format:

- Must have proper XML declaration
- Must use SBVR namespace
- Element names must match expected tags

### Missing Noun Concepts

If relations reference undefined noun concepts:

1. Check the XMI for `nounConceptRef` values
2. Ensure all referenced concepts are defined
3. Use `--allow-unknown` flag to import partial models

### Unsupported Rule Type

Some SBVR rule patterns are not automatically converted:

- The import will succeed with a warning
- Review the `@original_sbvr` policies manually
- Convert to formal SEA expressions as needed

## See Also

- [SDS-007: SBVR Import](../specs/SDS-007-sbvr-import.md) - System design spec
- [Define Policies](./define-policies.md) - Writing SEA policies
- [CLI Commands](../reference/cli-commands.md) - Import command reference
- [Grammar Spec](../reference/grammar-spec.md) - SEA syntax reference
