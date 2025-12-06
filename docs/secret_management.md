# Secret Management with sops + age for DomainForge

This document explains how to safely manage and commit project secrets using Mozilla [SOPS](https://github.com/mozilla/sops) and the `age` encryption tool ([age](https://age-encryption.org)).

NOTE: This repo contains a `secrets/secrets.template.yaml` with placeholders — do not commit plaintext secrets.

## Files created by this repo

- `secrets/secrets.template.yaml` — a local plaintext template containing placeholders. **Do not commit real secrets.**
- `.sops.yaml` — SOPS config (created in project root) which contains a creation rule to encrypt files under `secrets/` with AGE recipients.

## Recommended workflow

1. Add your secrets to a local plaintext copy (or use the template):

```bash
cp secrets/secrets.template.yaml secrets/secrets.yaml
# Edit secrets/secrets.yaml and replace placeholders with real values
```

1. Encrypt with sops (in-place) and commit the encrypted file. Replace `age1...` with your `age` public key (or add multiple recipients for collaborators):

```bash
# replace age1PUBKEY below with your public age key
sops --encrypt --age "age1PUBKEY" --in-place secrets/secrets.yaml

# verify file was encrypted (sops adds a `sops` top-level key)
head -n 15 secrets/secrets.yaml

# Commit: only the encrypted `secrets/secrets.yaml` is safe to commit
git add secrets/secrets.yaml .sops.yaml
git commit -m "Add encrypted secrets with sops"
```

1. Decrypt locally when you need to read the secrets (requires private age key in your keyring):

```bash
sops --decrypt secrets/secrets.yaml > /tmp/decrypted-secrets.yaml
# or open with sops directly
sops secrets/secrets.yaml
```

## To add another age recipient (other developers/key rotation)

To add another public key as a recipient, run:

```bash
sops --add-recipients "age1OTHERPUBKEY" --in-place secrets/secrets.yaml
```

Remove a recipient by: (re-encrypt without recipient)

```bash
sops --remove-recipient "age1OLDKEY" --in-place secrets/secrets.yaml
```

## Using secrets in CI or tests

- Provide the age private key to the CI via secure environment variables and configure the CI runner to be able to decrypt with sops.
- Alternatively use KMS/secret manager integrations and use those providers in `.sops.yaml` instead of age.

## Security notes

- Keep your private age key out of the repository. Never store private keys in source control.
- Use the `secrets/secrets.template.yaml` to reference the keys required for different environments or for new developers.

## Additional references

- [sops](https://github.com/mozilla/sops)
- [age](https://age-encryption.org)

## Getting your age public key (examples)

If you already have an age key pair, you can extract or print your public key; otherwise, you can generate a new key pair.


Most common (Go implementation, official `age` binary):

1. Generate a new keypair (writes a private key file and prints the public key):

```bash
# Generate a new private key (and show public):
age-keygen -o ~/.config/age/identity.key
# Example output (public key printed by the command):
# Public key: age1qsomethingyours
```

1. If you already have a private key file, request the public key from it (prints the public key):

```bash
# If `age-keygen -y -i` is supported in your version of age:
age-keygen -y -i ~/.config/age/identity.key

# Alternatively: re-run key generation into a new file (it will print the public key):
age-keygen -o ~/tmp/identity.key
# the command will print the public key. Keep the new file private (remove after use) or move it securely.
```

1. Copy the public key to your clipboard (Linux / macOS):

```bash
# Example using xclip (Linux):
echo -n "age1...yourPublicKey..." | xclip -selection clipboard
# macOS (pbcopy):
echo -n "age1...yourPublicKey..." | pbcopy
```

1. Add the public key to your `.sops.yaml` (or use it with sops directly):

```bash
# Encrypt with sops using an age public key
sops --encrypt --age "age1YOURPUBKEY" --in-place secrets/secrets.yaml

> ⚠️ Tip: If you see an error like
>
> invalid age key configuration: expected string, []string, or nil, got map[string]interface {}
>
> it usually means your `.sops.yaml` used a `map` representation such as:
>
> ```yaml
> creation_rules:
>   - path_regex: "secrets/.*\\.ya?ml$"
>     age:
>       recipients:
>         - "age1YOURPUBKEY"
> ```
>
> but sops expects `age` to be a plain string or a list of strings. Change it to this instead:
>
> ```yaml
> creation_rules:
>   - path_regex: "secrets/.*\\.ya?ml$"
>     age:
>       - "age1YOURPUBKEY"
> ```
>
> This change is compatible with most sops versions and avoids the "invalid age key configuration" error.
```

Notes & troubleshooting:

- `age-keygen` is the common tool bundled with `age`. If your installation provides `rage` (Rust implementation) or a different CLI, check the tool's `--help` or online docs for the equivalent commands to print public keys.
- If `age-keygen -y -i` fails, re-generating a new key temporarily to show the public key is safe (but delete the private key if you don't want it), otherwise consider creating a new key pair and adding the public key as a recipient.
