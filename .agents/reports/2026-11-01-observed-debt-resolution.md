# Observed-debt resolution: decisions made without explicit instruction

Date: 2026-11-01. Branch: `fix/observed-debt-verified`, based on `main` (3f61ee4).
Commits (rollback points, oldest first):

- `eaa1be5` fix(core): resolve verified observed-debt items D1, D6, D-doc-egm-1..3
- `0da9cc0` test(core): multi-module D1 cases; honest D-doc-egm-1 guard
- ledger and report commit (see PR)

Each commit passed the repo's lefthook gates (`cargo fmt --check`, clippy `--all-targets --all-features -D warnings`, commitlint).

## Scope decisions

1. **Items fixed.** Only items that were verified against the code and reproduced on `main`:
   D1, D6, D-doc-egm-2, D-doc-egm-3, plus D-doc-egm-1 (hygiene, see below) and a partial D-serde-saphyr.
   Also added D-exit-1, which you raised mid-run.
2. **Items left alone.** I did not change the following:
   - D-native-1 (unsigned packages): you said to skip it.
   - D2, D4, D5, D11: process or policy calls, or they need maintainer action. Not mechanical.
   - D3 (drop stash@{0}): deletion needs your OK, and the stash is cheap to keep.
   - D12 (NULL semantics): choosing Kleene vs NULL-absorbing is a semantics decision, and it also changes the Rego projection.
   - D-native-2/3/4: need a runner or policy decision.
   - D-exit-1: see decision 9.
3. **Isolation.** I worked in a git worktree at `_wt/df-debt-fixes` on a branch from `main`. Your main checkout has uncommitted changes to `.claude/*` and `AGENTS.md`, which I did not touch. I also did not branch from the `docs/egm-architectural-contracts` branch, which is where the D-doc-egm-3 entry was logged.
4. **Merge method.** Merge commit, not squash. Squash would collapse the checkpoints you asked for, which are what let you roll back a single fix.

## Fix decisions

5. **D1 (`@version` on Graph).** The field is `Option<String>` with `skip_serializing_if = None`, so unversioned models keep the same JSON as before. The value comes from the entry module only, not from imports: resolved graphs merge namespaces, so there is no single "file version" otherwise. Justification: the header declares the version of the model file that was loaded, and the entry file is that file. I did not add it to the Devbox projection, because that would change emitted output and is a separate projection decision. Two tests cover this: one shows the entry version wins over an imported one, the other shows an imported version does not leak into an unversioned entry. A mutation (removing the propagation) fails two tests.
6. **D6 (CALM import).** Added a `Calm` variant to `ImportFormat` with an arm that calls `crate::calm::import`, mirroring the existing arms. Bad JSON exits 1 with a parse message.
7. **D-doc-egm-2 (`test` stub).** Chose to fail (exit 1, "not implemented") over implementing a runner or hiding the subcommand. Justification: a runner is a feature, and hiding it breaks `--help` discoverability. Failing loudly removes the false pass, which was the actual harm.
8. **D-doc-egm-1 (println).** The verifier found the line unreachable under the current grammar. The debt entry's impact claim was wrong. I removed the line anyway as dead-code hygiene and corrected the ledger entry. The test I added is a stdout-purity guard, not a regression proof. I say so in its comment.
9. **D-exit-1 (uniform exit code). Deferred, not implemented.** Justification: the top-level CLI returns `anyhow::Result`, so classifying errors needs typed errors across about 13 runners. Also, `pack.rs` already defines codes 1 to 6 with a different meaning, and `envelope.rs` uses 2 for I/O. Changing codes breaks scripts that depend on them. This is a contract change, so the code table should be chosen deliberately. Recommended table: 1 = validation/semantic failure, 2 = parse or usage error (matches pack's existing meaning), reserve the rest. Logged as an open entry.
10. **D-serde-saphyr (partial).**
    - Clippy was already clean on `main`; the flagged `map_or` calls were already `is_none_or`. Verified with the all-features clippy gate.
    - Bumped tokio 1.52.3 to 1.54.1 and spin 0.9.8 (yanked) to 0.9.9 in the lockfile. These clear the tokio unsound advisory and the spin yank.
    - Not fixed: quick-xml 0.37.5 (two advisories, fixed in 0.41). It is pinned by oxigraph 0.5.11. A lockfile-only oxigraph update does not move it. Fixing it needs an oxigraph upgrade, which may change SHACL/RDF behaviour, so I left it open.
    - The debt entry's original list of advisory crates (roxmltree) was stale; the current failing advisories are quick-xml.

## Verification process

- An independent adversarial verifier was run blind to my reasoning. It found that the D-doc-egm-1 test did not discriminate and that the D1 multi-module case was untested. Both are now addressed. The D-doc-egm-1 test is still a guard, not a regression proof.
- A second blind judge, given only the original debt descriptions and the verifier's evidence, marked D1, D-doc-egm-1 and D-serde-saphyr INSUFFICIENT. Responses:
  - D1: ran the full 8-test set and a mutation test.
  - D-doc-egm-1: corrected the ledger impact rather than claiming a fix.
  - D-serde: captured the final `cargo deny` output and lock versions, and recorded quick-xml as still open.
- The judge's rules were applied as written; I did not re-run the judge after these responses, so treat D1, D-doc-egm-1 and D-serde as "addressed, pending re-review" rather than independently re-confirmed.

## Open items after this change

- D-exit-1: uniform exit codes (decision 9).
- quick-xml 0.37.5 advisories (decision 10).
- D-doc-egm-3 is on the `docs/egm-architectural-contracts` branch's ledger, not this one. Its code fix (dead reference removed from `kg.rs`) is included here.
- Everything in the scope list above.
