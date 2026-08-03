# Next Steps

## 1. Review and integrate I1-I3

Review the completed I1-I3 diff and verification record, then choose whether to
commit or open a pull request. Expected outcome: the implementation is
integrated without staging, reverting, or rewriting the pre-existing unrelated
worktree changes.

## 2. Restore model-checker-backed TLA evidence when available

Install Java and provide `tla2tools.jar`, then rerun `just prove`. Expected
outcome: TLA evidence upgrades from the recorded structural-only fallback to a
model-checker-backed result. This is an environment limitation, not a failing
I1-I3 behavior or test.

## 3. Continue the prior Milestone 1 contract gate

Resume ADR-014 ratification only after this I1-I3 change is reviewed. Do not
combine Milestone 1 implementation with this intervention diff.

## Prior milestone next steps (superseded while I1-I3 is active)

## 1. Ratify ADR-014

Review and explicitly accept or amend
`docs/specs/ADR-014-application-review-and-approval-contract.md`. Expected
outcome: fixed, human-approved CLI, artifact, status, diff, and approval
contracts with no implementation-agent choices left open.

## 2. Write and adversarially review the Milestone 1 plan

Create
`.agents/plans/2026-07-19-conversational-application-generator-m1-review.md`
from specification Milestone 1 and the remediated Milestone 0 APIs. Expected
outcome: a test-first, codebase-grounded plan for Domain/Application IR,
inspection, domain review, semantic diff, and semantic approval capture.

## 3. Implement and gate Milestone 1

Execute one independently testable packet at a time, preserving canonical
hashing and binding parity. Expected outcome: inspect/review/diff/approval
workflows satisfy an adversarial Milestone 1 human gate without entering
provider, generation, or skill scope; explicit maintainer ratification remains
required before Milestone 2.
