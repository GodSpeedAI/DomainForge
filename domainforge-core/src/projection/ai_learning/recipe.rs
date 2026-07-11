//! JSON recipe for AI learning projections. Strictly validated: unknown keys,
//! bad split ratios, or unknown family/task/format names are hard errors.

use serde::{Deserialize, Serialize};

pub const LLM_FAMILIES: &[&str] = &[
    "authorization_qa",
    "policy_explanation",
    "counterexample_detection",
    "entity_lookup",
];
pub const GRAPH_ML_TASKS: &[&str] = &[
    "node_classification",
    "link_prediction",
    "policy_violation_detection",
];
pub const CEP_FAMILIES: &[&str] = &["AuthorityBench", "ConformanceBench"];
pub const LLM_FORMATS: &[&str] = &["chatml_jsonl"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub name: String,
    /// Path to an AuthorityEnvironmentConfig JSON, relative to the recipe file.
    /// Required for any projection that needs resolver-grounded labels.
    #[serde(default)]
    pub authority_config: Option<String>,
    #[serde(default)]
    pub projections: Projections,
    #[serde(default)]
    pub splits: SplitRatios,
    #[serde(default)]
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Projections {
    #[serde(default)]
    pub llm_dataset: LlmProjection,
    #[serde(default)]
    pub graph_ml_dataset: GraphMlProjection,
    #[serde(default)]
    pub cep_eval_dataset: CepEvalProjection,
    /// BAML capability projection (`--format baml`). Only the fields BAML
    /// rendering actually needs live here — one recipe system, not two.
    #[serde(default)]
    pub baml: BamlProjection,
    /// DSPy optimization projection (`--format dspy`). Only the fields DSPy
    /// rendering actually needs live here — one recipe system, not two.
    #[serde(default)]
    pub dspy: DspyProjection,
    /// ZenML learning-pipeline projection (`--format zenml`). Only the fields
    /// ZenML rendering actually needs live here — one recipe system, not two.
    #[serde(default)]
    pub zenml: ZenmlProjection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZenmlProjection {
    /// Reserved for parity with the other projection sections; the `--format
    /// zenml` CLI always renders regardless of this flag (the format itself is
    /// the selector).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Name of the generated `@pipeline` function. Must be a valid Python
    /// identifier; defaults to `authority_learning_pipeline`.
    #[serde(default = "default_zenml_pipeline_name")]
    pub pipeline_name: String,
    /// Name of the ZenML Model the pipeline versions its runs under. Must be a
    /// valid Python identifier; defaults to `authority_decider`. The model
    /// *version* is keyed to the source model hash, not this name.
    #[serde(default = "default_zenml_model_name")]
    pub model_name: String,
    /// Minimum decision-agreement score (0.0..=1.0) the emitted `register`
    /// step's promotion gate requires before promoting a run's model version.
    /// Default 0.0 = always promote (no gate).
    #[serde(default)]
    pub promotion_threshold: f64,
    /// Include `unknown`-labeled examples (no applicable policy) in the loaded
    /// dataset. Off by default so examples stay policy-grounded — mirrors the
    /// DSPy `include_unknown` switch.
    #[serde(default)]
    pub include_unknown: bool,
}

fn default_zenml_pipeline_name() -> String {
    "authority_learning_pipeline".to_string()
}
fn default_zenml_model_name() -> String {
    "authority_decider".to_string()
}

impl Default for ZenmlProjection {
    fn default() -> Self {
        Self {
            enabled: true,
            pipeline_name: default_zenml_pipeline_name(),
            model_name: default_zenml_model_name(),
            promotion_threshold: 0.0,
            include_unknown: false,
        }
    }
}

/// Recognized DSPy module strategies (how the signature is wrapped into a
/// prediction module). `predict` is the v1 default; `chain_of_thought` is a
/// documented recipe option that lowers to `dspy.ChainOfThought`.
pub const DSPY_MODULE_STRATEGIES: &[&str] = &["predict", "chain_of_thought"];
/// Recognized DSPy teleprompters the emitted `optimize.py` can construct with
/// only the `dspy` package installed.
pub const DSPY_OPTIMIZERS: &[&str] = &["BootstrapFewShot", "LabeledFewShot"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DspyProjection {
    /// Reserved for parity with the other projection sections; the `--format
    /// dspy` CLI always renders regardless of this flag (the format itself is
    /// the selector).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Name of the generated DSPy signature class. Must be a valid Python
    /// identifier; defaults to `AuthorityDecision`.
    #[serde(default = "default_dspy_signature_name")]
    pub signature_name: String,
    /// Module strategy: `predict` (default) or `chain_of_thought`.
    #[serde(default = "default_dspy_module_strategy")]
    pub module_strategy: String,
    /// Teleprompter the emitted `optimize.py` constructs. One of
    /// [`DSPY_OPTIMIZERS`]; defaults to `BootstrapFewShot`.
    #[serde(default = "default_dspy_optimizer")]
    pub optimizer: String,
    /// Cap on self-bootstrapped few-shot demonstrations (BootstrapFewShot).
    #[serde(default = "default_dspy_bootstrapped_demos")]
    pub max_bootstrapped_demos: u32,
    /// Cap on labeled few-shot demonstrations drawn from the train split.
    #[serde(default = "default_dspy_labeled_demos")]
    pub max_labeled_demos: u32,
    /// Tolerated drop (0.0..=1.0) of the optimized dev score below the baseline
    /// dev score. The emitted `optimize.py` fails if
    /// `optimized < baseline - regression_threshold`. Default 0.0 = no
    /// regression tolerated.
    #[serde(default)]
    pub regression_threshold: f64,
    /// Include `unknown`-labeled examples (no applicable policy) in the DSPy
    /// train/dev sets. Off by default so examples stay policy-grounded — this
    /// mirrors the BAML `include_unknown` switch.
    #[serde(default)]
    pub include_unknown: bool,
}

fn default_dspy_signature_name() -> String {
    "AuthorityDecision".to_string()
}
fn default_dspy_module_strategy() -> String {
    "predict".to_string()
}
fn default_dspy_optimizer() -> String {
    "BootstrapFewShot".to_string()
}
fn default_dspy_bootstrapped_demos() -> u32 {
    4
}
fn default_dspy_labeled_demos() -> u32 {
    16
}

impl Default for DspyProjection {
    fn default() -> Self {
        Self {
            enabled: true,
            signature_name: default_dspy_signature_name(),
            module_strategy: default_dspy_module_strategy(),
            optimizer: default_dspy_optimizer(),
            max_bootstrapped_demos: default_dspy_bootstrapped_demos(),
            max_labeled_demos: default_dspy_labeled_demos(),
            regression_threshold: 0.0,
            include_unknown: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamlProjection {
    /// Reserved for parity with the other projection sections; the `--format
    /// baml` CLI always renders regardless of this flag (the format itself is
    /// the selector).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Name of the generated policy-decision function. Must be a valid BAML
    /// identifier; defaults to `DecideAuthority`.
    #[serde(default = "default_baml_function_name")]
    pub function_name: String,
    /// Emit a `test` block for tuples the resolver labels "unknown" (no
    /// applicable policy). Off by default so examples stay policy-grounded.
    #[serde(default)]
    pub include_unknown: bool,
}

fn default_baml_function_name() -> String {
    "DecideAuthority".to_string()
}

impl Default for BamlProjection {
    fn default() -> Self {
        Self {
            enabled: true,
            function_name: default_baml_function_name(),
            include_unknown: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmProjection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_llm_format")]
    pub format: String,
    #[serde(default = "default_llm_families")]
    pub families: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphMlProjection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_graph_tasks")]
    pub tasks: Vec<String>,
    /// Open-world safety: resolver Unknown/NotApplicable tuples are excluded
    /// from negative samples unless this is set.
    #[serde(default)]
    pub treat_unknown_as_negative: bool,
    /// Cap on negative samples, as a multiple of positive samples.
    #[serde(default = "default_negative_cap")]
    pub max_negative_ratio: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CepEvalProjection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cep_families")]
    pub families: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitRatios {
    pub train: f64,
    pub validation: f64,
    pub test: f64,
}

fn default_true() -> bool {
    true
}
fn default_llm_format() -> String {
    "chatml_jsonl".to_string()
}
fn default_llm_families() -> Vec<String> {
    LLM_FAMILIES.iter().map(|s| s.to_string()).collect()
}
fn default_graph_tasks() -> Vec<String> {
    GRAPH_ML_TASKS.iter().map(|s| s.to_string()).collect()
}
fn default_cep_families() -> Vec<String> {
    CEP_FAMILIES.iter().map(|s| s.to_string()).collect()
}
fn default_negative_cap() -> u32 {
    10
}

impl Default for LlmProjection {
    fn default() -> Self {
        Self {
            enabled: true,
            format: default_llm_format(),
            families: default_llm_families(),
        }
    }
}

impl Default for GraphMlProjection {
    fn default() -> Self {
        Self {
            enabled: true,
            tasks: default_graph_tasks(),
            treat_unknown_as_negative: false,
            max_negative_ratio: default_negative_cap(),
        }
    }
}

impl Default for CepEvalProjection {
    fn default() -> Self {
        Self {
            enabled: true,
            families: default_cep_families(),
        }
    }
}

impl Default for SplitRatios {
    fn default() -> Self {
        Self {
            train: 0.8,
            validation: 0.1,
            test: 0.1,
        }
    }
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            authority_config: None,
            projections: Projections::default(),
            splits: SplitRatios::default(),
            seed: 0,
        }
    }
}

impl Recipe {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let recipe: Recipe =
            serde_json::from_str(json).map_err(|e| format!("invalid recipe: {e}"))?;
        recipe.validate()?;
        Ok(recipe)
    }

    pub fn validate(&self) -> Result<(), String> {
        const EPSILON: f64 = 1e-9;
        let sum = self.splits.train + self.splits.validation + self.splits.test;
        if (sum - 1.0).abs() > EPSILON {
            return Err(format!("split ratios must sum to 1.0, got {sum}"));
        }
        for r in [self.splits.train, self.splits.validation, self.splits.test] {
            if !(0.0..=1.0).contains(&r) {
                return Err(format!("split ratio out of range: {r}"));
            }
        }
        check_known(
            "llm family",
            &self.projections.llm_dataset.families,
            LLM_FAMILIES,
        )?;
        check_known(
            "graph task",
            &self.projections.graph_ml_dataset.tasks,
            GRAPH_ML_TASKS,
        )?;
        check_known(
            "cep family",
            &self.projections.cep_eval_dataset.families,
            CEP_FAMILIES,
        )?;
        if !LLM_FORMATS.contains(&self.projections.llm_dataset.format.as_str()) {
            return Err(format!(
                "unknown llm format '{}' (supported: {})",
                self.projections.llm_dataset.format,
                LLM_FORMATS.join(", ")
            ));
        }
        let fname = &self.projections.baml.function_name;
        let valid_ident = !fname.is_empty()
            && fname
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && fname.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid_ident {
            return Err(format!(
                "baml function_name '{fname}' is not a valid BAML identifier \
                 (letters, digits, and underscore; must not start with a digit)"
            ));
        }
        let dspy = &self.projections.dspy;
        if !is_valid_py_identifier(&dspy.signature_name) {
            return Err(format!(
                "dspy signature_name '{}' is not a valid Python identifier \
                 (letters, digits, and underscore; must not start with a digit)",
                dspy.signature_name
            ));
        }
        if !DSPY_MODULE_STRATEGIES.contains(&dspy.module_strategy.as_str()) {
            return Err(format!(
                "unknown dspy module_strategy '{}' (supported: {})",
                dspy.module_strategy,
                DSPY_MODULE_STRATEGIES.join(", ")
            ));
        }
        if !DSPY_OPTIMIZERS.contains(&dspy.optimizer.as_str()) {
            return Err(format!(
                "unknown dspy optimizer '{}' (supported: {})",
                dspy.optimizer,
                DSPY_OPTIMIZERS.join(", ")
            ));
        }
        if !(0.0..=1.0).contains(&dspy.regression_threshold) {
            return Err(format!(
                "dspy regression_threshold out of range: {} (expected 0.0..=1.0)",
                dspy.regression_threshold
            ));
        }
        let zenml = &self.projections.zenml;
        if !is_valid_py_identifier(&zenml.pipeline_name) {
            return Err(format!(
                "zenml pipeline_name '{}' is not a valid Python identifier \
                 (letters, digits, and underscore; must not start with a digit)",
                zenml.pipeline_name
            ));
        }
        if !is_valid_py_identifier(&zenml.model_name) {
            return Err(format!(
                "zenml model_name '{}' is not a valid Python identifier \
                 (letters, digits, and underscore; must not start with a digit)",
                zenml.model_name
            ));
        }
        if !(0.0..=1.0).contains(&zenml.promotion_threshold) {
            return Err(format!(
                "zenml promotion_threshold out of range: {} (expected 0.0..=1.0)",
                zenml.promotion_threshold
            ));
        }
        Ok(())
    }
}

/// A valid Python identifier: non-empty, starts with a letter or underscore,
/// and contains only ASCII alphanumerics and underscores.
fn is_valid_py_identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn check_known(kind: &str, given: &[String], known: &[&str]) -> Result<(), String> {
    for name in given {
        if !known.contains(&name.as_str()) {
            return Err(format!(
                "unknown {kind} '{name}' (supported: {})",
                known.join(", ")
            ));
        }
    }
    Ok(())
}
