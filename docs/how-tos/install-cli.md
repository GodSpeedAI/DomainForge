# Install the SEA CLI

Goal: Install and verify the DomainForge SEA CLI on Linux, macOS, and Windows.

## Prerequisites

- Rust toolchain 1.77+ installed via `rustup` (recommended target triples: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`).
- Build essentials for your OS (gcc/clang + make on Linux, Xcode CLTs on macOS, Build Tools for Visual Studio on Windows).
- Optional: `just` for running test recipes.

## Install from Source (recommended until binary releases are published)

1. Clone the repo and install:

```bash
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
# For the CLI only
cargo install --path sea-core --features cli
# If you need SHACL validation (validate-kg), include the shacl feature as well
cargo install --path sea-core --features "cli,shacl"
```

2. Ensure Cargo bin is on PATH (Linux/macOS):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

3. Verify the binary:

```bash
sea --version
sea validate --format human sea-core/examples/basic.sea
```

- Expected: version prints `sea-core 0.1.0` (or newer) and validation exits successfully.

## Install via GitHub Release (when artifacts are available)

- Download the archive that matches your OS/arch (e.g., `sea-x86_64-unknown-linux-gnu.tar.gz`).
- Extract and place the `sea` binary on your PATH (`/usr/local/bin` or `%USERPROFILE%\.cargo\bin`).
- Run `sea --version` to confirm the binary is executable.

## Windows-specific Notes

- Use **Developer PowerShell** or **x64 Native Tools Command Prompt** to ensure MSVC is available.
- Set the default toolchain: `rustup default stable-x86_64-pc-windows-msvc`.
- If OpenSSL build errors appear, install `vcpkg` and ensure `VCPKGRS_DYNAMIC=1` before running `cargo install`.
- Ensure Rust binaries are on your PATH in PowerShell:

```powershell
$env:PATH += ";$HOME\.cargo\bin"
# Or persist for current user
setx PATH "$env:PATH;$HOME\.cargo\bin"
```

## macOS-specific Notes

- Install Xcode Command Line Tools: `xcode-select --install`.
- On Apple Silicon, prefer the native toolchain; if you need x86_64 binaries, use Rosetta with `arch -x86_64 cargo install ...`.

## Linux-specific Notes

- Ensure `pkg-config` and `openssl-dev` are installed (`sudo apt-get install build-essential pkg-config libssl-dev`).
- If building inside a container, mount a writable `$CARGO_HOME` to avoid permission errors.

## Running Smoke Tests after Installation

```bash
# For users installing from source (developer/local builds):
sea validate --format human sea-core/examples/basic.sea
sea project --format calm sea-core/examples/basic.sea /tmp/basic.calm.json
sea project --format kg sea-core/examples/basic.sea /tmp/basic.ttl

# For users installing from a release binary: if `sea` provides a built-in self-check command, run it; otherwise validate using a minimal inline example:
# Example (if no self-check command exists):
echo 'entity A {}' > /tmp/minimal.sea
sea validate --format human /tmp/minimal.sea
```

- These commands confirm parsing, CALM export, and KG export paths.

## Upgrading or Reinstalling

- To force an upgrade after changes: `cargo install --path sea-core --features cli --force`.
- Remove old binaries from other PATH locations to avoid running stale versions.

## Troubleshooting

- **Linker errors on Windows**: reinstall the MSVC build tools and re-run `rustup update`.
- **Python interference (macOS/Homebrew)**: unset `PYTHONHOME` and rebuild; or create a fresh virtualenv before invoking Cargo to avoid conflicting headers.
- **Missing OpenSSL**: install `libssl-dev` (Debian/Ubuntu) or `openssl@3` (Homebrew) and set `PKG_CONFIG_PATH` accordingly.
- **Permission denied**: install to a user-writable directory or run `cargo install` without `sudo`.

## Verification Checklist

- [ ] `sea --version` prints the expected version string.
- [ ] `sea validate` succeeds against `sea-core/examples/basic.sea`.
- [ ] `sea project --format calm` produces a CALM file containing `sea:version` metadata.
  - [ ] `sea project --format kg` generates Turtle without SHACL errors when run through `sea validate-kg` (requires installing with the `shacl` feature: `--features cli,shacl`).

## Links

- Tutorials: [Getting Started](../tutorials/getting-started.md)
- Reference: [CLI Commands](../reference/cli-commands.md), [Configuration](../reference/configuration.md), [Versioning Strategy](../explanations/versioning-strategy.md)

## Post-install Configuration

- Configure logging verbosity with CLI flags: `--verbose` or `--quiet` (mutually exclusive).
- Set `SEA_COLOR=always` to force colorized output if your terminal strips ANSI codes.
- Use `.sea-registry.toml` in your project root to manage namespaces; the CLI discovers it automatically during parse/validate/project commands.

## Example CI Step

```yaml
- name: Install SEA CLI
  run: |
    rustup override set stable
    cargo install --path sea-core --features cli --force
- name: Validate models
  run: |
    sea validate --format human models/**/*.sea
```

- Cache `$CARGO_HOME` between runs to speed up subsequent installs.
