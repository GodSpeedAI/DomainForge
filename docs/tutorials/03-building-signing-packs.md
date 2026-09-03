# Tutorial: Governing Vocabulary with Signed Semantic Packs

In this tutorial, you will build a Semantic Pack from a SEA domain model, inspect its canonical JSON structure, sign it with an Ed25519 cryptographic key, and test automated breaking drift detection.

---

## Prerequisites
- DomainForge CLI installed (`domainforge --version`).
- `openssl` installed to generate an Ed25519 key pair.
- The `logistics.sea` file from [Tutorial 1](01-first-sea-model.md).

---

## 1. Build a Candidate Semantic Pack

Run `domainforge pack build` with `--approval candidate`:

```bash
domainforge pack build \
  --source "logistics.sea" \
  --org "acme" \
  --domain "logistics" \
  --version "1.0.0" \
  --meaning-version "1.0.0" \
  --approval candidate \
  --out logistics-pack.json
```

### Inspect the Output Pack
Open `logistics-pack.json`. You will see:
- `schema_version`: `"0.3.0"`
- `pack_id`: `"acme/logistics"`
- `meaning_fingerprint`: A SHA-256 hash calculated over all concept definition records.
- `concepts`: An array containing definitions for `Warehouse`, `FulfillmentCenter`, `Package`, and `daily_dispatch_limit`.
- `trust.approval`: `"candidate"`
- `trust.signature.state`: `"unsigned"`

---

## 2. Generate an Ed25519 Signing Key

Generate a local Ed25519 private key in PEM format:
```bash
openssl genpkey -algorithm ed25519 -out private_key.pem
```

---

## 3. Sign the Semantic Pack

Sign the candidate pack using your private key:

```bash
domainforge pack sign \
  --in logistics-pack.json \
  --key private_key.pem \
  --out signed-pack.json
```

### Inspect the Signed Pack
Open `signed-pack.json` and examine the `trust.signature` section:
```json
"signature": {
  "state": "signed",
  "key_id": "ed25519-key-1",
  "algorithm": "Ed25519",
  "value": "base64-encoded-signature...",
  "public_key": "base64-encoded-public-key..."
}
```

---

## 4. Validate the Signature

Run the pack validator:
```bash
domainforge pack validate --pack signed-pack.json
```

### Expected Output
```text
Validating semantic pack signed-pack.json...
Signature: VALID (Ed25519 verified against embedded public key)
Approval: candidate
Content Hash: MATCH
Pack is valid!
```

---

## 5. Detect Breaking Semantic Drift

Now let's simulate a breaking change. Modify `logistics.sea` to delete the `Warehouse` entity:
```bash
cp logistics.sea modified_logistics.sea
sed -i '/Warehouse/d' modified_logistics.sea
```

Build a new pack candidate from the modified file:
```bash
domainforge pack build \
  --source "modified_logistics.sea" \
  --org "acme" \
  --domain "logistics" \
  --version "2.0.0" \
  --meaning-version "2.0.0" \
  --approval candidate \
  --out new-candidate.json
```

Run `domainforge pack diff` to compare the original signed pack with the new candidate:
```bash
domainforge pack diff --old signed-pack.json --new new-candidate.json --format json
```

### Expected Output
```json
{
  "diff_kind": "breaking",
  "removed_concepts": [
    "logistics::Warehouse"
  ],
  "added_concepts": [],
  "modified_concepts": []
}
```
The diff engine flags the deletion as **`breaking`**! In a CI/CD pipeline, this blocks the pull request automatically, preventing downstream services from breaking due to missing concept definitions.

---

## Next Steps
- Learn more about authority states and review manifests in the [Semantic Packs Subsystem Guide](../subsystems/semantic-packs.md).
- Understand how semantic packs feed into the [Application Contracts Subsystem](../subsystems/application-contracts.md).
