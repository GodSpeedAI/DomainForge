# CI/CD Quick Reference Guide

## For Developers

### Running CI Commands Locally

All CI commands can now be run locally using `just`:

```bash
# Run the same tests CI runs
just ci-test-rust      # Rust tests
just ci-test-python    # Python tests
just ci-test-ts        # TypeScript tests

# Verify a binary like CI does
just ci-verify-binary target/debug/sea

# Check binary size
just ci-check-binary-size target/debug/sea

# Package a binary
just ci-package-binary target/debug/sea output.tar.gz

# Verify a package
just ci-verify-package output.tar.gz sea
```

### Using the CI Tasks Script Directly

```bash
# Check file size
python3 scripts/ci_tasks.py check-size \
  --file target/debug/sea \
  --max-bytes 52428800 \
  --label "CLI binary"

# Package files
python3 scripts/ci_tasks.py package \
  --input target/debug/sea \
  --output sea.tar.gz

# Verify binary runs
python3 scripts/ci_tasks.py verify-cli \
  --binary target/debug/sea

# Unpack and verify
python3 scripts/ci_tasks.py unpack-verify \
  --archive sea.tar.gz \
  --binary-name sea
```

### Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually on all files
pre-commit run --all-files

# Run on staged files only
pre-commit run

# Update hook versions
pre-commit autoupdate
```

### Troubleshooting CI Failures

When a CI job fails:

1. **Identify the failing step** in the GitHub Actions UI
2. **Find the `just` command** that failed (e.g., `just ci-test-rust`)
3. **Run it locally**: `just ci-test-rust`
4. **Debug** using your normal tools
5. **Fix and verify** locally before pushing

### Common CI Tasks

#### Update Size Limits

Edit `.github/workflows/ci.yml` or `.github/workflows/release.yml`:

```yaml
env:
  CLI_MAX_BYTES: "52428800" # 50MB
  WASM_MAX_BYTES: "1048576" # 1MB
  CLI_ARTIFACT_MAX_BYTES: "73400320" # 70MB
```

#### Add a New CI Check

1. Add a recipe to `justfile`:

   ```makefile
   ci-my-check:
       @echo "Running my check..."
       ./scripts/my_check.sh
   ```

2. Add a step to the workflow:
   ```yaml
   - name: Run my check
     run: just ci-my-check
   ```

#### Test Workflow Changes Locally

```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"

# Test the commands that would run
just ci-test-rust
just ci-verify-binary target/debug/sea
```

---

## For CI/CD Maintainers

### Workflow Structure

```
.github/workflows/
├── ci.yml                    # Main CI pipeline
├── release.yml               # Release builds
├── publish-python.yml        # PyPI publishing
├── dependabot-automerge.yml  # Auto-merge deps
└── dependency-review.yml     # Dependency scanning
```

### Key Files

- **`scripts/ci_tasks.py`**: Cross-platform CI automation
- **`justfile`**: Task runner (local + CI)
- **`.pre-commit-config.yaml`**: Pre-commit hooks
- **`scripts/resolve_rust_binary.py`**: Find Rust binaries

### Environment Variables

| Variable                 | Default  | Description                            |
| ------------------------ | -------- | -------------------------------------- |
| `CLI_MAX_BYTES`          | 52428800 | Max CLI binary size (50MB)             |
| `WASM_MAX_BYTES`         | 1048576  | Max WASM bundle size (1MB)             |
| `CLI_ARTIFACT_MAX_BYTES` | 73400320 | Max packaged artifact size (70MB)      |
| `CACHE_VERSION`          | v3       | Cache key version (bump to invalidate) |

### Cache Strategy

Caches include `Cargo.lock` hash for automatic invalidation:

```yaml
key: ${{ runner.os }}-${{ env.CACHE_VERSION }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

To force cache invalidation, bump `CACHE_VERSION`.

### Adding New Platforms

1. Add to matrix in `ci.yml`:

   ```yaml
   matrix:
     include:
       - os: ubuntu-latest
         target: aarch64-unknown-linux-gnu # New!
   ```

2. Ensure `scripts/ci_tasks.py` handles the platform
3. Test locally if possible

### Debugging Workflow Issues

```bash
# Check workflow syntax
actionlint .github/workflows/*.yml

# Validate with act (local GitHub Actions runner)
act -l  # List jobs
act -j test-rust  # Run specific job
```

### Monitoring

- **Job Duration**: Check for slowdowns over time
- **Cache Hit Rate**: Monitor cache effectiveness
- **Failure Rate**: Track flaky tests
- **Artifact Sizes**: Ensure they stay within limits

### Security

- All workflows use pinned action versions
- `dependabot-automerge.yml` only auto-merges patch/minor
- `dependency-review.yml` scans for vulnerabilities
- `security` job runs `cargo audit`

---

## Size Limits Reference

| Item              | Limit | Rationale                           |
| ----------------- | ----- | ----------------------------------- |
| CLI Binary        | 50 MB | GitHub release size, download speed |
| WASM Bundle       | 1 MB  | Browser download, initial load time |
| Packaged Artifact | 70 MB | Compressed archive overhead         |

---

## Workflow Triggers

### CI (`ci.yml`)

- Push to `main`, `dev`, `release/**`
- Pull requests to `main`, `dev`

### Release (`release.yml`)

- Tags matching `v*.*.*`

### Publish Python (`publish-python.yml`)

- Release published

### Dependabot Auto-merge (`dependabot-automerge.yml`)

- Dependabot PRs (opened, reopened, synchronized, labeled)
- PR reviews submitted

### Dependency Review (`dependency-review.yml`)

- Pull requests to `main`, `dev`

---

## Best Practices

1. **Test locally first**: Use `just` commands before pushing
2. **Keep workflows DRY**: Use `scripts/ci_tasks.py` for complex logic
3. **Document changes**: Update this guide when adding new workflows
4. **Monitor performance**: Watch for slow jobs
5. **Version dependencies**: Pin action versions for reproducibility
6. **Use caching wisely**: Include relevant files in cache keys
7. **Fail fast**: Set appropriate timeouts on jobs
8. **Make errors actionable**: Use `::error::` annotations

---

## Getting Help

- **Workflow issues**: Check GitHub Actions logs
- **Local testing**: Run `just` commands with `-v` for verbose output
- **Script issues**: Run `python3 scripts/ci_tasks.py --help`
- **Pre-commit**: Run `pre-commit run --all-files --verbose`
