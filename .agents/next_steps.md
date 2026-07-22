# Next Steps

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
