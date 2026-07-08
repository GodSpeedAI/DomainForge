//! DSPy optimization IR: the intermediate representation the renderer lowers to
//! a runnable DSPy program. Built entirely from an [`AiLearningContext`] so DSPy
//! reuses the same resolver-grounded authority cases, policy information, and
//! deterministic dataset split the ai-learning datasets are built from.
//!
//! The projection emits a *program plus config*; it never runs optimization
//! (that is the user's compute). Example data is never copied — the config
//! references the exact relative paths the ai-learning LLM-dataset projection
//! emits for the same fixture, so DSPy examples and the LLM dataset can never
//! diverge.
//!
//! Every id is minted through [`crate::projection::ids`]; all collections are
//! built in sorted order (BTreeMap/BTreeSet only) so the rendered output is
//! byte-deterministic for a fixed `created_at`.

use crate::projection::ai_learning::provenance::Provenance;
use crate::projection::ai_learning::split::{TRAIN, VALIDATION};
use crate::projection::ai_learning::AiLearningContext;
use crate::projection::ids::element_id;
use std::collections::BTreeSet;

/// Family tag for [`element_id`]: keeps DSPy ids disjoint from every other
/// projection family's ids for the same part tuple.
pub const DSPY_FAMILY: &str = "dspy";

/// Pinned DSPy Python API target. The generated program is written against this
/// release line. Bump procedure: change this constant, regenerate the fixture,
/// and confirm the `verify-dspy` CI job (Python `ast.parse` well-formedness
/// check) stays green. See `docs/dspy-projections.md` for what is and is not
/// live-validated.
pub const DSPY_TARGET_VERSION: &str = "2.5";

/// The ai-learning LLM-dataset subdirectory in a `--format ai-learning` run.
/// DSPy references the split files under this prefix so the referenced paths
/// are byte-identical to what `ai_learning::llm::emit` writes.
pub const LLM_DATASET_SUBDIR: &str = "llm_dataset";

/// The single ai-learning family DSPy draws labeled examples from: the
/// authorization-QA rows carry a resolver decision label per row.
pub const SOURCE_FAMILY: &str = "authorization_qa";

/// Reference to the ai-learning dataset files DSPy loads examples from. These
/// are *paths*, never copied data: `root` is where the user generated the
/// ai-learning projection (`--format ai-learning`), and `train`/`dev` are the
/// exact relative paths under it that `ai_learning::llm::emit` produces.
#[derive(Debug, Clone)]
pub struct DatasetRef {
    /// Default location of the sibling ai-learning output directory.
    pub root: String,
    /// Relative path (under `root`) of the train split JSONL.
    pub train: String,
    /// Relative path (under `root`) of the dev (validation) split JSONL.
    pub dev: String,
    /// The ai-learning family the examples are filtered to.
    pub family: String,
}

/// Optimizer (teleprompter) configuration plus the declared regression gate the
/// emitted `optimize.py` enforces.
#[derive(Debug, Clone)]
pub struct OptimizerSpec {
    pub name: String,
    pub max_bootstrapped_demos: u32,
    pub max_labeled_demos: u32,
    /// Tolerated drop of optimized dev score below baseline dev score.
    pub regression_threshold: f64,
}

/// The whole DSPy optimization capability: one authority-decision signature, a
/// module strategy, the dataset references, the metric, the optimizer config,
/// and the provenance stamp — everything a renderer needs.
#[derive(Debug, Clone)]
pub struct AIOptimizationIR {
    pub model_ref: String,
    pub model_hash: String,
    pub created_at: String,
    pub generator_version: String,
    /// Deterministic id for the capability (signature) itself.
    pub capability_id: String,
    pub signature_name: String,
    /// Module class name derived from the signature name.
    pub program_name: String,
    /// `predict` | `chain_of_thought`.
    pub module_strategy: String,
    /// The closed set of decision labels the signature output ranges over,
    /// sorted (mirrors the ai-learning `decision_label` vocabulary).
    pub decision_labels: Vec<String>,
    /// One human-readable line per authority policy, embedded in the signature
    /// docstring so the program carries the policy semantics it must respect.
    pub policy_lines: Vec<String>,
    /// Metric function name (decision-label agreement).
    pub metric_name: String,
    pub dataset: DatasetRef,
    pub optimizer: OptimizerSpec,
    /// Artifact-version stamp reused verbatim from the ai-learning provenance.
    pub provenance: Provenance,
}

