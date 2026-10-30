# packaging/linux — Linux Tarball Builder

## Overview
Builds a self-contained, reproducible Linux tarball holding the standalone
`domainforge` binary plus examples, independent of the `.deb`.

## Interface
- `[build-tarball.sh:set -eu]` — usage contract:
  `packaging/linux/build-tarball.sh <version> <target-triple> <bindir> <outdir>`;
  exits `2` on wrong argument count, bad version, or unsupported target.
- `[build-tarball.sh:target]` — accepts exactly
  `x86_64-unknown-linux-gnu` or `aarch64-unknown-linux-gnu`.
- `[build-tarball.sh:stage]` — stages `domainforge`, `LICENSE`,
  `examples/basic.sea`, `examples/invalid.sea`, and a generated `README.txt`
  documenting the glibc 2.35 baseline and example commands.
- `[build-tarball.sh:tar]` — packs with
  `tar --sort=name --mtime='UTC 1970-01-01' --owner=0 --group=0 --numeric-owner`
  piped through `gzip -9n`.
- `[build-tarball.sh:tar -tzf]` — post-build assertion that the archive contains
  `^${name}/domainforge$`.

## Invariants
- `[build-tarball.sh:version]` — the version must be non-empty and contain only
  `[0-9A-Za-z.+~-]`.
- `[build-tarball.sh:gzip -9n]` — the `-n` flag and the fixed `--mtime`/owner/group
  plus `--sort=name` are what make the archive byte-reproducible; dropping any of
  them breaks reproducibility.
- `[build-tarball.sh:test -s "$archive"]` — an empty archive is a failure.

## Side Effects
- `[build-tarball.sh:work]` — creates a `mktemp -d` staging directory and removes
  it via `trap 'rm -rf "$work"' EXIT`.
- `[build-tarball.sh:archive]` — writes `<outdir>/<name>.tar.gz` after
  `mkdir -p "$outdir"`.
- `[build-tarball.sh:install]` — copies the binary and assets into the staging tree
  with fixed modes (`0755`, `0644`).

## Verification
- Run: `sh packaging/linux/build-tarball.sh <version> x86_64-unknown-linux-gnu <bindir> <outdir>`
