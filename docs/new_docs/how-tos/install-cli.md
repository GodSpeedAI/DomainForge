# Install CLI

Goal: Install and verify the DomainForge CLI on Linux/macOS/Windows.

## Prerequisites

- Rust toolchain 1.77+ (recommended via `rustup`).
- Build tools for your OS (Xcode CLTs on macOS, Build Tools for Visual Studio on Windows).

## Install from Source

1) Clone the repo: `git clone https://github.com/GodSpeedAI/DomainForge.git && cd DomainForge`.
2) Build/install the CLI: `cargo install --path sea-core --features cli`.
3) (Optional) Add to PATH if cargo bin is not already there: `export PATH="$HOME/.cargo/bin:$PATH"`.

## Verify

- Version: `sea --version` (should report `sea-core 0.1.0` or later).
- Smoke test: `sea validate --format human sea-core/examples/basic.sea` (should finish without errors).
- Help: `sea --help` and `sea validate --help` for command options.

## Troubleshooting

- If linking fails on Windows, open a new Developer PowerShell and ensure the MSVC toolchain matches `rustup default stable-x86_64-pc-windows-msvc`.
- For Homebrew Python interference, rebuild with a clean virtualenv or set `PYTHONHOME` empty before running `cargo install`.
- If the binary already exists, reinstall with `cargo install --path sea-core --features cli --force`.
- Prebuilt releases (PyPI/npm/crates) will be available once release tokens are configured; until then, source build is the supported path.
