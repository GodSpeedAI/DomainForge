use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::AuthorityError;
use super::policy::*;
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityExpression {
    pub expression: String,
    pub location: Option<String>,
}

pub struct PolicyCompiler {
    semantics_version: String,
    compatibility_version: String,
}

impl PolicyCompiler {
    pub fn new(semantics_version: String, compatibility_version: String) -> Self {
        Self {
            semantics_version,
            compatibility_version,
        }
    }

    pub fn compile(
        &self,
        raw_policies: Vec<RawPolicy>,
    ) -> Result<Vec<AuthorityPolicy>, AuthorityError> {
        let mut compiled = Vec::new();
        for raw in raw_policies {
            let policy = self.compile_one(raw)?;
            compiled.push(policy);
        }
        Ok(compiled)
    }

    pub fn semantics_version(&self) -> &str {
        &self.semantics_version
    }

    pub fn compatibility_version(&self) -> &str {
        &self.compatibility_version
    }

    fn compile_one(&self, raw: RawPolicy) -> Result<AuthorityPolicy, AuthorityError> {
        let applies_to = self.parse_structural_predicates(&raw.applies_to)?;
        let when = if raw.when.is_empty() {
            None
        } else {
            Some(self.parse_condition_predicates(&raw.when)?)
        };
        let requires_fact = self.parse_fact_requirements(&raw.requires_fact)?;

        // A3: a `when` path with no matching `requires_fact` entry is
        // satisfied by whatever the caller puts in the request context
        // (ConditionPredicates::evaluate has no source-class filter of its
        // own). Forgetting `requires_fact` silently trusts the caller, so
        // refuse to compile the policy instead of shipping the gap.
        if let Some(ref when) = when {
            for path in when.conditions.keys() {
                if !requires_fact.iter().any(|r| &r.fact_path == path) {
                    return Err(AuthorityError::new(
                        super::error::AuthorityErrorCode::PolicyParseError,
                        format!(
                            "Policy '{}': `when` condition on '{}' has no matching `requires_fact` \
                             entry, so it would be satisfied by unverified caller-supplied context. \
                             Add a `requires_fact` entry for '{}' naming the trusted source classes.",
                            raw.policy_id, path, path
                        ),
                    ));
                }
            }
        }

        // A4: `required_transform` is parsed but never enforced by
        // `FactRequirement::is_satisfied_by`. Silently accepting it would
        // give the policy author a false sense of security. Refuse to
        // compile until enforcement exists.
        for req in &requires_fact {
            if req.required_transform.is_some() {
                return Err(AuthorityError::new(
                    super::error::AuthorityErrorCode::PolicyParseError,
                    format!(
                        "Policy '{}': `required_transform` on fact '{}' is not yet enforced by the \
                         resolver. Remove it rather than rely on an unenforced field.",
                        raw.policy_id, req.fact_path
                    ),
                ));
            }
        }

        Ok(AuthorityPolicy {
            policy_id: raw.policy_id,
            modality: raw.modality,
            priority: raw.priority,
            applies_to,
            when,
            requires_fact,
            semantics_version: self.semantics_version.clone(),
            override_spec: raw.override_spec,
            obligation_spec: raw.obligation_spec,
            description: raw.description,
            evidence_ref: raw.evidence_ref,
        })
    }

    fn parse_structural_predicates(
        &self,
        raw: &HashMap<String, serde_json::Value>,
    ) -> Result<StructuralPredicates, AuthorityError> {
        // A10: `StructuralPredicates::matches` recognizes exactly five keys
        // ("action", "actor.id", "actor.role", "resource.id",
        // "resource.type") and treats everything else as an open metadata
        // lookup. A typo like "resource.typ" therefore doesn't error — it
        // silently becomes a metadata predicate that (almost certainly)
        // never matches, making the policy permanently inert with no
        // diagnostic. The "resource."/"actor." prefixes are reserved for
        // the known structural fields, so reject unrecognized keys under
        // them at compile time instead of shipping a silently-dead policy.
        const KNOWN_STRUCTURAL_KEYS: [&str; 5] = [
            "action",
            "actor.id",
            "actor.role",
            "resource.id",
            "resource.type",
        ];
        for key in raw.keys() {
            validate_fact_path(key)?;
            if (key.starts_with("resource.") || key.starts_with("actor."))
                && !KNOWN_STRUCTURAL_KEYS.contains(&key.as_str())
            {
                return Err(AuthorityError::new(
                    super::error::AuthorityErrorCode::PolicyParseError,
                    format!(
                        "Unrecognized structural predicate key '{}'. The 'resource.' and 'actor.' \
                         prefixes are reserved for {:?}; did you mean one of those?",
                        key, KNOWN_STRUCTURAL_KEYS
                    ),
                ));
            }
        }
        Ok(StructuralPredicates {
            predicates: raw.clone(),
        })
    }

    fn parse_condition_predicates(
        &self,
        raw: &HashMap<String, serde_json::Value>,
    ) -> Result<ConditionPredicates, AuthorityError> {
        for key in raw.keys() {
            validate_fact_path(key)?;
        }
        Ok(ConditionPredicates {
            conditions: raw.clone(),
        })
    }

