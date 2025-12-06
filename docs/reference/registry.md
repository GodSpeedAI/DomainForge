# Workspace Registry (`.sea-registry.toml`)

Purpose: map files to namespaces so identifiers stay deterministic across a workspace.

## File format

```toml
version = 1
default_namespace = "default" # optional fallback

[[namespaces]]
namespace = "logistics"
patterns = ["examples/namespaces/logistics/**/*.sea"]

[[namespaces]]
namespace = "finance"
patterns = ["examples/namespaces/finance/**/*.sea"]
```

- `version`: currently only `1` is supported.
- `default_namespace`: applied when no patterns match.
- `namespaces`: each entry declares the namespace and one or more glob `patterns` (relative to the registry file).
- Schema: `schemas/sea-registry.schema.json` (validated in tests).

## Ambiguity handling

- If multiple patterns match a file, the **longest literal prefix** wins.
- Tie-breaker: deterministic alphabetical order.
- Prefer using `--fail-on-ambiguity` (CLI) when you want ambiguous matches to error.

## CLI usage

```bash
sea validate --registry .sea-registry.toml models/payments/payment.sea
sea graph --registry .sea-registry.toml models/**/*.sea
```

- Without a registry, unnamed namespaces default to `"default"`.
- Combine with `--format json` for CI pipelines that need namespace-aware outputs.

## Maintenance tips

- Keep glob patterns specific to avoid accidental overlaps.
- Regenerate or pin baselines in tests when changing registry behavior.
- Store the registry at the workspace root so relative patterns are stable.

## See also

- [Configuration](./configuration.md) for CLI flags and env vars
- [Parse SEA Files](../how-tos/parse-sea-files.md) for parsing examples
