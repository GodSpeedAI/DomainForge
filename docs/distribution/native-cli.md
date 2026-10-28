# Native CLI distribution

DomainForge publishes the standalone `domainforge` command as precompiled
packages on the GitHub Release for each `vX.Y.Z` tag. The native packages are
additive: crates.io, PyPI, npm, and the WASM package keep their existing
publication paths and the same unified version.

No Rust, Cargo, Bun, Node.js, Python, Devbox, or build environment is needed to
use an installed native binary.

## Artifact matrix

| Platform | Architecture | Artifact |
| --- | --- | --- |
| Linux | x86_64 | `domainforge_<version>_amd64.deb` |
| Linux | x86_64 | `domainforge-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `domainforge_<version>_arm64.deb` |
| Linux | aarch64 | `domainforge-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x86_64 | `domainforge-<version>-x86_64-pc-windows-msvc.zip` |
| Windows | x86_64 | `domainforge-<version>-x86_64-pc-windows-msvc-setup.exe` |
| macOS | arm64 + x86_64 | `domainforge-<version>-macos-universal.pkg` |
| macOS | arm64 + x86_64 | `domainforge-<version>-macos-universal.dmg` |

Every release also carries `SHA256SUMS` over the native assets. Tagged releases
are built from the tag commit and compared against that commit before assets
are attached. On repositories where GitHub attestations are enabled, the SLSA
build provenance attestation binds the artifact digests to the tag commit.

If the macOS Intel cross-build ever fails, the workflow ships
`domainforge-<version>-macos-arm64.pkg/.dmg` and builds native Intel packages on
an Intel runner as `domainforge-<version>-macos-x86_64.pkg/.dmg`. It never
labels an arm64-only binary as universal.

## Linux

The `.deb` installs `/usr/bin/domainforge`, the licence, and example models
under `/usr/share/doc/domainforge/examples/`. Dependencies are computed with
`dpkg-shlibdeps`; the only expected runtime dependency is the system C library
family already present on Ubuntu 22.04 or newer.

```bash
sudo apt-get install ./domainforge_0.21.2_amd64.deb
domainforge --version
sudo apt-get remove domainforge
```

The tarball is independent of Debian packaging:

```bash
tar -xzf domainforge-0.21.2-x86_64-unknown-linux-gnu.tar.gz
cd domainforge-0.21.2-x86_64-unknown-linux-gnu
./domainforge --version
```

Both Linux packages are built on Ubuntu 22.04 runners with glibc 2.35, which is
the supported ABI baseline. CI rejects a binary whose maximum required
`GLIBC_x.y` symbol is newer than 2.35, whose ELF interpreter is wrong, that
carries an RPATH/RUNPATH, references `/nix/store`, or links an unexpected
library. The installed package is also exercised inside clean `ubuntu:22.04`
and `debian:12` containers.

### NVIDIA Jetson (Ubuntu 22.04 aarch64)

Use `domainforge_<version>_arm64.deb`. It is built natively on an Ubuntu 22.04
ARM64 runner, not cross-compiled, so the ELF machine, interpreter
(`/lib/ld-linux-aarch64.so.1`), and glibc symbol ceiling match the target.

## macOS

```bash
sudo installer -pkg domainforge-0.21.2-macos-universal.pkg -target /
/usr/local/bin/domainforge --version
sudo rm -f /usr/local/bin/domainforge
sudo rm -rf /usr/local/share/doc/domainforge
sudo pkgutil --forget ai.godspeed.domainforge
```

The `.dmg` contains the same `.pkg` payload (verified by SHA-256 in CI), the
licence, and install notes. The universal executable is checked with
`lipo -archs` after installation.

The package is **unsigned and not notarized**: no Apple Developer ID
certificate is used. Gatekeeper may require Control-click → Open, or removal of
the quarantine attribute. The executable is ad-hoc signed by the build, which
is not the same as a Developer ID signature and is not a notarization claim.

