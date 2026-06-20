# Migrating from `sea-dsl` to `domainforge` (Python)

The Python package was renamed on PyPI and the import module changed:

| Before               | After              |
|----------------------|--------------------|
| `sea-dsl` (PyPI)     | `domainforge`      |
| `import sea_dsl`     | `import domainforge` |

## Install

```bash
# old: pip install sea-dsl
pip install domainforge
```

## Import

```python
# old: from sea_dsl import Graph, Entity
from domainforge import Graph, Entity
```

The API surface is otherwise unchanged.
