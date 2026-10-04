//! `world_ref`: the ergonomic, immutable reference to exactly one resolved
//! semantic-world revision.
//!
//! A world revision is identified by [`DomainModelIdentity::canonical_digest`]
//! and nothing else; this module defines no second hash. The canonical text
//! form is defined here once:
//!
//! ```text
//! world:<name>@sha256:<64 lowercase hex>
//! ```
//!
//! * [`WorldRef`] — immutable; durable and consequential records pin this.
//! * [`WorldAlias`] — mutable selector `world:<name>`; must be resolved to a
//!   [`WorldRef`] before consequential execution ([`WorldCatalog::pin`]).
//! * [`WorldLabel`] — presentation metadata only; never identity.
//!
//! The digest binds the exact source set, the semantic closure, the producer
//! release and the interpretation/canonicalization versions. A comment-only
//! source edit or a DomainForge upgrade therefore mints a new `world_ref`;
//! [`DomainModelIdentity::semantic_closure_hash`] is the field that proves two
//! revisions mean the same thing.

use super::envelope::{CanonicalSemanticEnvelopeDocument, DomainModelIdentity};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// The only identity-preimage scheme this module accepts.
pub const SUPPORTED_IDENTITY_SCHEME: &str = "v2-full-preimage";
const WORLD_PREFIX: &str = "world:";
const DIGEST_PREFIX: &str = "sha256:";
const MAX_NAME_LEN: usize = 64;
const MAX_LABEL_LEN: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldRefError {
    Malformed(String),
    InvalidName(String),
    InvalidDigest(String),
    InvalidLabel(String),
    DigestMismatch {
        expected: String,
        actual: String,
    },
    UnsupportedIdentityScheme(String),
    MalformedIdentity(String),
    UnknownWorld(String),
    UnknownAlias(String),
    NameConflict {
        digest: String,
        existing: String,
        requested: String,
    },
}

impl fmt::Display for WorldRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(s) => write!(f, "malformed world reference: {s}"),
            Self::InvalidName(s) => write!(f, "invalid world name: {s}"),
            Self::InvalidDigest(s) => write!(f, "invalid world digest: {s}"),
            Self::InvalidLabel(s) => write!(f, "invalid world label: {s}"),
            Self::DigestMismatch { expected, actual } => write!(
                f,
                "world identity digest mismatch: reference pins {expected}, identity hashes to {actual}"
            ),
            Self::UnsupportedIdentityScheme(s) => {
                write!(f, "unsupported identity scheme '{s}' (expected {SUPPORTED_IDENTITY_SCHEME})")
            }
            Self::MalformedIdentity(s) => write!(f, "malformed domain model identity: {s}"),
            Self::UnknownWorld(s) => write!(f, "unknown world: {s}"),
            Self::UnknownAlias(s) => write!(f, "unknown world alias: {s}"),
            Self::NameConflict { digest, existing, requested } => write!(
                f,
                "world {digest} is already registered as '{existing}', not '{requested}'"
            ),
        }
    }
}

impl std::error::Error for WorldRefError {}

