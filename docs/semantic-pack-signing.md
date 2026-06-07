# Semantic Pack Signing and Verification

Semantic packs use Ed25519 detached signatures to provide tamper evidence and authenticity verification. This document describes the signing algorithm, payload format, CLI commands, and how the LSP and CI handle signature states.

## Signing Algorithm

DomainForge uses **Ed25519** (Edwards-curve Digital Signature Algorithm) for pack signing. Ed25519 provides:

- 128-bit security level.
- Deterministic signatures (no randomness in signature generation).
- Compact signatures (64 bytes).
- Fast verification.

The implementation uses the `ed25519-dalek` crate with PKCS#8 PEM key encoding.

## Signature Payload Format

The signing payload is a UTF-8 byte string with the following format:

```
DOMAINFORGE_SEMANTIC_PACK_V0_3\n<content_hash>\n<pack_id>\n<schema_version>\n
```

Where:

- `DOMAINFORGE_SEMANTIC_PACK_V0_3` is a fixed domain separation string.
- `<content_hash>` is the pack content hash (SHA-256) with signature fields excluded.
- `<pack_id>` is the pack's unique identifier (e.g., `acme/logistics/1.1.0`).
- `<schema_version>` is the pack schema version (e.g., `0.3`).

The trailing newline ensures the payload is unambiguous when concatenated.

## Pack Content Hash

The pack content hash is computed over the canonical JSON representation of the pack with all signature fields cleared:

```rust
// Fields cleared before hashing:
pack.trust.signature = None;
pack.trust.signature_state = Unsigned;
```

This means:

- Signing a pack does not change its content hash.
- The content hash is deterministic regardless of signature state.
- Two packs with identical semantic content but different signatures have the same content hash.

The canonicalization process sorts all keys, arrays, and sub-objects deterministically to ensure reproducible hashes across builds.

## CLI Commands

### Signing a Pack

```bash
sea pack sign <pack-path> --key <private-key.pem>
```

Options:

| Option        | Description                                   |
|---------------|-----------------------------------------------|
| `<pack-path>` | Path to the unsigned pack JSON file.          |
| `--key`       | Path to the Ed25519 private key (PEM format). |
| `--out`       | (Optional) Output path for the signed pack.   |

The command reads the pack, computes the content hash, signs the payload, and writes the signed pack with the `trust` fields updated:

```json
{
  "trust": {
    "approval_state": "approved",
    "signature_state": "signed",
    "signed_by": "pack-filename",
    "signature_alg": "ed25519",
    "signature": "<base64-encoded-signature>"
  }
}
```

Example:

```bash
sea pack sign packs/acme-logistics-1.1.0.json \
  --key keys/acme-private.pem \
  --out packs/acme-logistics-1.1.0-signed.json
```

### Verifying a Pack Signature

```bash
sea pack verify <pack-path> --key <public-key.pem>
```

Options:

| Option        | Description                                   |
|---------------|-----------------------------------------------|
| `<pack-path>` | Path to the signed pack JSON file.            |
| `--key`       | Path to the Ed25519 public key (PEM format).  |

The command:

1. Reads the pack.
2. Computes the content hash (excluding signature fields).
3. Reconstructs the signing payload.
4. Decodes the base64 signature from the pack.
5. Verifies the Ed25519 signature against the public key.

Example:

```bash
sea pack verify packs/acme-logistics-1.1.0-signed.json \
  --key keys/acme-public.pem
```

## Exit Codes

| Code | Meaning                                 |
|------|-----------------------------------------|
| 0    | Signature is valid.                     |
| 3    | Pack is unsigned (no signature present).|
| 4    | Signature is invalid or has been tampered with. |

The `sea pack validate` command uses these same codes when `--require-signature` is set:

```bash
# Exit 3 if unsigned
sea pack validate --pack packs/unsigned.json --require-signature models/*.sea

# Exit 4 if signature is invalid
sea pack validate --pack packs/tampered.json --require-signature models/*.sea
```

## LSP Behavior

The DomainForge LSP adjusts its behavior based on the pack's signature state:

| Signature State    | LSP Behavior                                                            |
|--------------------|-------------------------------------------------------------------------|
| `unsigned`         | Loads the pack in **warn mode**. All features are available but a diagnostic warns that the pack is unsigned. |
| `signed`           | Full functionality with no warnings.                                    |
| `invalid_signature`| Pack features are **disabled**. The LSP reports a critical error and does not use the pack for resolution. |

In practice, unsigned packs are common during development. Teams should sign packs before merging to main branches.

## CI Strict Behavior

When `--mode strict --require-signature` is set:

- Unsigned packs cause the validation to fail with exit code 3.
- Invalid signatures cause the validation to fail with exit code 4.
- The `--expected-hash` flag provides additional tamper protection by pinning the expected content hash.

```bash
sea pack validate \
  --pack packs/acme-logistics-1.1.0-signed.json \
  --mode strict \
  --require-signature \
  --expected-hash sha256:abc123def456... \
  models/**/*.sea
```

If the pack's content hash does not match `--expected-hash`, the command exits with code 4 regardless of signature validity.

## Generating Ed25519 Keys

Use `openssl` to generate Ed25519 key pairs:

### Generate a Private Key

```bash
openssl genpkey -algorithm Ed25519 -out acme-private.pem
```

### Extract the Public Key

```bash
openssl pkey -in acme-private.pem -pubout -out acme-public.pem
```

### Verify Key Format

Private keys should be in PKCS#8 PEM format:

```
-----BEGIN PRIVATE KEY-----
MC4CAQ...
-----END PRIVATE KEY-----
```

Public keys should be in SPKI PEM format:

```
-----BEGIN PUBLIC KEY-----
MCowBQ...
-----END PUBLIC KEY-----
```

## Expected-Hash Pinning

For environments where defense-in-depth is required, the `--expected-hash` flag pins the expected content hash of the pack. This provides tamper protection even if the signing key is compromised:

```bash
EXPECTED_HASH=$(sea pack inspect --pack packs/acme-logistics-1.1.0.json --format json | jq -r '.content_hash')

# Store the hash in CI configuration
echo "PACK_HASH=$EXPECTED_HASH" >> ci.env

# Later, in CI:
sea pack validate \
  --pack packs/acme-logistics-1.1.0-signed.json \
  --require-signature \
  --expected-hash "$PACK_HASH" \
  models/**/*.sea
```

The content hash is computed with signature fields excluded, so it remains stable across signing and verification operations.

## Signature Verification Errors

The verifier reports specific error types:

| Error                        | Cause                                                    |
|------------------------------|----------------------------------------------------------|
| `MissingSignature`          | The pack has no `signature` field in `trust`.            |
| `InvalidSignatureFormat`    | The base64 signature cannot be decoded to 64 bytes.      |
| `InvalidSignature`          | The Ed25519 signature does not verify against the public key and payload. |
| `KeyParseError`             | The public key PEM is malformed or not Ed25519.          |
| `Base64Error`               | The signature base64 encoding is invalid.                |

## See Also

- [Semantic Packs](semantic-packs.md) for the overall pack system.
- [Semantic Pack Review Process](semantic-pack-review.md) for the review gate that precedes signing.
- [Semantic Diagnostic Codes](diagnostics.md) for `pack_unsigned` and `pack_signature_invalid` codes.
