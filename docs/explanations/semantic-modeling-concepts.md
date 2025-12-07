# Semantic Modeling Concepts

DomainForge uses a specific set of primitives to model enterprise architecture. Unlike generic diagramming tools, these primitives have semantic meaning that allows for automated analysis, policy enforcement, and code generation.

## The Core Primitives

### 1. Entity

**What it is**: A logical component, service, actor, or system boundary.
**When to use**: Use Entities to represent *who* or *what* is performing actions.
**Examples**: `Customer`, `PaymentService`, `InventorySystem`.

```sea
Entity "PaymentService"
```

### 2. Resource

**What it is**: Passive data, infrastructure, or state that is acted upon.
**When to use**: Use Resources to represent databases, queues, file stores, or API endpoints.
**Examples**: `UserDatabase`, `OrderQueue`, `S3Bucket`.

```sea
Resource "UserDatabase" units
```

`units` denotes a count-based unit (i.e., number of instances/items). Use `units` for raw counts or specify another unit (e.g., `kg`, `USD`) when applicable; this aligns the resource with the `Unit` registry so flow quantities and aggregations can be compared consistently.

### 3. Flow

**What it is**: A directional interaction between two concepts, often involving a resource.
**When to use**: Use Flows to model data movement, API calls, or dependencies.
**Key Attribute**: Flows are strictly typed (e.g., `read`, `write`, `trigger`).

```sea
flow "Payment" {
    from = "PaymentService",
    to = "UserDatabase",
    interaction = "write",
    quantity = 1
}
```

### 4. Instance

**What it is**: A concrete realization of an Entity or Resource in a specific environment.
**When to use**: Use Instances to model physical deployments (e.g., "Production Payment Service" vs "Staging Payment Service").

```sea
Instance prod_payment_db of "UserDatabase" {
    env: "production",
    region: "us-east-1"
}
```

### 5. Policy

**What it is**: A constraint or rule that must be true for the model to be valid.
**When to use**: Use Policies to enforce security, compliance, or architectural standards.

```sea
Policy secure_writes as:
    forall f in flows: (f.to = "UserDatabase" and f.quantity >= 1)
```

## Modeling Patterns

### The "Actor-Action-Object" Pattern

Most architectural interactions can be modeled as:

- **Actor**: An Entity (e.g., WebServer)
- **Action**: A Flow (e.g., reads from)
- **Object**: A Resource (e.g., Database)

### The "Service-to-Service" Pattern

Direct communication between services is modeled as a Flow between two Entities.

- `Frontend` -> (Flow: HTTP/REST) -> `Backend`

## Semantic vs. Visual

In DomainForge, the *meaning* comes first. A box on a diagram is just a rendering of an Entity. Because the model is semantic, we can ask questions like:

- "Show me all services that write to the UserDatabase."
- "Are there any flows crossing trust boundaries without encryption?"

## See Also

- [Policy Evaluation Logic](policy-evaluation-logic.md)
- [Architecture Overview](architecture-overview.md)
