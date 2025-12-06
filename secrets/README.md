# Secrets directory

This folder contains templates for project secrets and instructions to encrypt them using SOPS and Age.

- `secrets.template.yaml` — placeholders only. Do not add real secrets here.
- `secrets.yaml` — recommended final encrypted file, committed to the repo as encrypted by sops.

Quick usage:

```bash
# copy the template to a file, populate it with real values (locally only)
cp secrets/secrets.template.yaml secrets/secrets.yaml
# encrypt in-place (replace AGE recipient)
sops --encrypt --age "age1PUBKEY" --in-place secrets/secrets.yaml
# verify, commit
git add secrets/secrets.yaml && git commit -m "Add encrypted secrets"
```

Get your age public key (example):

```bash
# generate a new key pair and display the public key
age-keygen -o ~/.config/age/identity.key
# the command prints "Public key: age1..." which you can use in .sops.yaml or sops --age
```

If you accidentally commit unencrypted secrets, remove them from git history, rotate any keys, and inform the team.
