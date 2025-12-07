# Migrating Schema Versions

As DomainForge evolves, the `.sea` file format may change. This guide explains how to handle migrations.

## Detection

The CLI will warn if you are using deprecated syntax.

```text
Warning: 'interface' keyword is deprecated. Use 'flow' instead.
```

## Breaking Changes Strategy

### 1. Automated Migration (Future)

We plan to add `sea-cli migrate <file>` to automatically update syntax.

### 2. Manual Migration

For now, manual updates are required.

**Example: Renaming `connection` to `flow`**
*Old:*
```sea
connection "c1" from "A" to "B"
```
*New:*
```sea
flow "Data" from "A" to "B"
```

## Backward Compatibility

We strive to support the previous MAJOR version's syntax for one release cycle.

- **v0.4**: Introduces `flow`, deprecates `connection`. Both work.
- **v0.5**: Removes `connection`.

## Best Practices

- Pin the CLI version in your CI pipeline to avoid unexpected breakages.
- Read the `CHANGELOG.md` before upgrading.
