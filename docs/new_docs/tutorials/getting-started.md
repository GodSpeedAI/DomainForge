# Getting Started with DomainForge

This tutorial will guide you through installing the DomainForge CLI, writing your first Semantic Enterprise Architecture (SEA) model, and validating it.

**Time to complete**: 15 minutes

## Prerequisites

- **Rust**: You need Rust and Cargo installed. [Install Rust](https://rustup.rs/).

## Step 1: Install the CLI

Build and install the CLI tool directly from the source.

```bash
# From the root of the domainforge repository
cargo install --path . --features cli
```

Verify the installation:

```bash
sea-cli --version
# Output: sea-cli 0.1.0 (or current version)
```

## Step 2: Create Your First Model

Create a new file named `hello.sea` in your favorite text editor. We will model a simple "Hello World" system with a Web Server and a Database.

```sea
// hello.sea

entity WebServer {
    type = "service"
    layer = "frontend"
}

resource UserDB {
    type = "database"
    engine = "postgres"
}

flow greeting_flow {
    from = WebServer
    to = UserDB
    interaction = "read"
}
```

## Step 3: Parse and Validate

Run the CLI to parse your file. This checks for syntax errors and builds the internal graph.

```bash
sea-cli parse hello.sea
```

**Expected Output:**
```text
Successfully parsed hello.sea
Found:
  - 1 Entities
  - 1 Resources
  - 1 Flows
```

## Step 4: Add a Policy

Let's add a rule to ensure our architecture is secure. Append this to `hello.sea`:

```sea
policy secure_db_access {
    enforce: forall f in Flow {
        if f.to.type == "database" then f.interaction == "read"
    }
}
```

Run the parser again. The CLI automatically evaluates policies.

```bash
sea-cli parse hello.sea
```

**Expected Output:**
```text
...
Policy Results:
  [PASS] secure_db_access
```

## Step 5: Break the Policy

Change the interaction in `greeting_flow` to "write" and run the CLI again.

```sea
flow greeting_flow {
    // ...
    interaction = "write"
}
```

**Expected Output:**
```text
...
Policy Results:
  [FAIL] secure_db_access
     -> Violation at Flow(greeting_flow)
```

## Next Steps

- Learn more about [Semantic Modeling Concepts](../explanations/semantic-modeling-concepts.md).
- Try the [Python Binding Quickstart](python-binding-quickstart.md).