fn is_lower_hex(s: &str) -> bool {
    s.bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate_digest(text: &str) -> Result<(), WorldRefError> {
    let hex = text
        .strip_prefix(DIGEST_PREFIX)
        .ok_or_else(|| WorldRefError::InvalidDigest(format!("'{text}' is not sha256:<hex>")))?;
    if hex.len() != 64 || !is_lower_hex(hex) {
        return Err(WorldRefError::InvalidDigest(format!(
            "'{text}' is not sha256: followed by 64 lowercase hex digits"
        )));
    }
    Ok(())
}

/// `[a-z][a-z0-9]*([._-][a-z0-9]+)*`, at most 64 characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorldName(String);

impl WorldName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for WorldName {
    type Err = WorldRefError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        let bad = |why: &str| WorldRefError::InvalidName(format!("'{s}': {why}"));
        if bytes.is_empty() || bytes.len() > MAX_NAME_LEN {
            return Err(bad("length must be 1..=64"));
        }
        if !bytes[0].is_ascii_lowercase() {
            return Err(bad("must start with a lowercase letter"));
        }
        let mut prev_sep = false;
        for &b in bytes {
            let sep = matches!(b, b'.' | b'_' | b'-');
            if !(b.is_ascii_lowercase() || b.is_ascii_digit() || sep) {
                return Err(bad("only lowercase letters, digits and . _ -"));
            }
            if sep && prev_sep {
                return Err(bad("separators must not repeat"));
            }
            prev_sep = sep;
        }
        if prev_sep {
            return Err(bad("must not end with a separator"));
        }
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for WorldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Immutable reference to one resolved semantic-world revision.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorldRef {
    name: WorldName,
    /// `sha256:<64 lowercase hex>`, exactly `DomainModelIdentity::canonical_digest()`.
    digest: String,
}

impl WorldRef {
    /// Mint a reference from a DomainForge identity. Fails closed on an
    /// unsupported identity scheme or malformed hash fields.
    pub fn from_identity(
        name: WorldName,
        identity: &DomainModelIdentity,
    ) -> Result<Self, WorldRefError> {
        identity.validate()?;
        Ok(Self {
            name,
            digest: identity.canonical_digest(),
        })
    }

    pub fn name(&self) -> &WorldName {
        &self.name
    }

    /// `sha256:<hex>`, identical to `DomainModelIdentity::canonical_digest()`.
    pub fn digest(&self) -> String {
        self.digest.clone()
    }

    /// Recompute the identity digest and require it to equal the pinned one.
    pub fn verify_identity(&self, identity: &DomainModelIdentity) -> Result<(), WorldRefError> {
        identity.validate()?;
        let actual = identity.canonical_digest();
        if actual == self.digest {
            Ok(())
        } else {
            Err(WorldRefError::DigestMismatch {
                expected: self.digest.clone(),
                actual,
            })
        }
    }
}

impl FromStr for WorldRef {
    type Err = WorldRefError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s
            .strip_prefix(WORLD_PREFIX)
            .ok_or_else(|| WorldRefError::Malformed(format!("'{s}' lacks the 'world:' scheme")))?;
        let (name, digest) = rest.split_once('@').ok_or_else(|| {
            WorldRefError::Malformed(format!("'{s}' has no @sha256:<digest> (is it an alias?)"))
        })?;
        let name: WorldName = name.parse()?;
        validate_digest(digest)?;
        Ok(Self {
            name,
            digest: digest.to_string(),
        })
    }
}

impl fmt::Display for WorldRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{WORLD_PREFIX}{}@{}", self.name, self.digest)
    }
}

impl Serialize for WorldRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WorldRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Mutable selector `world:<name>`. Never identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorldAlias(WorldName);

impl WorldAlias {
    pub fn name(&self) -> &WorldName {
        &self.0
    }
}

impl FromStr for WorldAlias {
    type Err = WorldRefError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s
            .strip_prefix(WORLD_PREFIX)
            .ok_or_else(|| WorldRefError::Malformed(format!("'{s}' lacks the 'world:' scheme")))?;
        if rest.contains('@') {
            return Err(WorldRefError::Malformed(format!(
                "'{s}' pins a digest; it is a world_ref, not an alias"
            )));
        }
        Ok(Self(rest.parse()?))
    }
}

impl fmt::Display for WorldAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{WORLD_PREFIX}{}", self.0)
    }
}

/// Human-readable name, e.g. `GodSpeed Corporate World`. Presentation only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLabel(String);

