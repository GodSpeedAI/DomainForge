//! ZenML learning-pipeline IR: the intermediate representation the renderer
//! lowers to a runnable ZenML pipeline package. Built entirely from an
//! [`AiLearningContext`] so ZenML reuses the same resolver-grounded authority
//! cases, policy information, and deterministic dataset split the ai-learning
//! datasets are built from.
//!
//! This closes the learning loop the ai-learning datasets opened: dataset
//! emission (the `--format ai-learning` output) → a pipeline that *consumes*
//! that dataset (this projection) → metrics carried back into the run's model
//! version. The projection emits the *pipeline definition*; it never runs
//! training (that is the user's compute, on the user's ZenML stack).
//!
//! Example data is never copied — the config references the exact relative
//! paths the ai-learning LLM-dataset projection emits for the same fixture, so
//! the ZenML pipeline's inputs and the LLM dataset can never diverge.
//!
//! Every id is minted through [`crate::projection::ids`]; all collections are
//! built in sorted order (BTreeMap/BTreeSet only) so the rendered output is
//! byte-deterministic for a fixed `created_at`.

use crate::projection::ai_learning::provenance::Provenance;
use crate::projection::ai_learning::split::{TRAIN, VALIDATION};
use crate::projection::ai_learning::AiLearningContext;
use crate::projection::ids::element_id;
use std::collections::BTreeSet;

/// Family tag for [`element_id`]: keeps ZenML ids disjoint from every other
/// projection family's ids for the same part tuple.
pub const ZENML_FAMILY: &str = "zenml";

/// Pinned ZenML Python API target. The generated pipeline is written against
/// this release line's `@pipeline` / `@step` / `Model` decorator API. Bump
/// procedure: change this constant (and [`ZENML_PIP_SPEC`]), regenerate the
/// fixture, and confirm the `verify-zenml` CI job (Python `ast.parse`
/// well-formedness check) stays green. ZenML's decorator API has shifted
/// historically, so the projection targets a pinned line, period — see
/// `docs/zenml-projections.md`.
pub const ZENML_TARGET_VERSION: &str = "0.68";

/// The pip requirement the emitted `requirements.txt` pins ZenML to. Kept in
/// lockstep with [`ZENML_TARGET_VERSION`].
pub const ZENML_PIP_SPEC: &str = "zenml>=0.68,<0.69";

/// The ai-learning LLM-dataset subdirectory in a `--format ai-learning` run.
/// ZenML references the split files under this prefix so the referenced paths
/// are byte-identical to what `ai_learning::llm::emit` writes.
pub const LLM_DATASET_SUBDIR: &str = "llm_dataset";

/// The single ai-learning family ZenML draws labeled examples from: the
/// authorization-QA rows carry a resolver decision label per row.
pub const SOURCE_FAMILY: &str = "authorization_qa";

/// The decision-agreement metric name the pipeline computes and gates on. Named
/// to mirror the DSPy family's `decision_match` metric (single semantics across
/// the AI family).
pub const METRIC_NAME: &str = "decision_agreement";

/// Reference to the ai-learning dataset files the pipeline's `load_dataset`
/// step reads. These are *paths*, never copied data: `root` is where the user
/// generated the ai-learning projection (`--format ai-learning`), and
/// `train`/`dev` are the exact relative paths under it that
/// `ai_learning::llm::emit` produces.
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

/// One `@step` in the pipeline: its function name, the (sorted) input step
/// names it consumes, and the deterministic id its output artifact is named by.
#[derive(Debug, Clone)]
pub struct StepSpec {
    /// Python function name (a valid identifier).
    pub name: String,
    /// Names of the steps whose outputs feed this step, in DAG order.
    pub inputs: Vec<String>,
    /// Deterministic id naming this step's output artifact.
    pub artifact_id: String,
}

