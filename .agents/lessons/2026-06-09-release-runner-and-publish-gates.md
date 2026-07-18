# Release Runner And Publish Gates

- For macOS release targets, do not rely on `macos-latest` for Intel builds. Use an explicit Intel runner label for `x86_64-apple-darwin` and an Apple Silicon runner for `aarch64-apple-darwin`.
- Keep release workflow artifact budgets aligned with pre-release validation. The WASM bundle for v0.11.0 is larger than 2 MiB, so the CI budget and release budget must agree.
- A GitHub Release created by a workflow token may not fan out to release-triggered publish workflows. Be ready to use `workflow_dispatch` or a publishing token pattern that is allowed to trigger downstream workflows.
- Windows Git Bash can produce checksum output that needs normalization before comparing decrypted secret SHA256 values. Strip path-mode escape prefixes before comparison.
- Cross-target PyPI publishing should explicitly run `rustup target add` for the matrix target before invoking maturin.
- npm publish E404 after a valid scoped package tarball usually points to npm organization/scope/token permission or package creation access, not to a build artifact problem.
- If release preparation happens on `dev`, do not let the final release script tag that branch directly. Enforce a `main`-only release cut or explicitly automate the `dev` to `main` promotion first; otherwise the published tag can drift away from the default branch.