## Windows

The `.zip` is self-contained: `domainforge.exe`, `LICENSE`, and
`examples/basic.sea` at the archive root.

The `-setup.exe` is a per-user Inno Setup installer. It installs to
`%LOCALAPPDATA%\Programs\DomainForge`, adds that directory to the user `PATH`,
and removes both the files and the `PATH` entry on uninstall. It does not
require administrator privileges and installs no service or extra runtime.

```powershell
# Silently, per user
.\domainforge-0.21.2-x86_64-pc-windows-msvc-setup.exe /VERYSILENT /CURRENTUSER
domainforge --version
```

The installer is **unsigned** (no Authenticode certificate).

## What the installed binary is tested against

Each platform installs the produced package and runs the same functional
acceptance workflow against the installed executable:

```bash
domainforge --version
domainforge validate --format human --no-color examples/basic.sea
domainforge validate --format human --no-color examples/invalid.sea   # must fail
domainforge parse --ast --format json examples/basic.sea               # valid JSON
domainforge project --format domain-python examples/basic.sea ./out     # materializes files
domainforge project --format rdf --created-at 2026-01-01T00:00:00Z examples/basic.sea ./rdf-a
domainforge project --format rdf --created-at 2026-01-01T00:00:00Z examples/basic.sea ./rdf-b
diff -r ./rdf-a ./rdf-b                                                # byte-identical
```

The smoke test also compiles the generated Python package, so a projection that
emits unreadable files fails acceptance. `examples/invalid.sea` is the existing
repository fixture `fixtures/projection_cell/invalid/01_syntax_broken.sea`.

## npm and PyPI are separate installation methods

`npm install -g @godspeedai/domainforge` installs the napi-rs bindings package;
`pip install domainforge` installs the Python extension. Neither is the same
artifact as the native `domainforge` executable, and neither is replaced by
this distribution. The GitHub Release packages are the only channel that gives
you the standalone CLI without a package-manager runtime.

## Release lifecycle

The native packages use the existing Release Please lifecycle:

1. Conventional commits update the standing Release Please PR.
2. A maintainer merges the release PR.
3. Release Please creates the single `vX.Y.Z` tag and GitHub Release.
4. `deploy.yml` validates manifest parity, builds and install-tests the native
   packages, and only then attaches `SHA256SUMS` and the packages.
5. The same `deploy.yml` run publishes crates.io, PyPI, and npm.

`package.yml` is reusable (`workflow_call`), can be dispatched manually, and
also runs on pushes to `main` and `feat/native-cli-distribution`. Branch and
manual runs upload release-candidate workflow artifacts; they do not create or
modify a GitHub Release.

## Building locally

```bash
version=$(sh packaging/version.sh)
cargo build --release --locked -p domainforge-core --bin domainforge --features cli

# Linux
sh packaging/check-linux-binary.sh target/release/domainforge amd64
sh packaging/deb/build.sh "$version" amd64 target/release dist
sh packaging/linux/build-tarball.sh "$version" x86_64-unknown-linux-gnu target/release dist

# Functional acceptance against the built binary
sh packaging/smoke.sh target/release/domainforge \
  domainforge-core/examples/basic.sea \
  fixtures/projection_cell/invalid/01_syntax_broken.sea \
  "$version"
```

macOS and Windows packaging scripts are `packaging/macos/build.sh` and
`packaging/windows/build.ps1`; see `package.yml` for their exact invocation.

## Limitations

- The Linux baseline is glibc 2.35 (Ubuntu 22.04). Older distributions are not
  supported by these packages.
- Windows is x86_64 only; the Windows ARM64 target is not part of this matrix.
- macOS and Windows packages are unsigned. Signing and notarization require
  credentials that are deliberately not stored in this repository.
- The release assets are only attached after all platform jobs pass. A failed
  packaging job leaves the release without native assets and fails the deploy
  workflow rather than publishing a partial set.
