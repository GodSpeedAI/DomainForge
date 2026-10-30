# GitHub Workflows

DomainForge uses one repository version across Rust, Python, TypeScript/native Node bindings, and WASM.

## Release model

`release-please.yml` watches `main` and maintains a single Release Please PR. The canonical version is `version.txt`; Release Please synchronizes that version into:

- `domainforge-core/Cargo.toml`
- `domainforge-python/pyproject.toml`
- `domainforge-typescript/package.json`

Merging the Release Please PR creates exactly one tag and one GitHub Release:

```
vX.Y.Z
```

The release title is normalized to the tag itself, so the repository sidebar shows the semantic version rather than a language/component name.

The migration baseline is v0.18.0. Older component-prefixed releases remain as historical records, but future releases use the unified tag.

## Publishing

A `vX.Y.Z` tag triggers `deploy.yml`. After validating the tag, the workflow dispatches all registry publishers from the same source commit:

| Workflow | Distribution |
| --- | --- |
| `release-crates.yml` | `domainforge-core` on crates.io |
| `release-pypi.yml` | `domainforge` on PyPI |
| `release-npm.yml` | `@godspeedai/domainforge` and `@godspeedai/domainforge-wasm` on npm |
| `package.yml` | native CLI `.deb`/`.tar.gz`/`.pkg`/`.dmg`/`.zip`/`setup.exe` on the GitHub Release |

The registry publishers are idempotent where the registry supports checking/skipping an already-published version.

`package.yml` builds the standalone CLI on Ubuntu 22.04 amd64 and arm64, macOS
(universal, with an Intel fallback), and Windows x86_64. Each package is
installed and exercised on the runner that built it before `deploy.yml` attaches
it. A packaging failure means no native assets are attached; the deploy run
fails instead of publishing a partial binary set. Packages are unsigned; see
`docs/distribution/native-cli.md` for the full matrix, checksums, and signing
status.

There is no GitHub Environment named `prod` in the release path. CI on `main` is the merge gate; the version tag is the publication signal.

## npm authentication

npm publication uses Trusted Publishing (OIDC), not a long-lived npm write token. The workflow requests `id-token: write` and uses npm 11.15.0 or newer.

Because `release-npm.yml` is invoked through `workflow_call`, npm validates the calling workflow. Configure each npm package's Trusted Publisher as:

- Provider: GitHub Actions
- Organization/user: `GodSpeedAI`
- Repository: `DomainForge`
- Workflow filename: `deploy.yml`
- Environment: leave blank
- Allowed action: direct `npm publish`

Configure this for both `@godspeedai/domainforge` and `@godspeedai/domainforge-wasm`. A package must exist on npm before its Trusted Publisher can be configured, so a never-published package needs one initial authenticated publish.

For an existing package, the CLI equivalent is:

```bash
npm install -g npm@^11.15.0
npm trust github @godspeedai/domainforge --file deploy.yml --repo GodSpeedAI/DomainForge --allow-publish
```

After `@godspeedai/domainforge-wasm` has been published once, configure it the same way by replacing the package name in that command.

## Release sequence

1. Merge normal conventional commits to `main`.
2. Release Please opens or updates the release PR and chooses the SemVer bump.
3. Review and merge that release PR.
4. Release Please creates `vX.Y.Z` and the GitHub Release.
5. `deploy.yml` publishes that same version to crates.io, PyPI, and npm.
6. `deploy.yml` also builds, install-tests, checksums, and attaches the native
   CLI packages to that same GitHub Release. Only a fully successful package
   matrix is attached.

`package.yml` can also be dispatched manually or runs on pushes to `main` and
`feat/native-cli-distribution`. Those runs upload workflow artifacts
(release candidates) but never create or modify a GitHub Release.

Do not manually create component-prefixed tags. The former manual `prepare-release.yml` path has been removed to keep one version authority and one release path.

## Required credentials

| Credential | Purpose |
| --- | --- |
| `CREATE_PR_TOKEN` | Allows Release Please-created tags to trigger `deploy.yml` |
| `SOPS_AGE_KEY` | Decrypts the existing crates.io/PyPI publication credentials |
| npm Trusted Publisher | Short-lived OIDC authentication for npm; no npm write token stored in GitHub |

## CI

`ci.yml` remains the merge gate for the repository. It includes a
`package-linux` job that builds the `.deb` and tarball on Ubuntu 22.04, verifies
the ELF/glibc baseline, installs the package, runs the functional smoke test,
and repeats the install inside clean Ubuntu 22.04 and Debian 12 containers.

For local validation, run `just all-tests` plus the language-specific
build/test commands relevant to the change.
