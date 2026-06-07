use serde::{Deserialize, Serialize};

use super::compiler::{CompatibilityLoweringAuditor, PolicyCompiler};
use super::error::{AuthorityError, AuthorityErrorCode};
use super::fact_resolver::{FactResolver, FactSourceRegistry};
use super::pack::AuthorityPack;
use super::resolver::AuthorityResolver;
use super::trace::{AuthorityTrace, AuthorityTraceEmitter, EvidenceSink};
use super::transform::{DerivedFactEngine, FactTransformRegistry};
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityEnvironmentConfig {
    pub resolver_semantics_version: String,
    pub specificity_profile: SpecificityProfile,
    pub unknown_handling: UnknownHandlingConfig,
    pub fact_sources: Vec<FactSource>,
    pub fact_transforms: Vec<FactTransform>,
    pub authority_packs: Vec<serde_json::Value>,
    pub strict_mode: bool,
    pub compatibility_lowering_version: String,
    pub resolver_version: String,
}

pub struct AuthorityEnvironment {
    config: AuthorityEnvironmentConfig,
    source_registry: FactSourceRegistry,
    transform_registry: FactTransformRegistry,
    loaded_packs: Vec<AuthorityPack>,
    compiler: PolicyCompiler,
    compatibility_auditor: CompatibilityLoweringAuditor,
    resolver: AuthorityResolver,
    trace_emitter: AuthorityTraceEmitter,
    validated: bool,
}

impl AuthorityEnvironment {
    pub fn new(config: AuthorityEnvironmentConfig) -> Result<Self, AuthorityError> {
        let mut source_registry = FactSourceRegistry::new();
        for source in &config.fact_sources {
            source_registry.register(source.clone())?;
        }

        let mut transform_registry = FactTransformRegistry::new();
        for transform in &config.fact_transforms {
            transform_registry.register(transform.clone())?;
        }

        let mut loaded_packs = Vec::new();
        for pack_value in &config.authority_packs {
            let pack: AuthorityPack = serde_json::from_value(pack_value.clone()).map_err(|e| {
                AuthorityError::new(
                    AuthorityErrorCode::InvalidPolicyPack,
                    format!("Failed to parse pack: {}", e),
                )
            })?;
            pack.validate()?;
            pack.validate_semantics_version(&config.resolver_semantics_version)?;
            loaded_packs.push(pack);
        }

        let compiler = PolicyCompiler::new(
            config.resolver_semantics_version.clone(),
            config.compatibility_lowering_version.clone(),
        );
        let compatibility_auditor =
            CompatibilityLoweringAuditor::new(config.compatibility_lowering_version.clone());
        let resolver = AuthorityResolver::new(
            config.unknown_handling.clone(),
            config.specificity_profile.clone(),
            config.strict_mode,
            config.resolver_semantics_version.clone(),
            config.compatibility_lowering_version.clone(),
        );
        let trace_emitter = AuthorityTraceEmitter::new(
            config.resolver_version.clone(),
            config.resolver_semantics_version.clone(),
            config.compatibility_lowering_version.clone(),
            EvidenceSink::Memory,
        );

        Ok(Self {
            config,
            source_registry,
            transform_registry,
            loaded_packs,
            compiler,
            compatibility_auditor,
            resolver,
            trace_emitter,
            validated: false,
        })
    }

    pub fn validate(&mut self) -> Result<(), AuthorityError> {
        self.source_registry.validate()?;
        self.transform_registry.validate()?;

        for pack in &self.loaded_packs {
            if pack.required_specificity_profile != self.config.specificity_profile.id
                && self.config.strict_mode
            {
                return Err(AuthorityError::conflicting_specificity_profile(
                    &pack.id,
                    &self.config.specificity_profile.id,
                ));
            }
        }

        self.config.unknown_handling.validate()?;
        self.validated = true;
        Ok(())
    }

    pub fn evaluate(
        &self,
        request: &AuthorityRequest,
        provided_facts: &[FactEnvelope],
    ) -> Result<(AuthorityTrace, AuthorityDecision), AuthorityError> {
        if !self.validated {
            return Err(AuthorityError::invalid_environment(
                "Environment not validated. Call validate() first.",
            ));
        }

        request.validate()?;

        let fact_resolver = FactResolver::new(self.source_registry.clone());
        let now = chrono::Utc::now();
        let raw_facts = fact_resolver.wrap_context_as_caller_supplied(&request.context, now);
        let (all_facts, trust_decisions) =
            fact_resolver.resolve_trusted_facts(&raw_facts, provided_facts);

        let transform_keys: Vec<String> = self.config.fact_transforms
            .iter()
            .map(|t| format!("{}@{}", t.id, t.version))
            .collect();
        let derived_engine = DerivedFactEngine::new(self.transform_registry.clone());
        let (derived_facts, derived_lineages) =
            derived_engine.compute_derived_facts(&all_facts, &transform_keys)?;

        let mut all_resolved_facts = all_facts;
        all_resolved_facts.extend(derived_facts);

        let resolver_output = self.resolver.resolve(
            request,
            &self.loaded_packs,
            &all_resolved_facts,
            &trust_decisions,
            &derived_lineages,
        )?;

        let compat_decisions = vec![];

        self.trace_emitter.emit(
            request,
            &self.loaded_packs,
            &all_resolved_facts,
            &trust_decisions,
            &derived_lineages,
            &resolver_output,
            &self.config.specificity_profile,
            &self.config.unknown_handling,
            &compat_decisions,
        )
    }

    pub fn packs(&self) -> &[AuthorityPack] {
        &self.loaded_packs
    }

    pub fn config(&self) -> &AuthorityEnvironmentConfig {
        &self.config
    }
}
