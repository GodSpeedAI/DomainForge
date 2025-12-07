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
@namespace "hello.world"

Entity "WebServer"
Entity "UserDatabase"
Resource "UserData" units
Flow "UserData" from "WebServer" to "UserDatabase" quantity 1
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
Policy secure_db_access as:
    forall f in flows: (f.to = "UserDatabase" and f.quantity <= 1)
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

Increase the quantity on the flow and re-validate to see the policy fail.

```sea
Flow "UserData" from "WebServer" to "UserDatabase" quantity 5
```

**Expected Output:**
```text
...
Policy Results:
  [FAIL] secure_db_access
     -> Violation at Flow("UserData")
```

## Next Steps

- Learn more about [Semantic Modeling Concepts](../explanations/semantic-modeling-concepts.md).
- Try the [Python Binding Quickstart](python-binding-quickstart.md).
