//! BAML projection: compile a `.sea` model plus its authority environment into
//! a [BAML](https://docs.boundaryml.com/) project — typed enums/classes for the
//! entity/resource/role domains, one typed policy-decision function, and a
//! `test` block per resolver-grounded authority case.
//!
//! Design rule (shared with the ai-learning family): no artifact invents domain
//! facts. The tests are the authority resolver's own decisions, reused from the
//! same [`AiLearningContext`] the datasets are built from — never re-derived.
//! The LLM client is a documented placeholder so generated code stays
//! vendor-neutral.
//!
//! Output is byte-deterministic for a fixed `created_at`: the IR is built in
//! sorted order (BTreeMap/BTreeSet only) and every id is minted through
//! `projection::ids`.

pub mod ir;
pub mod render;

use crate::graph::Graph;
use crate::projection::ai_learning::{recipe::Recipe, AiLearningContext, ArtifactSink};
use ir::AICapabilityIR;
use std::collections::BTreeMap;

/// Emit the BAML project into `sink`; returns the emitted relative paths.
///
/// Requires the context to carry a resolver-grounded authority environment
/// (`AiLearningContext::require_authority`): the function and its tests are the
/// authority pack's decisions.
pub fn emit(ctx: &AiLearningContext, sink: &mut ArtifactSink) -> Result<Vec<String>, String> {
    let cap = AICapabilityIR::build(ctx)?;

    let files: Vec<(&str, String)> = vec![
        ("baml_src/domain.baml", render::domain(&cap)),
        ("baml_src/functions.baml", render::functions(&cap)),
        ("baml_src/tests.baml", render::tests(&cap)),
        ("baml_src/clients.baml", render::clients(&cap)),
        ("README.md", render::readme(&cap)),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (rel, content) in files {
        sink.write(rel, &content)?;
        written.push(rel.to_string());
    }
    Ok(written)
}

/// Binding surface: JSON in, path→content map out, no filesystem. Mirrors
/// `project_ai_learning_in_memory` — the recipe supplies the `baml` section and
/// the authority environment is passed explicitly (the recipe's
/// `authority_config` path is not resolved in-memory).
pub fn project_baml_in_memory(
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
