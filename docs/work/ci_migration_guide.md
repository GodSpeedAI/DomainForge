# CI/CD Optimization - Migration Guide

## Overview

This guide helps the team transition to the optimized CI/CD workflows and pre-commit hooks.

## What Changed?

### For All Developers

#### 1. Pre-commit Hooks Updated

**Action Required**: Update your pre-commit installation

```bash
# Update pre-commit hooks
pre-commit autoupdate

# Or reinstall from scratch
pre-commit clean
pre-commit install
```

**What's Different**:

- Rust hooks now use local `cargo` commands (faster, more reliable)
- Updated to latest versions of `black`, `prettier`, and `pre-commit-hooks`
- Now covers `scripts/` directory for Python formatting

#### 2. New `just` Recipes for CI

**Action Required**: None, but you can now test CI commands locally!

```bash
# These are the same commands CI runs
just ci-test-rust
just ci-test-python
just ci-test-ts
just ci-verify-binary target/debug/sea
```

**Benefit**: When CI fails, you can reproduce it locally instantly.

### For CI/CD Maintainers

#### 1. New Script: `scripts/ci_tasks.py`

**What**: Cross-platform CI automation script  
**Why**: Eliminates duplicate shell logic in workflows  
**Usage**: Called by `just` recipes and workflows

```bash
# Test it
python3 scripts/ci_tasks.py --help
```

#### 2. Workflows Refactored

**Files Changed**:

- `.github/workflows/ci.yml` - Now uses `just` commands
- `.github/workflows/release.yml` - Simplified packaging logic
- `.github/workflows/publish-python.yml` - Added timeout

**Key Changes**:

- All jobs now install `just` using `extractions/setup-just@v2`
- Size checks use `scripts/ci_tasks.py`
- Packaging uses `scripts/ci_tasks.py`
- Cache keys include `Cargo.lock` hash
- `CACHE_VERSION` bumped to v3

#### 3. Environment Variables Centralized

**Location**: Workflow `env` section

```yaml
env:
  CLI_MAX_BYTES: "52428800"
  WASM_MAX_BYTES: "1048576"
  CLI_ARTIFACT_MAX_BYTES: "73400320"
```

**To Change**: Edit the workflow file, not individual steps.

---

## Migration Checklist

### Immediate (Before Merge)

- [ ] Review this PR and all changes
- [ ] Verify workflows are valid YAML
- [ ] Test `scripts/ci_tasks.py` locally
- [ ] Run `just --list` to see new recipes

### After Merge

- [ ] All developers: Update pre-commit hooks
- [ ] Monitor first CI run for any issues
- [ ] Update team documentation if needed
- [ ] Consider adding to onboarding docs

### Optional

- [ ] Add unit tests for `scripts/ci_tasks.py`
- [ ] Set up workflow validation in pre-commit
- [ ] Create workflow diagrams for documentation

---

## Breaking Changes

**None!** All changes are backward compatible.

- Workflows still produce the same artifacts
- Artifact names haven't changed
- Release process is unchanged
- Pre-commit hooks run the same checks (just updated versions)

---

## Rollback Plan

If issues arise, you can rollback by:

1. **Revert the PR**: `git revert <commit-hash>`
2. **Or cherry-pick specific files**:

   ```bash
   git checkout HEAD~1 .github/workflows/ci.yml
   git checkout HEAD~1 .github/workflows/release.yml
   git checkout HEAD~1 .pre-commit-config.yaml
   ```

3. **Bump cache version** if caches are causing issues:
   ```yaml
   env:
     CACHE_VERSION: v4 # Invalidates all caches
   ```

---

## Testing Strategy

### Before Merge

- [x] Validate YAML syntax
- [x] Test `scripts/ci_tasks.py` locally
- [x] Verify `just` recipes work
- [ ] Run pre-commit on all files

### After Merge (Automatic)

- [ ] CI runs on the merge commit
- [ ] All jobs pass
- [ ] Artifacts are created correctly

### Manual Testing

- [ ] Create a test branch and push
- [ ] Verify all CI jobs pass
- [ ] Check job durations (should be similar or faster)
- [ ] Verify cache hit rates

---

## Common Issues & Solutions

### Issue: Pre-commit hooks fail after update

**Solution**:

```bash
pre-commit clean
pre-commit install
pre-commit run --all-files
```

### Issue: `just` command not found in CI

**Solution**: Already handled - workflows install `just` using `extractions/setup-just@v2`

### Issue: Cache misses after update

**Expected**: First run after merge will miss cache (new `CACHE_VERSION`)  
**Solution**: None needed, caches will rebuild automatically

### Issue: Binary size check fails

**Solution**: Check if binary actually grew, or adjust limit in workflow `env` section

### Issue: Python script fails on Windows

**Solution**: Script is tested cross-platform, but if issues arise:

1. Check Python version (should be 3.11+)
2. Check file paths (script uses `pathlib` for cross-platform paths)
3. Report issue with full error output

---

## Performance Expectations

### CI Job Durations

**Expected**: Similar or slightly faster due to better caching

| Job             | Before | After  | Change    |
| --------------- | ------ | ------ | --------- |
| Lint            | ~2 min | ~2 min | No change |
| Test Rust       | ~5 min | ~5 min | No change |
| Test Python     | ~3 min | ~3 min | No change |
| Test TypeScript | ~2 min | ~2 min | No change |

### Cache Hit Rates

**Expected**: Higher hit rates due to `Cargo.lock` in cache key

### First Run After Merge

**Expected**: Cache misses (new `CACHE_VERSION`), jobs will be slower  
**Subsequent Runs**: Normal speed with cache hits

---

## Documentation Updates

### New Documents

- `docs/work/ci_optimization_summary.md` - Implementation summary
- `docs/work/ci_findings_report.md` - Issues identified
- `docs/work/ci_quick_reference.md` - Developer reference
- This file - Migration guide

### Updated Files

- `justfile` - New CI recipes
- `.pre-commit-config.yaml` - Updated hooks
- `.github/workflows/ci.yml` - Refactored
- `.github/workflows/release.yml` - Refactored
- `.github/workflows/publish-python.yml` - Minor updates

### New Files

- `scripts/ci_tasks.py` - CI automation script

---

## Support

### Questions?

- Check `docs/work/ci_quick_reference.md` for common tasks
- Review `docs/work/ci_findings_report.md` for rationale
- Ask in team chat or create an issue

### Found a Bug?

1. Check if it's a known issue in this guide
2. Try the solutions in "Common Issues"
3. Create an issue with:
   - What you were trying to do
   - What happened
   - Full error output
   - Your environment (OS, Python version, etc.)

### Want to Contribute?

- Add tests for `scripts/ci_tasks.py`
- Improve documentation
- Add more `just` recipes for common tasks
- Optimize workflow performance

---

## Success Criteria

This migration is successful when:

- ✅ All CI jobs pass on main branch
- ✅ Developers can reproduce CI failures locally
- ✅ Pre-commit hooks work for all developers
- ✅ No increase in CI job durations
- ✅ Team understands new workflow structure

---

## Timeline

- **Day 1**: PR merged, caches rebuild
- **Day 2-3**: Team updates pre-commit hooks
- **Week 1**: Monitor CI performance
- **Week 2**: Gather feedback, make adjustments if needed
- **Month 1**: Evaluate success, document lessons learned

---

## Feedback

Please provide feedback on:

- Ease of migration
- Developer experience improvements
- Any issues encountered
- Suggestions for further improvements

Create an issue or discussion with your feedback!
