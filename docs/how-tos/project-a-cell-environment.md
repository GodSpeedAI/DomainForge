# How to project a cell environment

## 1. Declare a `Cell`

```sea
@namespace "godspeed.cells.repair"
@version "1.0.0"
@owner "godspeed-platform"

Cell "RepairAgent"
    @profile "python-agent-v1"
    @network_default "deny"

SystemDependency "git" version "2.45"
Runtime "python" version "3.13"
Tool "uv" version "0.9"

DependencySet "python-application"
    @ecosystem "python"
    @manifest "pyproject.toml"
    @lockfile "uv.lock"
    @install "uv sync --frozen"
```

`@profile` must name one of `python-agent-v1`, `typescript-agent-v1`,
`rust-agent-v1`, `polyglot-agent-v1` (see
`docs/cell-environment-projections.md` for their defaults). Anything you
declare explicitly overrides that profile's default for the same name.

## 2. Project it

```bash
domainforge project --format cell \
  --created-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  path/to/model.sea out/cell
```

`input` and `output` are positional (`input` then `output`); `--format`
and the other flags come before them.

## 3. (Optional) add overrides

Create `domainforge.cell.toml` next to your model:

```toml
schema = "domainforge-cell-overrides/v1"

[mise.tools]
uv = "0.9.8"

[resources]
cpu = 2
memory_mb = 4096
```

```bash
domainforge project --format cell \
  --overrides path/to/domainforge.cell.toml \
  --created-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  path/to/model.sea out/cell
```

Overrides may only tighten/reduce/specialize — see
`docs/how-tos/override-cell-realization.md` for the safe/unsafe rules.

## 4. Use the output

Each manifest lives in its own subdirectory; run its tool from there so it
finds its config (`devbox.json` in `devbox/`, `mise.toml` in `mise/`).

```bash
# Devbox shell — run from the directory containing devbox.json
cd out/cell/devbox
devbox shell

# Mise install + prove — run from the directory containing mise.toml
cd ../mise
mise install
mise run prove-environment
```

## 5. (Optional) generate only some components

```bash
domainforge project --format cell --only devbox,mise \
  path/to/model.sea out/cell
```

`cell.lock` and `semantic/cell-ir.json` are always written in full — this
is a fast-iteration convenience, not a semantic subset.
