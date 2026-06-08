use super::canonical_json;
use super::schema::SemanticPack;

/// Pack signing result.
#[derive(Debug, Clone)]
pub struct SignOutput {
    pub signature: String,
    pub signature_alg: String,
}

/// Build the signing payload: DOMAINFORGE_SEMANTIC_PACK_V0_3\n<hash>\n<pack_id>\n<schema_version>
pub fn build_signing_payload(pack: &SemanticPack, content_hash: &str) -> Vec<u8> {
    let payload = format!(
        "DOMAINFORGE_SEMANTIC_PACK_V0_3\n{}\n{}\n{}\n",
        content_hash, pack.pack_id, pack.schema_version
    );
    payload.into_bytes()
}

/// Derive a base64-encoded public key fingerprint from a private key PEM.
pub fn derive_signer_id(private_key_pem: &[u8]) -> Result<String, SigningError> {
    let signing_key = parse_private_key(private_key_pem)?;
    let verifying_key = signing_key.verifying_key();
    Ok(base64_encode(verifying_key.as_bytes()))
}

/// Sign pack content with Ed25519 private key PEM bytes.
pub fn sign_pack(pack: &SemanticPack, private_key_pem: &[u8]) -> Result<SignOutput, SigningError> {
    use ed25519_dalek::Signer;

    let signing_key = parse_private_key(private_key_pem)?;
    let content_hash = canonical_json::compute_pack_content_hash(pack);
    let payload = build_signing_payload(pack, &content_hash);
    let signature = signing_key.sign(&payload);

    let sig_bytes = signature.to_bytes();
    Ok(SignOutput {
        signature: base64_encode(&sig_bytes),
        signature_alg: "ed25519".to_string(),
    })
}

/// Verify pack signature with Ed25519 public key PEM bytes.
pub fn verify_pack_signature(
    pack: &SemanticPack,
    public_key_pem: &[u8],
) -> Result<(), VerificationError> {
    use ed25519_dalek::Verifier;

    let verifying_key = parse_public_key(public_key_pem)?;

    let content_hash = canonical_json::compute_pack_content_hash(pack);
    let payload = build_signing_payload(pack, &content_hash);

    let sig_bytes = base64_decode(
        pack.trust
            .signature
            .as_deref()
            .ok_or(VerificationError::MissingSignature)?,
    )?;
    let signature = ed25519_dalek::Signature::from_slice(&sig_bytes)
        .map_err(|_| VerificationError::InvalidSignatureFormat)?;

    verifying_key
        .verify(&payload, &signature)
        .map_err(|_| VerificationError::InvalidSignature)?;

    Ok(())
}

fn parse_private_key(pem: &[u8]) -> Result<ed25519_dalek::SigningKey, SigningError> {
    use ed25519_dalek::pkcs8::DecodePrivateKey;
    ed25519_dalek::SigningKey::from_pkcs8_pem(&String::from_utf8_lossy(pem))
        .map_err(|e| SigningError::KeyParseError(e.to_string()))
}

fn parse_public_key(pem: &[u8]) -> Result<ed25519_dalek::VerifyingKey, VerificationError> {
    use ed25519_dalek::pkcs8::DecodePublicKey;
    ed25519_dalek::VerifyingKey::from_public_key_pem(&String::from_utf8_lossy(pem))
        .map_err(|e| VerificationError::KeyParseError(e.to_string()))
}

fn base64_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    base64::Engine::encode(&STANDARD, data)
}

fn base64_decode(s: &str) -> Result<Vec<u8>, VerificationError> {
    use base64::engine::general_purpose::STANDARD;
    base64::Engine::decode(&STANDARD, s).map_err(|e| VerificationError::Base64Error(e.to_string()))
}

#[derive(Debug, Clone)]
pub enum SigningError {
    KeyParseError(String),
}

#[derive(Debug, Clone)]
pub enum VerificationError {
    MissingSignature,
    InvalidSignatureFormat,
    InvalidSignature,
    KeyParseError(String),
    Base64Error(String),
}
