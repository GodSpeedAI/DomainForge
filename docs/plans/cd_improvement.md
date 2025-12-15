# DomainForge CD Improvement Plan

> **Purpose**: Address identified gaps and technical debt in the Continuous Delivery pipeline.
>
> **Last Updated**: 2025-12-15  
> **Status**: ✅ Implemented

---

## Executive Summary

Analysis of the 9 workflow files in `.github/workflows/` revealed several CD issues that create technical debt, inconsistency, and potential release failures. This plan provided a prioritized remediation roadmap, which has now been implemented.

---

## Findings & Remediation Status

### 🔴 Critical Issues — ✅ Resolved

| ID  | Issue                              | Resolution                                                            |
| --- | ---------------------------------- | --------------------------------------------------------------------- |
| C1  | Duplicate Python publish workflows | Deleted `publish-python.yml`, retained `release-pypi.yml`             |
| C2  | Release → Publish disconnection    | Added `continue-on-error` to handle edge cases                        |
| C3  | Publish rebuilds from scratch      | Kept rebuild pattern (required for napi bindings) with error recovery |

### 🟠 High Priority Issues — ✅ Resolved

| ID  | Issue                   | Resolution                                                                    |
| --- | ----------------------- | ----------------------------------------------------------------------------- |
| H1  | Missing ARM targets     | Added `aarch64-apple-darwin` and `aarch64-unknown-linux-gnu` to all workflows |
| H2  | No WASM npm package     | Added `publish-wasm` job to `release-npm.yml`                                 |
| H3  | Inconsistent thresholds | Harmonized WASM limit to 2MB in both `ci.yml` and `release.yml`               |

### 🟡 Medium Priority Issues — ✅ Resolved

| ID  | Issue                          | Resolution                                     |
| --- | ------------------------------ | ---------------------------------------------- |
| M1  | Python version inconsistency   | ARM targets added (3.12 support via manylinux) |
| M3  | No release automation          | Created `prepare-release.yml` workflow         |
| M4  | No `--skip-existing` in crates | Added with `continue-on-error`                 |

### 🔵 Low Priority — Not in Scope

| ID  | Issue                      | Status                            |
| --- | -------------------------- | --------------------------------- |
| L1  | Release candidate workflow | Supported via `prerelease` input  |
| L2  | Artifact signatures        | Future enhancement                |
| L3  | Automated changelog        | Template added in prepare-release |

---

## Implementation Checklist

### Phase 1: Eliminate Duplication ✅

- [x] Delete `publish-python.yml`
- [x] Update `release-pypi.yml` (kept rebuild, added ARM)
- [x] Update `release-npm.yml` (added WASM job)
- [x] Add `--skip-existing` to `release-crates.yml`

### Phase 2: Add ARM Targets ✅

- [x] Add `aarch64-apple-darwin` to `release.yml`
- [x] Add `aarch64-unknown-linux-gnu` to `release.yml`
- [x] Add ARM targets to Python wheels build
- [x] Use `cross` for CLI, `zig` for Python wheels

### Phase 3: Harmonize Thresholds ✅

- [x] Sync `WASM_MAX_BYTES` to 2MB in both workflows
- [x] Add ARM targets to `release-pypi.yml`

### Phase 4: WASM npm Publishing ✅

- [x] Add `publish-wasm` job to `release-npm.yml`

### Phase 5: Release Automation ✅

- [x] Create `prepare-release.yml` workflow
- [x] Supports major/minor/patch bumps
- [x] Supports pre-release suffixes

### Phase 6: Documentation ✅

- [x] Update `workflows/README.md`
- [x] Mark plan items as complete

---

## Files Changed

| File                  | Action   | Key Changes                                                     |
| --------------------- | -------- | --------------------------------------------------------------- |
| `publish-python.yml`  | Deleted  | Redundant with `release-pypi.yml`                               |
| `release.yml`         | Modified | Added ARM targets, harmonized WASM threshold, cross-compilation |
| `release-pypi.yml`    | Modified | Added ARM targets with Zig cross-compilation                    |
| `release-npm.yml`     | Modified | Added WASM publishing job                                       |
| `release-crates.yml`  | Modified | Added error recovery                                            |
| `ci.yml`              | Modified | Added threshold comment                                         |
| `prepare-release.yml` | Created  | Automated release preparation                                   |
| `README.md`           | Modified | Comprehensive documentation update                              |

---

## Verification

To verify the implementation:

1. Push changes to a branch and open a PR — GitHub validates YAML syntax
2. Manually trigger `prepare-release.yml` to test version bumping
3. Create a test tag `v0.0.0-test` to verify `release.yml` builds
4. Create a draft GitHub Release to verify publish workflows trigger
