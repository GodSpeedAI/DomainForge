//! DSPy projection: compile a `.sea` model plus its authority environment into
//! a runnable [DSPy](https://dspy.ai/) optimization program — an
//! authority-decision signature, a metric measuring decision-label agreement,
//! and an optimizer script with a declared regression gate.
//!
//! Design rule (shared with the ai-learning family): no artifact invents domain
//! facts, and no artifact copies dataset rows. The train/dev examples are the
//! ai-learning LLM dataset's own splits, *referenced by path* — the same
//! [`AiLearningContext`] the datasets are built from drives this projection, so
//! DSPy examples and the LLM dataset can never diverge.
//!
//! The projection emits *program + config*; it never runs optimization (the
//! user's compute). The baseline-vs-optimized comparison and the
//! regression-threshold enforcement live in the emitted `optimize.py`, not in
//! DomainForge.
//!
//! Output is byte-deterministic for a fixed `created_at`: the IR is built in
//! sorted order (BTreeMap/BTreeSet only) and every id is minted through
//! `projection::ids`.

pub mod ir;
pub mod render;

use crate::graph::Graph;
use crate::projection::ai_learning::{recipe::Recipe, AiLearningContext, ArtifactSink};
use ir::AIOptimizationIR;
use std::collections::BTreeMap;

/// Emit the DSPy program into `sink`; returns the emitted relative paths.
///
/// Requires the context to carry a resolver-grounded authority environment
/// (`AiLearningContext::require_authority`): the signature and its examples are
/// grounded in the authority pack's decisions.
pub fn emit(ctx: &AiLearningContext, sink: &mut ArtifactSink) -> Result<Vec<String>, String> {
    let capability = AIOptimizationIR::build(ctx)?;

    let files: Vec<(&str, String)> = vec![
        ("program.py", render::program(&capability)),
        ("metric.py", render::metric(&capability)),
        ("optimize.py", render::optimize(&capability)),
        ("dspy.config.json", render::config(&capability)?),
        ("README.md", render::readme(&capability)),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (rel, content) in files {
        sink.write(rel, &content)?;
        written.push(rel.to_string());
    }
    Ok(written)
}

/// Binding surface: JSON in, path→content map out, no filesystem. Mirrors
/// `project_baml_in_memory` — the recipe supplies the `dspy` section and the
/// authority environment is passed explicitly (the recipe's `authority_config`
/// path is not resolved in-memory).
pub fn project_dspy_in_memory(
    graph: &Graph,
    recipe_json: Option<&str>,
    authority_config_json: Option<&str>,
    model_ref: &str,
    seed: Option<u64>,
    created_at: Option<String>,
) -> Result<BTreeMap<String, String>, String> {
    let recipe = match recipe_json {
        Some(json) => Recipe::from_json(json)?,
        None => Recipe::default(),
    };
    let authority = match authority_config_json {
        Some(json) => {
            let config: crate::authority::AuthorityEnvironmentConfig =
                serde_json::from_str(json).map_err(|e| format!("invalid authority config: {e}"))?;
            let mut env = crate::authority::AuthorityEnvironment::new(config)
                .map_err(|e| format!("failed to create authority environment: {e}"))?;
            env.validate()
                .map_err(|e| format!("authority environment validation failed: {e}"))?;
            Some(env)
        }
        None => None,
    };
    let ctx = AiLearningContext::build(
        graph,
        model_ref.to_string(),
        recipe,
        "<inline>".to_string(),
        seed,
        created_at,
        authority,
        authority_config_json.map(|_| "<inline>".to_string()),
    )?;

    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(&ctx, &mut sink)?;
    Ok(map)
}
