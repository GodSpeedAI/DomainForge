# How to override cell realization

`domainforge.cell.toml` (schema `domainforge-cell-overrides/v1`) lets you
specialize *how* a declared cell is realized without changing what it
*means*. Every table below is optional; unknown keys anywhere in the
document fail closed (`CELL010`).

## Safe overrides

```toml
schema = "domainforge-cell-overrides/v1"

[devbox.packages]
openssl = "openssl_3"     # exact Devbox package for a declared SystemDependency

[mise.tools]
python = "3.13.5"         # must be >= the declared version (CELL011 if weaker)

[dependency_sets.python-application]
install_command = "uv sync --frozen --no-dev"

[resources]                # may only reduce vs the Cell's declared ceiling (CELL012)
cpu = 2
memory_mb = 4096
disk_mb = 10240
timeout_seconds = 900

[network.endpoints.package-mirror]   # must reference a declared Endpoint (CELL013)
host = "pypi.mirror.internal"
port = 443

[evidence]
extra_probes = ["python -c 'import ssl'"]   # always additive
```

Every override target (`[mise.tools].<name>`, `[dependency_sets.<name>]`,
`[network.endpoints.<name>]`) must already be declared in `.sea` —
overriding something undeclared fails with `CELL013`.

## What overrides may never do

- weaken a declared version (`CELL011`)
- raise a resource value above its declared ceiling (`CELL012`)
- reference anything not already declared (`CELL013`)
- introduce an unknown key at any level (`CELL010`, via
  `#[serde(deny_unknown_fields)]`)

There is no override key that broadens network access, disables evidence,
or turns a read-only mount read-write — those keys simply don't exist in
the schema.

## Unsafe overrides

For a genuine, time-boxed exception, add:

```toml
[unsafe_overrides]
enabled = true
ticket = "GS-1427"
authority = "platform-architecture"
rationale = "Temporary compatibility investigation"
expires_at = "2026-08-01T00:00:00Z"
```

All five fields are required together (`CELL015` if incomplete). An
expired `expires_at` fails closed (`CELL016`). An accepted unsafe override
is recorded in `cell.lock` (`overrides.unsafe = true`) and in
`authority/dependency-mutation-policy.json` — it is never silently
accepted, and it always prints a warning.
