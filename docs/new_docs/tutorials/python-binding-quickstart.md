# Python Binding Quickstart

This guide shows how to use DomainForge programmatically using Python. This is useful for writing custom analysis scripts, generating reports, or integrating SEA into Python-based tooling.

## Prerequisites

- Python 3.8+
- `maturin` (for building bindings locally, or install from PyPI if available)

## Step 1: Installation

If you are working in the repo:

```bash
# Create venv
python -m venv .venv
source .venv/bin/activate

# Install dependencies and build bindings
pip install maturin
maturin develop --features python
```

## Step 2: Create a Script

Create `analyze_model.py`.

```python
import domainforge
from domainforge import Entity, Flow

# 1. Define a simple model string (or load from file)
sea_content = """
entity Web { type = "service" }
resource DB { type = "database" }
flow f1 { from = Web, to = DB, interaction = "read" }
"""

# 2. Parse the content
try:
    model = domainforge.parse(sea_content)
    print("Parse successful!")
except Exception as e:
    print(f"Error: {e}")
    exit(1)

# 3. Inspect the graph
print(f"Entities: {len(model.entities)}")
print(f"Flows: {len(model.flows)}")

# 4. Iterate and analyze
for flow in model.flows:
    source = flow.from_entity
    target = flow.to_entity
    print(f"Flow: {source.name} -> {target.name} ({flow.interaction})")

# 5. Programmatic Modification (Hypothetical API)
# new_entity = Entity(name="Cache", type="redis")
# model.add_entity(new_entity)
```

## Step 3: Run the Script

```bash
python analyze_model.py
```

**Expected Output:**
```text
Parse successful!
Entities: 1
Flows: 1
Flow: Web -> DB (read)
```

## Integration with Pytest

You can use DomainForge to test your architecture definitions.

```python
# test_architecture.py
import pytest
import domainforge

def test_no_plaintext_passwords():
    model = domainforge.parse_file("production.sea")
    for flow in model.flows:
        if "password" in flow.payload.lower():
            assert flow.encrypted == True, f"Flow {flow.name} sends passwords in plain text!"
```

## See Also

- [Cross-Language Binding Strategy](../explanations/cross-language-binding-strategy.md)
