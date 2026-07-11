# Vendored CEP Specifications

Pinned copies of the normative CEP specs consumed by the CEP Evaluation
Dataset Projection (`domainforge-core/src/projection/ai_learning/cep_eval.rs`
and `schemas/cep_eval/`).

| File | Source | Pinned commit | cep_version |
| --- | --- | --- | --- |
| CEP-0008-semantic-envelope.md | github/local `cep` repo, `spec/` | `ae9b73b7526e307ed6b03595329114aa5413143c` | 0.1.0 |
| CEP-0009-benchmark-model.md | github/local `cep` repo, `spec/` | `ae9b73b7526e307ed6b03595329114aa5413143c` | 0.1.0 |

To update: copy the new spec files here, update the pin and `CEP_VERSION` in
`cep_eval.rs`, and re-derive `schemas/cep_eval/` from the new MUST-field lists
(CEP-0008 §12/§49.1/§52, CEP-0009 §13.1/§15.1/§16.1).
