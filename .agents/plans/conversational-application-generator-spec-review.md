# Conversational Application Generator Spec Review Plan

Objective: replace the speculative v0.1.1 proposal with a repository-grounded,
implementable specification that preserves the intended non-developer workflow
without claiming semantics the current DSL and Domain IR do not contain.

## Checkpoints

- [x] Read repository instructions, current work state, the complete spec, and
      the review/writing skill contracts.
- [x] Verify the current CLI, grammar, Domain IR, projection pattern,
      `ArtifactSink`, Cell contracts, security policy, schemas, fixtures, and
      verification conventions in source.
- [x] Reconcile an independent adversarial review against the source evidence.
- [x] Revise the spec in place: define the semantic expressiveness gate, narrow
      the MVP, specify the user journey and approvals, settle trust and provider
      execution, define atomic output and proof semantics, and make milestones
      independently shippable.
- [x] Run structural and repository-reference checks; inspect the final diff;
      record findings and validation evidence in `.agents/current_state.md` and
      the next two or three actions in `.agents/next_steps.md`.

## Acceptance

- Every current-state claim cites a real repository path or is labelled an
  assumption/roadmap item.
- No generated behavior depends on invented domain semantics or arbitrary
  provider-authored executable code.
- The first release has a bounded vertical slice, explicit non-goals, measurable
  UX outcomes, failure states, security limits, and executable acceptance tests.
- Later production, provider-marketplace, UI, and general-capability claims are
  separated from MVP conformance.