    fn parse_fact_requirements(
        &self,
        raw: &[RawFactRequirement],
    ) -> Result<Vec<FactRequirement>, AuthorityError> {
        let mut requirements = Vec::new();
        for req in raw {
            validate_fact_path(&req.fact_path)?;
            requirements.push(FactRequirement {
                fact_path: req.fact_path.clone(),
                allowed_source_classes: req.allowed_source_classes.clone(),
                allowed_source_ids: req.allowed_source_ids.clone().unwrap_or_default(),
                max_age: req
                    .max_age
                    .as_ref()
                    .map(|s| parse_duration(s))
                    .transpose()?,
                evidence_ref_required: req.evidence_ref_required.unwrap_or(false),
                signature_required: req.signature_required.unwrap_or(false),
                minimum_confidence: req.minimum_confidence,
                required_transform: req.required_transform.clone(),
                derived_from_source: req.derived_from_source.clone(),
            });
        }
        Ok(requirements)
    }
}

/// A9: parses a duration string. Fixes three foot-guns in the previous
/// version: a bare number silently meant hours (surprising — now rejected,
/// a unit suffix is required); negative durations were accepted (`"-5h"`
/// made every fact look "fresh" forever — now rejected); and suffix
/// matching checked single-char suffixes first, so `"10ms"` matched the
/// `s` branch and became `"10m"` before parsing, producing a confusing
/// error instead of 10 milliseconds. Longer suffixes (`ms`) are now matched
/// before shorter ones (`s`, `m`, `h`, `d`).
fn parse_duration(s: &str) -> Result<chrono::Duration, AuthorityError> {
    let s = s.trim();
    let invalid = || AuthorityError::invalid_environment(format!("Invalid duration: '{}'", s));

    // Longer suffixes must be checked before shorter ones ("ms" before "s").
    let parsed = if let Some(n) = s.strip_suffix("ms") {
        n.parse::<i64>().ok().map(chrono::Duration::milliseconds)
    } else if let Some(n) = s.strip_suffix('s') {
        n.parse::<i64>().ok().map(chrono::Duration::seconds)
    } else if let Some(n) = s.strip_suffix('m') {
        n.parse::<i64>().ok().map(chrono::Duration::minutes)
    } else if let Some(n) = s.strip_suffix('h') {
        n.parse::<i64>().ok().map(chrono::Duration::hours)
    } else if let Some(n) = s.strip_suffix('d') {
        n.parse::<i64>().ok().map(chrono::Duration::days)
    } else {
        return Err(AuthorityError::invalid_environment(format!(
            "Invalid duration: '{}' — a unit suffix (ms, s, m, h, d) is required",
            s
        )));
    };

    match parsed {
        Some(d) if d >= chrono::Duration::zero() => Ok(d),
        _ => Err(invalid()),
    }
}

pub struct CompatibilityLoweringAuditor {
    version: String,
}

