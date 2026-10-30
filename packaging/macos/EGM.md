# packaging/macos — macOS pkg/dmg Builder

## Overview
Builds the macOS `.pkg` and `.dmg` for one architecture or a universal binary. The
package installs `/usr/local/bin/domainforge` and is explicitly UNSIGNED (no Apple
Developer ID, no notarization); the executable is ad-hoc signed after `lipo`.

## Interface
- `[build.sh:set -eu]` — usage contract:
  `packaging/macos/build.sh <version> <universal|arm64|x86_64> <arm64dir|''>
  <x86dir|''> <outdir>`; exits `2` on wrong argument count, bad version, or
  unsupported label.
- `[build.sh:label]` — accepts exactly `universal`, `arm64`, or `x86_64`.
- `[build.sh:lipo]` — for `universal`, fuses `$arm_dir/domainforge` and
  `$x86_dir/domainforge` with `lipo -create ... -output stage/bin/domainforge`.
- `[build.sh:codesign]` — runs `codesign --force --sign -` (ad-hoc) on the staged
  binary so the arm64 slice can execute on Apple silicon.
- `[build.sh:pkgbuild]` — builds the `.pkg` with identifier
  `ai.godspeed.domainforge` and `--install-location /usr/local`.
- `[build.sh:hdiutil]` — builds the `.dmg` (`-format UDZO`) from a staging folder
  containing the `.pkg`, `LICENSE`, and a generated `README-install.txt`.

## Invariants
- `[build.sh:universal]` — `universal` requires both `arm_dir` and `x86_dir` to be
  non-empty; `arm64`/`x86_64` require their respective directory. A missing one is
  an error, not a silent downgrade.
- `[build.sh:version]` — the version must be non-empty and contain only
  `[0-9A-Za-z.+~-]`.
- `[build.sh:[ -x ... ]]` — every required binary directory must contain an
  executable `domainforge` before any packaging happens.
- `[build.sh:codesign --sign -]` — the signature is deliberately ad-hoc (`-`), not
  a Developer ID; the package remains unsigned and un-notarized by design, and the
  README tells users to clear the quarantine flag.
- `[build.sh:test -s]` — both the `.pkg` and `.dmg` must be non-empty after build.

## Side Effects
- `[build.sh:work]` — creates a `mktemp -d` work tree (`stage`, `dmg` subdirs) and
  removes it via `trap 'rm -rf "$work"' EXIT`.
- `[build.sh:lipo]` — writes the fused binary into the staging tree.
- `[build.sh:codesign]` — mutates the code signature of the staged binary.
- `[build.sh:pkgbuild]` / `[build.sh:hdiutil]` — write
  `domainforge-<version>-macos-<label>.pkg` and `.dmg` into `<outdir>` after
  `mkdir -p "$outdir"`.
- `[build.sh:install]` — copies `LICENSE` and the two `.sea` examples into
  `stage/share/doc/domainforge`.

## Verification
- Run: `sh packaging/macos/build.sh <version> universal <arm64dir> <x86dir> <outdir>`
