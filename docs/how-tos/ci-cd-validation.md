# Validate SEA Models in CI/CD

Goal: run `sea` validation in CI pipelines with machine-readable output and PR feedback.

## Prerequisites

- Rust toolchain installed in the CI runner.
- Access to the repository containing `.sea` models (checked out in the workflow).
- Optional: Python/Node for post-processing JSON results.

## Steps

1. **Install and build the CLI**

   ```yaml
   - uses: actions/checkout@v4
   - uses: actions-rust-lang/setup-rust-toolchain@v1
   - name: Build SEA CLI
     run: |
       cd sea-core
       cargo build --release --bin sea --features cli
   ```

2. **Validate models with JSON output**

   ```yaml
   - name: Validate models
     run: |
       ./target/release/sea validate --format json models/ > validation-results.json
   ```

   - Exit code is non-zero on failure; wrap with `|| true` if you want to parse the report first.

3. **Parse and fail the job on violations**

   ```yaml
   - name: Parse validation results
     if: always()
     run: |
       python3 <<'PY'
       import json, sys
       results = json.load(open("validation-results.json"))
       errors = results.get("error_count", 0)
       if errors:
           print(f"❌ Validation failed with {errors} errors")
           for v in results.get("violations", []):
               print(f"- [{v.get('severity','').upper()}] {v.get('policy_name')}: {v.get('message')}")
           sys.exit(1)
       print(f"✅ Validation passed ({len(results.get('violations', []))} checks)")
       PY
   ```

4. **Optional: Comment on pull requests**

   ```yaml
   - name: Comment on PR
     if: github.event_name == 'pull_request' && always()
     uses: actions/github-script@v7
     with:
       script: |
         const fs = require('fs');
         const results = JSON.parse(fs.readFileSync('validation-results.json', 'utf8'));
         let body = '## SEA DSL Validation Results\n\n';
         if (results.error_count > 0) {
           body += `❌ **${results.error_count} errors found**\n\n`;
           body += '### Violations:\n';
           results.violations.forEach(v => { body += `- **[${v.severity.toUpperCase()}]** ${v.policy_name}: ${v.message}\n`; });
         } else {
           body += `✅ **All validations passed** (${results.violations.length} checks)\n`;
         }
         github.rest.issues.createComment({ issue_number: context.issue.number, owner: context.repo.owner, repo: context.repo.repo, body });
   ```

5. **Publish artifacts for debugging (optional)**

   ```yaml
   - uses: actions/upload-artifact@v4
     if: always()
     with:
       name: sea-validation
       path: validation-results.json
   ```

## Verification

- Workflow fails when `error_count > 0`.
- `validation-results.json` contains error codes, locations, and suggestions for CI parsing.

## See also

- [CLI Commands](../reference/cli-commands.md) (`validate`, `explain` flags)
- [Error Codes](../reference/error-codes.md) for interpreting violations
- [Configuration](../reference/configuration.md) for registry and logging options
