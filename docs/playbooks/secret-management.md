# Secret Management with SOPS + age

Scope: securely manage project secrets under `secrets/` using SOPS and age; keep encrypted files in git and avoid plaintext leaks.

## Preconditions

- age key pair available (or generate one).
- `sops` installed locally/CI.
- `secrets/secrets.template.yaml` present as a placeholder.

## Steps

1. **Create editable copy**

   ```bash
   cp secrets/secrets.template.yaml secrets/secrets.yaml
   # fill in real values
   ```

2. **Encrypt in place**

   ```bash
   # replace AGE_PUBLIC_KEY with your age public key
   sops --encrypt --age "AGE_PUBLIC_KEY" --in-place secrets/secrets.yaml
   ```

   - Verify encryption (`head -n 5 secrets/secrets.yaml` shows a `sops:` section).

3. **Commit only encrypted files**

   ```bash
   git add secrets/secrets.yaml .sops.yaml
   git commit -m "Add encrypted secrets"
   ```

4. **Decrypt when needed (local only)**

   ```bash
   sops --decrypt secrets/secrets.yaml > /tmp/decrypted-secrets.yaml
   # or edit inline
   sops secrets/secrets.yaml
   ```

5. **Rotate or add recipients**

   ```bash
   sops --add-recipients "AGE_PUBLIC_KEY" --in-place secrets/secrets.yaml
   # remove a key by re-encrypting without it
   ```

6. **Use in CI**

   - Provide `SOPS_AGE_KEY` (private key) as a CI secret.
   - Decrypt with `sops --decrypt secrets/secrets.yaml > /tmp/secrets.yaml` before publishing packages.

## Security notes

- Never commit plaintext secrets or private age keys.
- Keep `.sops.yaml` aligned with repository paths; use per-environment keys when possible.

## See also

- [Configuration](../reference/configuration.md) for registry/environment flags
- [.sops.yaml](../../.sops.yaml) for creation rules
