# packaging/deb — Debian Package Builder

## Overview
Builds `domainforge_<version>_<arch>.deb` from a pre-built native release binary,
computing real library dependencies via `dpkg-shlibdeps` rather than hand-written
guesses.

## Interface
- `[build.sh:set -eu]` — usage contract:
  `packaging/deb/build.sh <version> <amd64|arm64> <bindir> <outdir>`; exits `2` on
  wrong argument count, bad version, or unsupported architecture.
- `[build.sh:check-linux-binary.sh]` — delegates ABI/baseline validation to
  `packaging/check-linux-binary.sh "$binary" "$arch"` before packaging.
- `[build.sh:dpkg-shlibdeps]` — runs `dpkg-shlibdeps -O -e"$binary"` inside a
  throwaway `debian/control` tree to derive the `Depends:` field.
- `[build.sh:dpkg-deb --build]` — assembles the staged tree
  (`usr/bin/domainforge`, `usr/share/doc/domainforge/copyright`,
  `usr/share/doc/domainforge/examples/{basic,invalid}.sea`) into the `.deb`.
- `[build.sh:dpkg-deb --field]` — post-build assertions on `Package`, `Version`,
  `Architecture` plus a `--contents` check for `./usr/bin/domainforge`.

## Invariants
- `[build.sh:version]` — the version must be non-empty and contain only
  `[0-9A-Za-z.+~-]`; anything else is rejected with exit `2`.
- `[build.sh:deps]` — the computed `Depends` must contain `libc6`; a shlibdeps
  result without it aborts the build.
- `[build.sh:dpkg-shlibdeps is required]` — a missing `dpkg-shlibdeps` (dpkg-dev)
  is a hard failure, since guessing dependencies is explicitly not allowed.
- `[build.sh:dpkg-deb --field Architecture]` — the emitted package architecture
  must equal the requested `amd64`/`arm64` argument.

## Side Effects
- `[build.sh:work]` — creates a `mktemp -d` staging tree (including a synthetic
  `$work/shlibs/debian/control`) and removes it via `trap 'rm -rf "$work"' EXIT`.
- `[build.sh:dpkg-deb --build]` — writes `domainforge_<version>_<arch>.deb` into
  `<outdir>` (`mkdir -p "$outdir"` is performed first).
- `[build.sh:install]` — copies the binary, `LICENSE`, and two `.sea` examples into
  the staging tree with fixed modes (`0755`, `0644`).

## Verification
- Run: `sh packaging/deb/build.sh <version> amd64 <bindir> <outdir>`
