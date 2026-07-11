# Evidence pack (generated)

This directory holds the machine- and human-readable proof that DomainForge's
claims hold. **It is not committed** (see `.gitignore`) — only this README is
tracked. Everything under `evidence/latest/` is regenerated from the current
worktree by:

```bash
just prove
```

## Layout

```
evidence/
  README.md                  # this file (tracked)
  latest/
    proof.json               # machine-readable evidence (merged fragments)
    proof.md                 # human-readable summary
    fragments/               # per-gate JSON fragments merged into proof.json
      language.json
      canonical.json
      projections.json
      roundtrip.json
      drift.json
      contracts.json
```

## What `just prove` runs

1. `rust-test` — unit/integration tests gate the evidence.
2. `prove-language` — parse/validate positive fixtures, reject negative
   fixtures, format round-trip stability (subset).
3. `prove-canonical` — two isolated RDF projections, byte-identical.
4. `prove-projections` — `scripts/verify/projection-targets/all.sh`
   (every projection gate) + projection-contract parity.
5. `prove-drift` — CALM + KG round-trip, and `pack diff` matching/drift cases.
6. `prove-evidence` — merge fragments, cross-check `PROOFS.md` claim statuses,
   render `proof.json` + `proof.md`, set `overall_result`.

`just prove` exits nonzero if any required proof fails. See `PROOFS.md` at the
repo root for the claim ledger (proven / partial / planned / blocked).
