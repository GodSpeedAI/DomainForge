# packaging — Native Installer and Archive Builders

## Overview
Per-platform packaging of the standalone `domainforge` CLI plus two shared
validators: a Linux ELF/ABI gate and a functional acceptance smoke test. Version
comes from a single `version.txt` guarded by `version.sh`.

## Interface
- `[version.sh:set -eu]` — usage contract `packaging/version.sh` (no arguments);
  prints the repository version and fails if any manifest disagrees.
- `[check-linux-binary.sh:set -eu]` — usage contract
  `packaging/check-linux-binary.sh <binary> <amd64|arm64>`; exits `2` on wrong
  argument count or unsupported architecture.
- `[smoke.sh:set -eu]` — usage contract
  `packaging/smoke.sh <domainforge> <valid.sea> <invalid.sea> <version>`; exits `2`
  on wrong argument count.
- `[smoke.sh:fail]` — helper that prints `FAIL: <msg>` to stderr and exits `1`.
- Child contracts: `./deb/EGM.md` — `.deb` builder; `./linux/EGM.md` — reproducible
  Linux tarball builder; `./macos/EGM.md` — macOS `.pkg`/`.dmg` builder.
- Unlinked (no EGM contract): `./windows/` holds only `build.ps1`, `domainforge.iss`,
  and `smoke.ps1`. PowerShell/Inno Setup are not in the recognized source-extension
  set, so no contract is emitted for that directory.

## Invariants
- `[version.sh:version.txt]` — the version must exist, be non-whitespace, and match
  `[0-9A-Za-z.+~-]*`; anything else exits `1`.
- `[version.sh:domainforge-core/Cargo.toml]` — the version in `version.txt` must
  equal the version in `domainforge-core/Cargo.toml` (`[package]` section),
  `domainforge-python/pyproject.toml`, `domainforge-typescript/package.json`, and
  `Cargo.lock`; any drift exits `1`.
- `[check-linux-binary.sh:expected_machine]` — the ELF machine must match the
  requested architecture (`Advanced Micro Devices X86-64` for `amd64`, `AArch64` for
  `arm64`), the class must be `ELF64`, and the type must be `EXEC` or `DYN`.
- `[check-linux-binary.sh:expected_interpreter]` — the program interpreter must be
  exactly `/lib64/ld-linux-x86-64.so.2` (amd64) or `/lib/ld-linux-aarch64.so.1`
  (arm64).
- `[check-linux-binary.sh:RPATH|RUNPATH]` — a binary carrying `RPATH`/`RUNPATH` is
  rejected, as is one containing the string `/nix/store`.
- `[check-linux-binary.sh:needed]` — the NEEDED set must be non-empty (a
  glibc-linked binary is expected) and may contain only `libc.so.6`, `libm.so.6`,
  `libdl.so.2`, `libpthread.so.0`, `librt.so.1`, `libgcc_s.so.1`,
  `ld-linux-aarch64.so.1`, `ld-linux-x86-64.so.2`, `linux-vdso.so.1`.
- `[check-linux-binary.sh:GLIBCXX_|CXXABI_]` — any `GLIBCXX_`/`CXXABI_` symbol
  requirement is rejected (the CLI must not need the C++ runtime).
- `[check-linux-binary.sh:baseline]` — the highest required `GLIBC_` version must be
  `<= 2.35` (the Ubuntu 22.04 baseline); no versioned GLIBC symbols at all is also a
  failure.
- `[smoke.sh:version_output]` — `--version` output must be exactly
  `domainforge <expected_version>`.
- `[smoke.sh:validate]` — a valid model must be accepted *and* print
  `Validation succeeded`; an invalid model must exit nonzero *and* emit a
  non-empty diagnostic.
- `[smoke.sh:declarations]` — `parse --ast --format json` must yield a non-empty
  `declarations` array.
- `[smoke.sh:compileall]` — the `domain-python` projection must materialize
  `pyproject.toml`, `README.md`, an `entities.py` under `src`, and at least one
  `test_*.py`, and the whole tree must pass `python3 -m compileall`.
- `[smoke.sh:bad_rc]` — projecting an invalid model must exit nonzero.
- `[smoke.sh:diff -r]` — two RDF projections at `--created-at
  2026-01-01T00:00:00Z` must be byte-identical (`model.ttl` must exist).
- `[smoke.sh:python3]` — `python3` (or `python`) is required; absence is an error,
  since the JSON and projection checks depend on it.

## Side Effects
- `[check-linux-binary.sh:readelf]` — spawns `readelf -h/-l/-d/--dyn-syms` against
  the candidate binary; read-only.
- `[smoke.sh:work]` — creates a `mktemp -d` work directory and removes it via
  `trap 'rm -rf "$work"' EXIT`.
- `[smoke.sh:project]` — writes `domain-python` and `rdf` projections into the temp
  work dir and runs `python3 -m compileall` over the generated package.
- `[smoke.sh:python]` — spawns an inline Python heredoc validator over the emitted
  AST JSON.
- Child builders write installers into a caller-supplied `<outdir>`; see
  `./deb/EGM.md`, `./linux/EGM.md`, and `./macos/EGM.md`.

## Verification
- Run: `sh packaging/version.sh`
- Run: `sh packaging/check-linux-binary.sh <binary> amd64`
- Run: `sh packaging/smoke.sh <domainforge> <valid.sea> <invalid.sea> <version>`