impl WorldLabel {
    pub fn new(text: &str) -> Result<Self, WorldRefError> {
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.chars().count() > MAX_LABEL_LEN {
            return Err(WorldRefError::InvalidLabel(
                "must be 1..=200 characters after trimming".into(),
            ));
        }
        if trimmed.chars().any(char::is_control) {
            return Err(WorldRefError::InvalidLabel(
                "contains control characters".into(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl DomainModelIdentity {
    /// Build the identity of a resolved model from its canonical document.
    /// `content_hash` is the document's `self_hash`; the pack-set hash is the
    /// document's (the empty pack set still hashes a fixed frame).
    pub fn from_document(
        doc: &CanonicalSemanticEnvelopeDocument,
        registry_content_hash: Option<&str>,
    ) -> Self {
        Self {
            identity_scheme_version: SUPPORTED_IDENTITY_SCHEME.to_string(),
            producer: doc.producer.name.clone(),
            producer_version: doc.producer.version.clone(),
            language_schema_version: doc.inputs.language_schema_version.clone(),
            compiler_interpretation_version: doc.envelope.compiler_interpretation_version.clone(),
            canonicalization_version: doc.envelope.canonicalization_version.clone(),
            source_set_hash: doc.inputs.source_set_hash.clone(),
            content_hash: doc.self_hash.clone(),
            semantic_closure_hash: doc.semantic_closure_hash.clone(),
            semantic_pack_set_hash: Some(doc.inputs.semantic_pack_set_hash.clone()),
            registry_content_hash: registry_content_hash.map(str::to_string),
        }
    }

    /// Fail closed on an unsupported scheme or malformed hash fields.
    pub fn validate(&self) -> Result<(), WorldRefError> {
        if self.identity_scheme_version != SUPPORTED_IDENTITY_SCHEME {
            return Err(WorldRefError::UnsupportedIdentityScheme(
                self.identity_scheme_version.clone(),
            ));
        }
        let required = [
            ("source_set_hash", &self.source_set_hash),
            ("content_hash", &self.content_hash),
            ("semantic_closure_hash", &self.semantic_closure_hash),
        ];
        for (field, value) in required {
            validate_digest(value)
                .map_err(|e| WorldRefError::MalformedIdentity(format!("{field}: {e}")))?;
        }
        for (field, value) in [
            ("semantic_pack_set_hash", &self.semantic_pack_set_hash),
            ("registry_content_hash", &self.registry_content_hash),
        ] {
            if let Some(v) = value {
                validate_digest(v)
                    .map_err(|e| WorldRefError::MalformedIdentity(format!("{field}: {e}")))?;
            }
        }
        Ok(())
    }
}

/// In-memory catalog of verified, immutable worlds plus mutable aliases.
/// Worlds are append-only: a registered revision is never replaced or removed,
/// and alias retargeting only appends to history.
#[derive(Debug, Default, Clone)]
pub struct WorldCatalog {
    worlds: BTreeMap<String, (WorldRef, DomainModelIdentity)>,
    aliases: BTreeMap<WorldName, Vec<WorldRef>>,
}

impl WorldCatalog {
    /// Verify and register a world. Idempotent for the same name and identity.
    /// A digest has exactly one canonical name within a catalog.
    pub fn register(
        &mut self,
        name: WorldName,
        identity: DomainModelIdentity,
    ) -> Result<WorldRef, WorldRefError> {
        let world = WorldRef::from_identity(name, &identity)?;
        if let Some((existing, _)) = self.worlds.get(&world.digest) {
            if existing.name != world.name {
                return Err(WorldRefError::NameConflict {
                    digest: world.digest.clone(),
                    existing: existing.name.to_string(),
                    requested: world.name.to_string(),
                });
            }
            return Ok(existing.clone());
        }
        self.worlds
            .insert(world.digest.clone(), (world.clone(), identity));
        Ok(world)
    }

    /// The registered identity for a reference; unknown references fail closed.
    pub fn lookup(&self, world: &WorldRef) -> Result<&DomainModelIdentity, WorldRefError> {
        match self.worlds.get(&world.digest) {
            Some((registered, identity)) if registered.name == world.name => Ok(identity),
            Some((registered, _)) => Err(WorldRefError::NameConflict {
                digest: world.digest.clone(),
                existing: registered.name.to_string(),
                requested: world.name.to_string(),
            }),
            None => Err(WorldRefError::UnknownWorld(world.to_string())),
        }
    }

    /// Known world and a presented identity that hashes to the pinned digest.
    pub fn verify(
        &self,
        world: &WorldRef,
        presented: &DomainModelIdentity,
    ) -> Result<(), WorldRefError> {
        self.lookup(world)?;
        world.verify_identity(presented)
    }

    /// Point an alias at a registered world. Appends to the alias history.
    pub fn retarget_alias(
        &mut self,
        alias: &WorldAlias,
        target: &WorldRef,
    ) -> Result<(), WorldRefError> {
        self.lookup(target)?;
        self.aliases
            .entry(alias.0.clone())
            .or_default()
            .push(target.clone());
        Ok(())
    }

    pub fn resolve_alias(&self, alias: &WorldAlias) -> Result<WorldRef, WorldRefError> {
        self.aliases
            .get(&alias.0)
            .and_then(|history| history.last())
            .cloned()
            .ok_or_else(|| WorldRefError::UnknownAlias(alias.to_string()))
    }

    /// Every revision the alias has ever pointed at, oldest first.
    pub fn alias_history(&self, alias: &WorldAlias) -> Vec<WorldRef> {
        self.aliases.get(&alias.0).cloned().unwrap_or_default()
    }

    /// Resolve a selector to a known immutable [`WorldRef`] for consequential
    /// use. Accepts a pinned `world:<name>@sha256:<digest>` (must be known) or
    /// an alias `world:<name>` (must be known and is resolved now). Anything
    /// else fails closed.
    pub fn pin(&self, selector: &str) -> Result<WorldRef, WorldRefError> {
        if selector.contains('@') {
            let world: WorldRef = selector.parse()?;
            self.lookup(&world)?;
            Ok(world)
        } else {
            self.resolve_alias(&selector.parse::<WorldAlias>()?)
        }
    }
}

// ---- string/JSON entry points shared by the Python, TypeScript and WASM bindings ----
//
// Bindings call these and add no logic of their own.

/// Resolve a source map and return its `DomainModelIdentity` as canonical JSON.
pub fn domain_model_identity_json(
    entry_logical_path: &str,
    sources_json: &str,
    registry_content_hash: Option<&str>,
) -> Result<String, Vec<super::ApplicationDiagnostic>> {
    let doc = super::resolve_semantic_envelope(entry_logical_path, sources_json)?;
    let identity = DomainModelIdentity::from_document(&doc, registry_content_hash);
    let value = serde_json::to_value(&identity).expect("DomainModelIdentity serializes");
    Ok(crate::semantic_pack::canonical_json::canonical_json(&value))
}

fn identity_from_json(identity_json: &str) -> Result<DomainModelIdentity, WorldRefError> {
    serde_json::from_str(identity_json)
        .map_err(|e| WorldRefError::MalformedIdentity(format!("not a DomainModelIdentity: {e}")))
}

/// Mint the canonical `world:<name>@sha256:<digest>` text from identity JSON.
pub fn world_ref_from_identity_json(
    name: &str,
    identity_json: &str,
) -> Result<String, WorldRefError> {
    let identity = identity_from_json(identity_json)?;
    Ok(WorldRef::from_identity(name.parse()?, &identity)?.to_string())
}

/// Verify that identity JSON hashes to the digest pinned by a world_ref.
pub fn verify_world_ref_json(world_ref: &str, identity_json: &str) -> Result<(), WorldRefError> {
    let world: WorldRef = world_ref.parse()?;
    world.verify_identity(&identity_from_json(identity_json)?)
}

/// Parse and return the canonical text of a world_ref (rejects anything else).
pub fn canonical_world_ref(text: &str) -> Result<String, WorldRefError> {
    Ok(text.parse::<WorldRef>()?.to_string())
}
