//! Ed25519 verification for fact envelope signatures (A1).
//!
//! Mirrors `semantic_pack::signing` — same key formats, same crate — but signs
//! a fact envelope's identity instead of a whole pack. Gated behind the
//! `signing` feature like the rest of the crate's crypto surface; without the
//! feature, `FactRequirement::is_satisfied_by` falls back to a documented
//! presence-only check (see `types.rs`).

use super::types::FactEnvelope;

/// Canonical payload a fact source signs: path, value, source id, and
/// observation time. Binding all four prevents replaying a signed value under
/// a different path/source/time.
pub fn build_signing_payload(envelope: &FactEnvelope) -> Vec<u8> {
    format!(
        "DOMAINFORGE_FACT_V1\n{}\n{}\n{}\n{}\n",
        envelope.path,
        serde_json::to_string(&envelope.value).unwrap_or_default(),
        envelope.source_id,
        envelope.observed_at.to_rfc3339(),
    )
    .into_bytes()
}

pub fn verify_fact_signature(
    envelope: &FactEnvelope,
    public_key_pem: &[u8],
) -> Result<(), VerificationError> {
    use ed25519_dalek::pkcs8::DecodePublicKey;
    use ed25519_dalek::Verifier;

    let verifying_key =
        ed25519_dalek::VerifyingKey::from_public_key_pem(&String::from_utf8_lossy(public_key_pem))
            .map_err(|e| VerificationError::KeyParseError(e.to_string()))?;

    let sig_str = envelope
        .signature
        .as_deref()
        .ok_or(VerificationError::MissingSignature)?;
    let sig_bytes = {
        use base64::engine::general_purpose::STANDARD;
        base64::Engine::decode(&STANDARD, sig_str)
            .map_err(|e| VerificationError::Base64Error(e.to_string()))?
    };
    let signature = ed25519_dalek::Signature::from_slice(&sig_bytes)
        .map_err(|_| VerificationError::InvalidSignatureFormat)?;

    let payload = build_signing_payload(envelope);
    verifying_key
        .verify(&payload, &signature)
        .map_err(|_| VerificationError::InvalidSignature)
}

#[derive(Debug, Clone)]
pub enum VerificationError {
    MissingSignature,
    InvalidSignatureFormat,
    InvalidSignature,
    KeyParseError(String),
    Base64Error(String),
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSignature => write!(f, "missing signature"),
            Self::InvalidSignatureFormat => write!(f, "invalid signature format"),
            Self::InvalidSignature => write!(f, "signature does not verify"),
            Self::KeyParseError(e) => write!(f, "public key parse error: {e}"),
            Self::Base64Error(e) => write!(f, "base64 decode error: {e}"),
        }
    }
}