impl AIOptimizationIR {
    /// Lower an [`AiLearningContext`] into the DSPy optimization IR. Requires a
    /// resolver-grounded authority environment: without it there are no
    /// policy-decision capabilities to optimize and no labeled examples.
    pub fn build(ctx: &AiLearningContext) -> Result<Self, String> {
        ctx.require_authority("dspy")?;
        let cfg = &ctx.recipe.projections.dspy;

        // The authorization-QA family is what supplies labeled examples; refuse
        // to emit a program with no decidable examples behind it.
        let has_decisive = ctx
            .authority_cases
            .iter()
            .any(|c| cfg.include_unknown || c.label != "unknown");
        if !has_decisive {
            return Err(
                "dspy projection requires at least one decisive authority case (allow/deny/\
                 escalate); the provided authority environment produced only 'unknown' tuples"
                    .to_string(),
            );
        }

        // Closed decision vocabulary, sorted for determinism. Matches the labels
        // the ai-learning datasets attach to authorization_qa rows.
        let decision_labels: Vec<String> = ["allow", "deny", "escalate", "reject", "unknown"]
            .iter()
            .filter(|l| cfg.include_unknown || **l != "unknown")
            .map(|l| l.to_string())
            .collect();

        // Policy semantics for the signature docstring, deterministic via the
        // BTreeMap-backed policy_info (reused, never re-derived).
        let policy_lines = ctx
            .policy_info
            .values()
            .map(|p| {
                let scope = [
                    p.actor_role.as_deref().map(|r| format!("role={r}")),
                    p.action.as_deref().map(|a| format!("action={a}")),
                    p.resource_type.as_deref().map(|r| format!("resource={r}")),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
                let desc = p.description.as_deref().unwrap_or("");
                format!(
                    "{} [{}] (priority {}) applies to {{ {} }}{}",
                    p.policy_id,
                    p.modality,
                    p.priority,
                    scope,
                    if desc.is_empty() {
                        String::new()
                    } else {
                        format!(": {desc}")
                    }
                )
            })
            .collect();

        // Sanity: the operation vocabulary must be non-empty (same guard BAML
        // uses), otherwise the capability has nothing to decide over.
        let operations: BTreeSet<&str> = ctx
            .authority_cases
            .iter()
            .map(|c| c.operation.as_str())
            .collect();
        if operations.is_empty() {
            return Err(
                "dspy projection requires at least one authority policy action; the provided \
                 authority environment declares none"
                    .to_string(),
            );
        }

        let signature_name = cfg.signature_name.clone();
        let program_name = format!("{signature_name}Program");
        let capability_id = element_id(DSPY_FAMILY, &["capability", &signature_name]);

        let dataset = DatasetRef {
            root: "../ai_learning".to_string(),
            train: format!("{LLM_DATASET_SUBDIR}/{TRAIN}.jsonl"),
            dev: format!("{LLM_DATASET_SUBDIR}/{VALIDATION}.jsonl"),
            family: SOURCE_FAMILY.to_string(),
        };
        let optimizer = OptimizerSpec {
            name: cfg.optimizer.clone(),
            max_bootstrapped_demos: cfg.max_bootstrapped_demos,
            max_labeled_demos: cfg.max_labeled_demos,
            regression_threshold: cfg.regression_threshold,
        };

        let provenance = ctx.provenance(
            ctx.policy_info.keys().cloned().collect(),
            "authority_resolver",
            "unreviewed",
        );

        Ok(Self {
            model_ref: ctx.model_ref.clone(),
            model_hash: ctx.model_hash.clone(),
            created_at: ctx.created_at.clone(),
            generator_version: crate::projection::ai_learning::GENERATOR_VERSION.to_string(),
            capability_id,
            signature_name,
            program_name,
            module_strategy: cfg.module_strategy.clone(),
            decision_labels,
            policy_lines,
            metric_name: "decision_match".to_string(),
            dataset,
            optimizer,
            provenance,
        })
    }

    /// The DSPy module class for the configured strategy.
    pub fn dspy_module_call(&self) -> &'static str {
        match self.module_strategy.as_str() {
            "chain_of_thought" => "dspy.ChainOfThought",
            _ => "dspy.Predict",
        }
    }
}