/// The whole ZenML learning capability: the dataset reference, the ordered
/// steps (load → train → evaluate → register) and their DAG edges, the metric,
/// the model-version key (= source model hash), the promotion gate, and the
/// provenance stamp — everything a renderer needs.
#[derive(Debug, Clone)]
pub struct LearningPipelineIR {
    pub model_ref: String,
    pub model_hash: String,
    pub created_at: String,
    pub generator_version: String,
    /// Deterministic id for the pipeline itself.
    pub pipeline_id: String,
    /// `@pipeline` function name.
    pub pipeline_name: String,
    /// ZenML Model name the runs are versioned under.
    pub model_name: String,
    /// ZenML Model *version* string, keyed to the source model hash so every
    /// run of the same model groups under one version.
    pub model_version: String,
    /// The closed set of decision labels the pipeline scores agreement over,
    /// sorted (mirrors the ai-learning `decision_label` vocabulary).
    pub decision_labels: Vec<String>,
    /// One human-readable line per authority policy, embedded in the README and
    /// the pipeline docstring so the package carries the policy semantics.
    pub policy_lines: Vec<String>,
    /// Metric the pipeline computes and the promotion gate reads.
    pub metric_name: String,
    /// Minimum metric score required to promote the run's model version.
    pub promotion_threshold: f64,
    pub dataset: DatasetRef,
    /// Ordered pipeline steps (load_dataset → train_model → evaluate_model →
    /// register_model) with their DAG edges.
    pub steps: Vec<StepSpec>,
    /// Artifact-version stamp reused verbatim from the ai-learning provenance.
    pub provenance: Provenance,
}

impl LearningPipelineIR {
    /// Lower an [`AiLearningContext`] into the ZenML learning-pipeline IR.
    /// Requires a resolver-grounded authority environment: without it there are
    /// no policy-decision capabilities to learn and no labeled examples.
    pub fn build(ctx: &AiLearningContext) -> Result<Self, String> {
        ctx.require_authority("zenml")?;
        let cfg = &ctx.recipe.projections.zenml;

        // The authorization-QA family is what supplies labeled examples; refuse
        // to emit a pipeline with no decidable examples behind it.
        let has_decisive = ctx
            .authority_cases
            .iter()
            .any(|c| cfg.include_unknown || c.label != "unknown");
        if !has_decisive {
            return Err(
                "zenml projection requires at least one decisive authority case (allow/deny/\
                 escalate); the provided authority environment produced only 'unknown' tuples"
                    .to_string(),
            );
        }

        // Sanity: the operation vocabulary must be non-empty (same guard DSPy
        // uses), otherwise the pipeline has nothing to learn to decide over.
        let operations: BTreeSet<&str> = ctx
            .authority_cases
            .iter()
            .map(|c| c.operation.as_str())
            .collect();
        if operations.is_empty() {
            return Err(
                "zenml projection requires at least one authority policy action; the provided \
                 authority environment declares none"
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

        // Policy semantics for the README / docstring, deterministic via the
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

        let pipeline_name = cfg.pipeline_name.clone();
        let pipeline_id = element_id(ZENML_FAMILY, &["pipeline", &pipeline_name]);

        // Model version keyed to the source model hash: strip the `sha256:`
        // prefix (a colon is not a legal ZenML version char) and prefix a `v`.
        let hash_digits = ctx
            .model_hash
            .rsplit(':')
            .next()
            .unwrap_or(ctx.model_hash.as_str());
        let model_version = format!("v-{hash_digits}");

        // The load → train → evaluate → register DAG. Each step's output
        // artifact is named by a deterministic id under the zenml family.
        let step = |name: &str, inputs: &[&str]| StepSpec {
            name: name.to_string(),
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
            artifact_id: element_id(ZENML_FAMILY, &["artifact", &pipeline_name, name]),
        };
        let steps = vec![
            step("load_dataset", &[]),
            step("train_model", &["load_dataset"]),
            step("evaluate_model", &["load_dataset", "train_model"]),
            step("register_model", &["train_model", "evaluate_model"]),
        ];

        let dataset = DatasetRef {
            root: "../ai_learning".to_string(),
            train: format!("{LLM_DATASET_SUBDIR}/{TRAIN}.jsonl"),
            dev: format!("{LLM_DATASET_SUBDIR}/{VALIDATION}.jsonl"),
            family: SOURCE_FAMILY.to_string(),
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
            pipeline_id,
            pipeline_name,
            model_name: cfg.model_name.clone(),
            model_version,
            decision_labels,
            policy_lines,
            metric_name: METRIC_NAME.to_string(),
            promotion_threshold: cfg.promotion_threshold,
            dataset,
            steps,
            provenance,
        })
    }

    /// DAG edges as `(from_step, to_step)` pairs, derived from each step's
    /// declared inputs. Sorted and deduplicated for deterministic rendering.
    pub fn dag_edges(&self) -> Vec<(String, String)> {
        let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
        for step in &self.steps {
            for input in &step.inputs {
                edges.insert((input.clone(), step.name.clone()));
            }
        }
        edges.into_iter().collect()
    }
}
