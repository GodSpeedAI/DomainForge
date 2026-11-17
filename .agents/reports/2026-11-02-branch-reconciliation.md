# Branch reconciliation: only `main` left locally

Date: 2026-11-02. Operator: Sam Prime. Scope: reconcile local and remote branches and worktrees into `main`, delete stale work, and record what was kept.

## Result

- Local branches: only `main` (at `f90905a`, tracking `origin/main`).
- Worktrees: only the primary checkout `/home/sprime01/projects/domainforge`.
- Remote branches left: `origin/main`, `origin/agent/projection-families` (draft PR #114), eight `origin/dependabot/*` branches backing open PRs #154–#163, and `origin/release-please--branches--main--components--domainforge` (backs open release PR #165).
- Stashes: 7 remain, untouched.
- Your uncommitted edits in the primary checkout (`.claude/helpers/graft-hooks.cjs`, `.claude/helpers/graft-statusline.cjs`, `.claude/settings.json`, `.claude/skills/graft/SKILL.md`, `AGENTS.md`) are unchanged. They match `origin/main` on those paths, so switching branches did not touch them.

## What was merged

- **PR #168** (`merge/egm-drift-report`, rebase merge into `main` as `f90905a`): the EGM drift report (`.agents/reports/2026-10-30-egm-drift-report.md`), the root blueprint count correction in `EGM.md`, and a D-doc-egm-3 entry in `.agents/OBSERVED_DEBT.md`.
  - The D-doc-egm-3 entry is marked PARTIAL. The dead `.sea/interaction/README.md` reference is gone from `main`. The open gap is that `to_graph` drops the `sea:EntityInstance` and `sea:Policy` triples that `to_turtle` emits.
  - The original commit (`5b62281`) had a conflict in the ledger, so the entry was rewritten during the cherry-pick. The original is recoverable from the reflog.
- **PR #166** (the EGM contracts branch, `docs/egm-architectural-contracts`): closed as superseded by #168. Its explainer and EGM copies were dropped because `main` already has the XSS fix and the EGM contracts.

## Checked and not needed

- `feat/native-cli-distribution` (`53ae189`, `b36f233`): the CodeQL workflow fix in `53ae189` is already on `main`, byte-identical in `.github`. `b36f233` only updated a session-state note that `main` has moved past.
- `feat/cep-snapshot-world-ref`, `feat/world-ref`, `fix/formatter-quantifier-comments`, `fix/observed-debt-verified`, `fix/openshell-supervisor-baseline`: their PRs (#132, #133, #130, #167, #162) were squash-merged into `main`. Feature checks confirmed the content is there.
- `release-please--*` and `rp-goldens-0.19.0`: release bot branches, superseded by tags `v0.18.2`, `v0.19.0`, `v0.21.2`. `main` is at version `0.21.2`. The golden-file content in `df-release` is identical to `main`.

## Deleted (recovery SHAs)

Recover any branch with `git branch <name> <sha>`. Reflog entries last about 90 days.

| Branch | Last SHA | Reason |
|---|---|---|
| `fix/observed-debt-verified` | `7d546a2` | PR #167 merged |
| `fix/openshell-supervisor-baseline` | `634f207` | PR #162 merged |
| `feat/world-ref` | `a3a4157` | PR #132 merged |
| `feat/cep-snapshot-world-ref` | `c4a2204` | PR #133 merged |
| `fix/formatter-quantifier-comments` | `aceffa2` | PR #130 merged |
| `feat/native-cli-distribution` | `53ae189` | PR #164 merged; fix already on `main` |
| `release-please--branches--main` | `81d1f59` | release branch, superseded |
| `release-please--branches--main--components--domainforge` (local) | `e010bec` | release branch, superseded |
| `rp-goldens-0.19.0` | `2e8f4eb` | release branch, superseded |
| `docs/egm-architectural-contracts` | `5b62281` | merged via #168 (rewritten) |
| `merge/egm-drift-report` | `1ae31d0` | merged via #168 |
| `chore/native-cli-security-fix` | `dd3db83` | empty temporary branch |

Remote branches deleted: `docs/egm-architectural-contracts`, `merge/egm-drift-report`. Remote refs for the merged observed-debt, OpenShell, and native-CLI branches, plus several `chore/*`, `codex/*`, `cycle/*`, `fix/*`, `remediation`, and `release/0.8.0` branches, were already absent after `git fetch --prune`. I did not delete those on the remote.

Worktrees removed (all clean, no uncommitted work): `_wt/df-v0212` (`d8cdea5`), `gsr-heads/domainforge` (`6e941ff`), `_wt/df-release` (`75b3e74`), `_wt/df-baseline` (`634f207`), `_wt/df-debt-fixes` (`7d546a2`), and two temporary worktrees I created (`_wt/df-egm`, `_wt/df-native-fix`).

## Decisions made without asking

1. Used a rebase merge for PR #168 because this repo rejects merge commits. This matches the earlier PR #167 merge.
2. Closed PR #166 instead of merging it. Its conflicts were in files `main` already superseded, and it had a failing CodeQL check on the pre-fix explainer.
3. Left the dependabot PRs and `agent/projection-families` open. The dependabot PRs fail MSRV and other required checks. The projection-families draft has 422 commits not on `main` and may be real work.
4. Did not delete the open release-please branch, because PR #165 uses it.
5. Did not commit this report. It is an uncommitted file in `.agents/reports/`.

## Open items for you

- **Dependabot PRs #154–#163:** failing MSRV and some lint/dependency-review checks. Decide whether to fix, rebase, or close each.
- **`origin/agent/projection-families` (draft PR #114):** 422 commits not on `main`. Decide whether to salvage or delete.
- **D-doc-egm-3 functional gap:** `to_graph` drops `sea:EntityInstance` and `sea:Policy` triples. Needs a test, then either a reconstruction or a documented omission.
- **Stashes:** 7 remain. Dropping them needs your OK.

## Gateway check (`just gateway-check`, read-only)

Run on 2026-11-02 as account `sprime01`. Stage `/home/sprime01/.cache/godspeed/gateway-stage`, digest `f425fc54d153cdd5be47854dbbd8c3a8b36f6261ff264ff85ba5e840a5bbdd98`. Result: **5 problems**, recipe exit 1.

Passing: both accounts exist; both are in `godspeed-sock`; `/usr/local/lib/godspeed-sea-forge` and its `bin` are `root:root 755`; `/usr/local/lib/godspeed-horizon/bin` is `root:root 755`; `/run/godspeed-sea-forge` is `godspeed-sea-forge:godspeed-sock 750`; `/var/lib/godspeed-sea-forge` is `godspeed-sea-forge:godspeed-sea-forge 1700`; `/var/lib/godspeed-horizon` is `godspeed-horizon:godspeed-horizon 700`; `bun` and `sea-forge-server` binaries are `root:root 755`; `/opt/godspeed-horizon` matches its manifest; both systemd units are `root:root 644`.

Problems:
1. `/var/lib/godspeed-sea-forge/realitytrace/data` missing (expected `godspeed-sea-forge:godspeed-sea-forge 700`).
2. `/var/lib/godspeed-sea-forge/realitytrace/state` missing (expected `godspeed-sea-forge:godspeed-sea-forge 700`).
3. `/var/lib/godspeed-sea-forge/server.yaml` missing (expected `root:godspeed-sock 640`).
4. `/var/lib/godspeed-sea-forge/policy.yaml` missing (expected `root:godspeed-sock 640`).
5. `/var/lib/godspeed-sea-forge/status.json` missing (expected `root:godspeed-sock 640`).

Notes: `realitytrace` is not installed by this installer, so its absence is expected. The socket check was skipped because `/run/godspeed-sea-forge/sea-forge.sock` is absent or `python3` is missing.

Not verified: whether the installer creates the five missing paths. The check recipe is read-only and does not create them. I did not run the installer or change any of these paths, since that needs root and was not requested. The next step is to decide whether to run the installer for those paths, then rerun `just gateway-check`.
