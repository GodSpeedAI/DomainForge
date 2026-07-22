//! `domainforge.cell.toml` override schema and safety validation.
//!
//! Overrides may only *specialize* realization (tighten a version, reduce a
//! resource ceiling, swap an approved concrete package) — never weaken
//! declared SEA semantics. Every safety rule below has a `CELL0xx` error
//! code; see `docs/cell-environment-projections.md` for the full table.
//! Unknown keys are rejected for free by `#[serde(deny_unknown_fields)]` at
//! every level (CELL014).

use serde::Deserialize;
use std::collections::BTreeMap;

pub const OVERRIDES_SCHEMA: &str = "domainforge-cell-overrides/v1";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CellOverrides {
    pub schema: String,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub devbox: Option<DevboxOverrides>,
    #[serde(default)]
    pub mise: Option<MiseOverrides>,
    #[serde(default)]
    pub dependency_sets: BTreeMap<String, DependencySetOverride>,
    #[serde(default)]
    pub resources: Option<ResourceOverrides>,
    #[serde(default)]
    pub network: Option<NetworkOverrides>,
    #[serde(default)]
    pub evidence: Option<EvidenceOverrides>,
    #[serde(default)]
    pub unsafe_overrides: Option<UnsafeOverrides>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DevboxOverrides {
    #[serde(default)]
    pub packages: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MiseOverrides {
    #[serde(default)]
    pub tools: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DependencySetOverride {
    pub install_command: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ResourceOverrides {
    pub cpu: Option<u32>,
    pub memory_mb: Option<u32>,
    pub disk_mb: Option<u32>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworkOverrides {
    #[serde(default)]
    pub endpoints: BTreeMap<String, EndpointOverride>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EndpointOverride {
    pub host: Option<String>,
    pub port: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EvidenceOverrides {
    #[serde(default)]
    pub extra_probes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnsafeOverrides {
    pub enabled: bool,
    pub ticket: String,
    pub authority: String,
    pub rationale: String,
    pub expires_at: String,
}

/// Parse and schema-check an override document. Does not apply it —
/// application happens against a built `CellIr` (see `ir.rs`) so safety
/// checks can compare against the already-merged profile/SEA values.
pub fn parse(source: &str) -> Result<CellOverrides, String> {
    let overrides: CellOverrides =
        toml::from_str(source).map_err(|e| format!("CELL010 invalid overrides document: {e}"))?;
    if overrides.schema != OVERRIDES_SCHEMA {
        return Err(format!(
            "CELL010 overrides schema must be '{OVERRIDES_SCHEMA}', got '{}'",
            overrides.schema
        ));
    }
    Ok(overrides)
}

/// Dotted-numeric version ordering shared with `profiles.rs`'s polyglot
/// union; re-exported here so override "tightening" checks use the same
/// comparison.
pub(super) fn version_key(v: &str) -> Vec<u64> {
    v.split('.')
        .map(|seg| {
            seg.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .map(|digits| digits.parse::<u64>().unwrap_or(0))
        .collect()
}

/// Validate `[unsafe_overrides]` metadata completeness (CELL015) and expiry
/// (CELL016) relative to `created_at` (RFC 3339). Called once up front so an
/// incomplete or expired exception never reaches the lock.
pub fn validate_unsafe_overrides(
    unsafe_overrides: &UnsafeOverrides,
    created_at: &str,
) -> Result<(), String> {
    if !unsafe_overrides.enabled {
        return Ok(());
    }
    if unsafe_overrides.ticket.trim().is_empty()
        || unsafe_overrides.authority.trim().is_empty()
        || unsafe_overrides.rationale.trim().is_empty()
        || unsafe_overrides.expires_at.trim().is_empty()
    {
        return Err(
            "CELL015 [unsafe_overrides] requires ticket, authority, rationale, and expires_at"
                .to_string(),
        );
    }
    let expires = chrono::DateTime::parse_from_rfc3339(&unsafe_overrides.expires_at)
        .map_err(|e| format!("CELL015 [unsafe_overrides].expires_at is not RFC 3339: {e}"))?;
    let now = chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|e| format!("CELL010 --created-at is not RFC 3339: {e}"))?;
    if expires <= now {
        return Err(format!(
            "CELL016 [unsafe_overrides].expires_at ({}) has already expired as of {}",
            unsafe_overrides.expires_at, created_at
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_schema() {
        let err = parse("schema = \"nope\"\n").unwrap_err();
        assert!(err.starts_with("CELL010"), "{err}");
    }

    #[test]
    fn rejects_unknown_top_level_key() {
        let err = parse("schema = \"domainforge-cell-overrides/v1\"\nbogus = 1\n").unwrap_err();
        assert!(err.starts_with("CELL010"), "{err}");
    }

    #[test]
    fn accepts_minimal_document() {
        let ov = parse("schema = \"domainforge-cell-overrides/v1\"\n").unwrap();
        assert_eq!(ov.schema, OVERRIDES_SCHEMA);
    }

    #[test]
    fn unsafe_overrides_requires_complete_metadata() {
        let ov = UnsafeOverrides {
            enabled: true,
            ticket: String::new(),
            authority: "platform".into(),
            rationale: "test".into(),
            expires_at: "2026-08-01T00:00:00Z".into(),
        };
        let err = validate_unsafe_overrides(&ov, "2026-07-11T00:00:00Z").unwrap_err();
        assert!(err.starts_with("CELL015"), "{err}");
    }

    #[test]
    fn unsafe_overrides_rejects_expired() {
        let ov = UnsafeOverrides {
            enabled: true,
            ticket: "GS-1".into(),
            authority: "platform".into(),
            rationale: "test".into(),
            expires_at: "2026-01-01T00:00:00Z".into(),
        };
        let err = validate_unsafe_overrides(&ov, "2026-07-11T00:00:00Z").unwrap_err();
        assert!(err.starts_with("CELL016"), "{err}");
    }
}
