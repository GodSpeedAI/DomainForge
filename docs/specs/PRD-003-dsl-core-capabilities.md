# PRD-003: DSL Core Capabilities

**Product:** SEA Domain-Specific Language  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Goals

- Define the complete vocabulary of SEA-DSL
- Enable business-readable, machine-executable models
- Support enterprise domain modeling patterns
- Provide a stable foundation for all projections and bindings

---

## 2. Non-Goals

- General-purpose programming language features
- Runtime execution environment
- Database or persistence layer
- UI/visualization components

---

## 3. Core Primitives

### 3.1 Entity

**Purpose:** Represent actors, locations, or organizational units

```sea
Entity "Warehouse" in logistics {
    id: UUID
    name: String
    capacity: Int
    location: Address
}
```

**Attributes:**

- `name`: Human-readable identifier (required)
- `namespace`: Logical grouping (optional, defaults to file namespace)
- `attributes`: Typed fields with optional patterns
- `relations`: Links to other entities

### 3.2 Resource

**Purpose:** Represent quantifiable items that flow between entities

```sea
Resource "Camera Units" units in inventory
Resource "Payment" USD in finance
```

**Attributes:**

- `name`: Resource identifier (required)
- `unit`: Unit of measurement (required)
- `namespace`: Logical grouping (optional)

### 3.3 Flow

**Purpose:** Represent movement of resources between entities

```sea
Flow "Camera Units" from "Assembly Line" to "Warehouse" quantity 100
```

**Attributes:**

- `resource`: Reference to Resource (required)
- `from`: Source Entity (required)
- `to`: Destination Entity (required)
- `quantity`: Numeric amount (required)

### 3.4 Instance

**Purpose:** Represent concrete, trackable instantiations of entities

```sea
Instance "Camera #SN12345" of "Camera" at "Warehouse A" {
    serial_number: "SN12345"
    manufactured_date: 2024-01-15
    status: "available"
}
```

**Attributes:**

- `name`: Instance identifier (required)
- `entity_type`: Reference to Entity type (required)
- `location`: Current location Entity (optional)
- `fields`: Concrete values for Entity attributes

### 3.5 Role

**Purpose:** Define capabilities that can be assigned to entities

```sea
Role "Supplier" in supply-chain {
    provides: Resource
}

Role "Consumer" in supply-chain {
    consumes: Resource
}
```

### 3.6 Relation

**Purpose:** Define typed relationships between entities via roles

```sea
Relation "supplies" {
    subject: Supplier
    object: Consumer
    predicate: "supplies"
    via: "Shipment"
}
```

### 3.7 Policy

**Purpose:** Express business rules and constraints

```sea
Policy production_minimum as:
    forall f in Flow where f.resource = "Camera Units":
        f.quantity >= 500

Policy inventory_limit severity warn as:
    forall e in Entity where e.type = "Warehouse":
        sum(Instance.quantity where Instance.at = e) <= e.capacity
```

**Attributes:**

- `name`: Policy identifier (required)
- `expression`: Logical expression (required)
- `severity`: error | warn | info (default: error)

### 3.8 Metric

**Purpose:** Define observable measurements with thresholds

```sea
Metric "Daily Throughput" on Flow where resource = "Camera Units" {
    aggregate: sum(quantity)
    warn_above: 1000
    error_above: 2000
}
```

### 3.9 Pattern

**Purpose:** Define reusable validation patterns

```sea
Pattern "email" as: /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/
Pattern "phone" as: /^\+?[1-9]\d{1,14}$/
```

### 3.10 ConceptChange

**Purpose:** Model temporal evolution of concepts

```sea
ConceptChange "Product Rename" {
    from: "Widget"
    to: "Gadget"
    effective: 2024-06-01
    reason: "Rebranding initiative"
}
```

---

## 4. Expression Language

### 4.1 Operators

| Category   | Operators                       |
| ---------- | ------------------------------- |
| Comparison | `=`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical    | `and`, `or`, `not`              |
| Arithmetic | `+`, `-`, `*`, `/`              |
| Membership | `in`                            |

### 4.2 Quantifiers

| Quantifier                 | Meaning                                            |
| -------------------------- | -------------------------------------------------- |
| `forall x in Domain: expr` | Universal: true if expr holds for all x            |
| `exists x in Domain: expr` | Existential: true if expr holds for at least one x |

### 4.3 Aggregations

| Function      | Description    |
| ------------- | -------------- |
| `sum(expr)`   | Sum of values  |
| `count(expr)` | Count of items |
| `min(expr)`   | Minimum value  |
| `max(expr)`   | Maximum value  |
| `avg(expr)`   | Average value  |

---

## 5. Type System

### 5.1 Built-in Types

| Type       | Description                       |
| ---------- | --------------------------------- |
| `String`   | Text value                        |
| `Int`      | Integer                           |
| `Float`    | Decimal number                    |
| `Bool`     | Boolean                           |
| `Date`     | Date (YYYY-MM-DD)                 |
| `DateTime` | Date with time                    |
| `UUID`     | Unique identifier                 |
| `Email`    | Email address (pattern validated) |
| `URL`      | Web URL                           |
| `Money`    | Currency amount                   |

### 5.2 Custom Types

Defined via Entity attributes with patterns:

```sea
Entity "Customer" {
    phone: String pattern: phone
    email: String pattern: email
}
```

---

## 6. Profiles

Restrict allowed constructs for specific use cases:

```sea
@profile "basic"

// Only Entity, Resource, Flow allowed in this file
```

| Profile    | Allowed Constructs               |
| ---------- | -------------------------------- |
| `basic`    | Entity, Resource, Flow           |
| `standard` | basic + Policy, Metric, Instance |
| `full`     | All constructs                   |

---

## 7. Quality Attributes

| Attribute         | Requirement                              |
| ----------------- | ---------------------------------------- |
| **Readability**   | Business analysts can understand models  |
| **Precision**     | Unambiguous semantics for all constructs |
| **Extensibility** | New primitives can be added              |
| **Performance**   | 10,000 entities validated in < 100ms     |

---

## Related Documents

- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
- [SDS-003: Parser and Semantic Graph](./SDS-003-parser-semantic-graph.md)
- [ADR-007: Policy Evaluation Engine](./ADR-007-policy-evaluation-engine.md)