impl CompatibilityLoweringAuditor {
    pub fn new(version: String) -> Result<Self, AuthorityError> {
        if version.trim().is_empty() {
            return Err(AuthorityError::invalid_environment(
                "Compatibility lowering version is required",
            ));
        }

        Ok(Self { version })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// A5: the previous implementation lowercased `expr` and applied the
    /// resulting *byte* offsets back onto the original string. For
    /// non-ASCII input where lowercasing changes byte length (e.g. `İ` →
    /// `i̇`), that could slice at a non-UTF8 boundary and panic. It also
    /// matched " and "/" or " inside quoted string literals, so a value
    /// like `role = "black or white"` was rejected as ambiguous. This
    /// version scans char-by-char, tracks quote state, and never lowercases
    /// the whole string — only individual ASCII keyword characters are
    /// compared case-insensitively.
    pub fn audit_expression(&self, expr: &str) -> Result<Option<LoweredPolicy>, AuthorityError> {
        // Reject all top-level structural OR as ambiguous (spec §14.1.7, §17.7).
        // "Top-level" = outside quoted string literals.
        if contains_top_level(expr, " or ") {
            return Err(AuthorityError::ambiguous_lowering(expr));
        }

        // Handle negated conditions: `not X = "Y"` lowers to when condition.
        if let Some(inner) = strip_prefix_ci(expr, "not ") {
            let inner_trimmed = inner.trim();
            // Reject if the negated expression targets structural keys
            // (spec §14.1.7): negative structural predicates are ambiguous.
            if starts_with_ci(inner_trimmed, "resource.")
                || starts_with_ci(inner_trimmed, "actor.")
                || starts_with_ci(inner_trimmed, "action")
            {
                return Err(AuthorityError::ambiguous_lowering(expr));
            }
            return match parse_equality(inner_trimmed) {
                Some((key, val)) => Ok(Some(LoweredPolicy {
                    original: expr.to_string(),
                    lowered_applies_to: HashMap::new(),
                    lowered_when: {
                        let mut m = HashMap::new();
                        m.insert(key, serde_json::json!({"__neq": val}));
                        m
                    },
                })),
                None => Err(AuthorityError::ambiguous_lowering(expr)),
            };
        }

        if contains_top_level(expr, " and ") {
            let mut applies_to = HashMap::new();
            let mut when = HashMap::new();
            for part in split_top_level(expr, " and ") {
                let trimmed = part.trim();
                let (k, v) = parse_equality(trimmed)
                    .ok_or_else(|| AuthorityError::ambiguous_lowering(expr))?;
                if starts_with_ci(trimmed, "resource.")
                    || starts_with_ci(trimmed, "actor.")
                    || starts_with_ci(trimmed, "action")
                {
                    applies_to.insert(k, v);
                } else {
                    when.insert(k, v);
                }
            }
            return Ok(Some(LoweredPolicy {
                original: expr.to_string(),
                lowered_applies_to: applies_to,
                lowered_when: when,
            }));
        }

        if let Some((k, v)) = parse_equality(expr) {
            let mut applies_to = HashMap::new();
            applies_to.insert(k, v);
            return Ok(Some(LoweredPolicy {
                original: expr.to_string(),
                lowered_applies_to: applies_to,
                lowered_when: HashMap::new(),
            }));
        }

        // Unrecognized syntax (e.g. `!=`, `<`, `>=`, an unparenthesized
        // mixed and/or): previously this returned `Ok(None)`, silently
        // leaving the policy uncompiled. A policy compiler that silently
        // drops a clause is a correctness hole, not a convenience — error
        // instead so the gap is visible at compile time.
        Err(AuthorityError::ambiguous_lowering(expr))
    }
}

/// True if `keyword` (matched case-insensitively, ASCII only) occurs outside
/// any double-quoted region of `s`.
fn contains_top_level(s: &str, keyword: &str) -> bool {
    find_top_level(s, keyword).is_some()
}

fn find_top_level(s: &str, keyword: &str) -> Option<usize> {
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let kw: Vec<char> = keyword.chars().collect();
    let mut in_quotes = false;
    let mut i = 0;
    while i < chars.len() {
        let (byte_idx, ch) = chars[i];
        if ch == '"' {
            in_quotes = !in_quotes;
            i += 1;
            continue;
        }
        if !in_quotes
            && i + kw.len() <= chars.len()
            && chars[i..i + kw.len()]
                .iter()
                .zip(kw.iter())
                .all(|((_, c), k)| c.eq_ignore_ascii_case(k))
        {
            return Some(byte_idx);
        }
        i += 1;
    }
    None
}

/// Split `s` on every top-level (outside-quotes) occurrence of `keyword`,
/// matched case-insensitively. Never allocates a lowercased copy of `s`, so
/// byte offsets always land on the original string's char boundaries.
fn split_top_level<'a>(s: &'a str, keyword: &str) -> Vec<&'a str> {
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let kw: Vec<char> = keyword.chars().collect();
    let mut parts = Vec::new();
    let mut in_quotes = false;
    let mut last = 0usize;
    let mut i = 0;
    while i < chars.len() {
        let (byte_idx, ch) = chars[i];
        if ch == '"' {
            in_quotes = !in_quotes;
            i += 1;
            continue;
        }
        if !in_quotes
            && i + kw.len() <= chars.len()
            && chars[i..i + kw.len()]
                .iter()
                .zip(kw.iter())
                .all(|((_, c), k)| c.eq_ignore_ascii_case(k))
        {
            parts.push(&s[last..byte_idx]);
            let (last_byte, last_ch) = chars[i + kw.len() - 1];
            last = last_byte + last_ch.len_utf8();
            i += kw.len();
            continue;
        }
        i += 1;
    }
    parts.push(&s[last..]);
    parts
}

fn starts_with_ci(s: &str, prefix: &str) -> bool {
    let mut sc = s.chars();
    prefix
        .chars()
        .all(|p| sc.next().is_some_and(|c| c.eq_ignore_ascii_case(&p)))
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if starts_with_ci(s, prefix) {
        let byte_len: usize = prefix
            .chars()
            .zip(s.chars())
            .map(|(_, c)| c.len_utf8())
            .sum();
        Some(&s[byte_len..])
    } else {
        None
    }
}

fn parse_equality(s: &str) -> Option<(String, serde_json::Value)> {
    let s = s.trim();
    if let Some(eq_pos) = find_top_level(s, " = ") {
        let key = s[..eq_pos].trim().to_string();
        let val = s[eq_pos + 3..].trim().trim_matches('"').to_string();
        Some((key, serde_json::Value::String(val)))
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct LoweredPolicy {
    pub original: String,
    pub lowered_applies_to: HashMap<String, serde_json::Value>,
    pub lowered_when: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPolicy {
    pub policy_id: String,
    pub modality: PolicyModality,
    pub priority: i32,
    #[serde(default)]
    pub applies_to: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub when: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub requires_fact: Vec<RawFactRequirement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_spec: Option<OverrideSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation_spec: Option<ObligationSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFactRequirement {
    pub fact_path: String,
    pub allowed_source_classes: Vec<SourceClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_source_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_transform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from_source: Option<Vec<SourceClass>>,
}
