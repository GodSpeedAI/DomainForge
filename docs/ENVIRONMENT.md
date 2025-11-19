# Developer Environment

This document captures the minimum tooling needed to work with the Supabase-backed
end-to-end type-safety pipeline.

## Devbox Environment

We rely on [Devbox](https://www.jetpack.io/devbox/docs/) to provide reproducible
CLI tooling. The root `devbox.json` includes the Supabase CLI. Start a shell with:

```bash
devbox shell
```

## Supabase CLI configuration

Authenticate against Supabase before running any of the commands below:

```bash
supabase login
supabase link --project-ref <your-project-ref>
```

You will need the following environment variables when working with the recipes:

| Variable | Purpose |
| --- | --- |
| `SUPABASE_PROJECT_REF` | Used by `just gen-types-ts` to determine which project schema to pull. |
| `SUPABASE_DB_URL` | Connection string consumed by `just db-migrate` for running migrations. |
| `SUPABASE_JSON_SCHEMA` (optional) | File path to a JSON schema to convert into Python models. Defaults to `schemas/sea-registry.schema.json`. |
| `SOPS_SECRETS_DIR` (optional) | Folder containing decrypted secrets that are mounted into Docker Compose services. |

## Type generation workflow

1. Generate TypeScript types exported by Supabase:
   ```bash
   just gen-types-ts
   ```
2. Generate Python (Pydantic) models:
   ```bash
   just gen-types-py
   ```
3. Verify the repository does not have unstaged changes for either output tree:
   ```bash
   just validate-types
   ```

These commands run inside CI through the default `just ai-validate` task, which
now depends on `validate-types` to ensure artifacts stay in sync.

## Database migrations

Apply migrations to a running Supabase/Postgres instance with:

```bash
SUPABASE_DB_URL=postgres://postgres:postgres@localhost:54322/postgres just db-migrate
```

The recipe is a lightweight wrapper around `supabase migration up`, so anything
supported by the CLI (SSL flags, schema selection, etc.) can be passed via the
connection string itself.

## Local infrastructure via Docker Compose

A minimal Compose configuration is available at `docker-compose.supabase.yml`.
It launches a Postgres instance and Supabase Studio configured to read sensitive
values from `${SOPS_SECRETS_DIR:-.sops-secrets}` so you can manage secrets using
[SOPS](https://github.com/getsops/sops) without committing them to the repo.
Start the stack with:

```bash
SOPS_SECRETS_DIR=$PWD/.sops-secrets docker compose -f docker-compose.supabase.yml up -d
```

Populate the referenced secrets directory with the files described in the
Compose file (anon key, service role key, JWT, etc.) after decrypting them with
SOPS.
