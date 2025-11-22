# CI/CD Integration Guide

This guide shows how to integrate SEA DSL validation into your CI/CD pipelines with structured error reporting.

## Overview

SEA provides JSON and LSP output formats specifically designed for CI/CD integration:

- **JSON format**: Machine-readable output for parsing by CI tools
- **LSP format**: IDE and language server integration
- **Exit codes**: Non-zero exit code on validation failures
- **Structured errors**: Error codes, locations, and suggestions

## GitHub Actions

### Basic Validation

```yaml
name: Validate SEA Models

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Build SEA CLI
        run: |
          cd sea-core
          cargo build --release --bin sea --features cli

      - name: Validate Models
        run: |
          ./target/release/sea validate --format json models/ > validation-results.json

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: validation-results
          path: validation-results.json
```

### Advanced: Parse and Report Errors

```yaml
name: Validate with Error Reporting

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Build SEA CLI
        run: |
          cd sea-core
          cargo build --release --bin sea --features cli

      - name: Validate and Report
        id: validate
        run: |
          ./target/release/sea validate --format json models/ > results.json || true
          echo "exit_code=$?" >> $GITHUB_OUTPUT

      - name: Parse Results
        if: always()
        run: |
          python3 << 'EOF'
          import json
          import sys

          with open('results.json') as f:
              results = json.load(f)

          if results['error_count'] > 0:
              print(f"❌ Validation failed with {results['error_count']} errors:")
              for violation in results['violations']:
                  print(f"  [{violation['severity'].upper()}] {violation['policy_name']}: {violation['message']}")
              sys.exit(1)
          else:
              print(f"✅ Validation passed with {len(results['violations'])} total checks")
          EOF

      - name: Comment on PR
        if: github.event_name == 'pull_request' && always()
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const results = JSON.parse(fs.readFileSync('results.json', 'utf8'));

            let comment = '## SEA DSL Validation Results\n\n';

            if (results.error_count > 0) {
              comment += `❌ **${results.error_count} errors found**\n\n`;
              comment += '### Violations:\n\n';
              results.violations.forEach(v => {
                comment += `- **[${v.severity.toUpperCase()}]** ${v.policy_name}: ${v.message}\n`;
              });
            } else {
              comment += `✅ **All validations passed** (${results.violations.length} checks)\n`;
            }

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
```

## GitLab CI

### Basic Pipeline

```yaml
# .gitlab-ci.yml
stages:
  - validate

validate_models:
  stage: validate
  image: rust:latest

  before_script:
    - cd sea-core
    - cargo build --release --bin sea --features cli

  script:
    - ./target/release/sea validate --format json ../models/ > validation-results.json

  artifacts:
    when: always
    paths:
      - validation-results.json
```

### With Error Reporting

```yaml
validate_models:
  stage: validate
  image: rust:latest

  before_script:
    - cd sea-core
    - cargo build --release --bin sea --features cli

  script:
    - |
      ./target/release/sea validate --format json ../models/ > results.json || true
      python3 << 'EOF'
      import json
      import sys

      with open('results.json') as f:
          results = json.load(f)

      print(f"\n{'='*60}")
      print(f"SEA DSL Validation Results")
      print(f"{'='*60}\n")

      if results['error_count'] > 0:
          print(f"❌ {results['error_count']} errors found:\n")
          for v in results['violations']:
              print(f"  [{v['severity'].upper()}] {v['policy_name']}")
              print(f"    {v['message']}\n")
          sys.exit(1)
      else:
          print(f"✅ All validations passed ({len(results['violations'])} checks)")
      EOF

  artifacts:
    when: always
    paths:
      - results.json
```

## Jenkins

### Declarative Pipeline

```groovy
pipeline {
    agent any

    stages {
        stage('Build SEA CLI') {
            steps {
                dir('sea-core') {
                    sh 'cargo build --release --bin sea --features cli'
                }
            }
        }

        stage('Validate Models') {
            steps {
                script {
                    def exitCode = sh(
                        script: './target/release/sea validate --format json models/ > validation-results.json',
                        returnStatus: true
                    )

                    def results = readJSON file: 'validation-results.json'

                    if (results.error_count > 0) {
                        echo "❌ Validation failed with ${results.error_count} errors"
                        results.violations.each { v ->
                            echo "[${v.severity.toUpperCase()}] ${v.policy_name}: ${v.message}"
                        }
                        error("SEA DSL validation failed")
                    } else {
                        echo "✅ Validation passed with ${results.violations.size()} checks"
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'validation-results.json', allowEmptyArchive: true
        }
    }
}
```

## CircleCI

```yaml
# .circleci/config.yml
version: 2.1

jobs:
  validate:
    docker:
      - image: rust:latest

    steps:
      - checkout

      - run:
          name: Build SEA CLI
          command: |
            cd sea-core
            cargo build --release --bin sea --features cli

      - run:
          name: Validate Models
          command: |
            sea-core/target/release/sea validate --format json models/ > results.json || true

      - run:
          name: Report Results
          command: |
            python3 << 'EOF'
            import json
            with open('results.json') as f:
                results = json.load(f)

            if results['error_count'] > 0:
                print(f"❌ {results['error_count']} errors")
                for v in results['violations']:
                    print(f"  [{v['severity']}] {v['message']}")
                exit(1)
            else:
                print(f"✅ Passed ({len(results['violations'])} checks)")
            EOF

      - store_artifacts:
          path: results.json

workflows:
  version: 2
  validate_models:
    jobs:
      - validate
```

## Pre-commit Hook

Add validation to your pre-commit hooks:

```bash
# .git/hooks/pre-commit
#!/bin/bash

echo "Running SEA DSL validation..."

# Find all .sea files
sea_files=$(find . -name "*.sea" -not -path "*/\.*")

if [ -z "$sea_files" ]; then
    echo "No .sea files found"
    exit 0
fi

# Validate
./target/release/sea validate --format human --no-color $sea_files

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ SEA DSL validation failed. Commit aborted."
    echo "Fix the errors above or use 'git commit --no-verify' to skip validation."
    exit 1
fi

echo "✅ SEA DSL validation passed"
exit 0
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Docker Integration

### Dockerfile for CI

```dockerfile
FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cd sea-core && \
    cargo build --release --bin sea --features cli

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y python3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sea /usr/local/bin/sea

ENTRYPOINT ["sea"]
CMD ["validate", "--help"]
```

### Usage in CI

```bash
# Build image
docker build -t sea-validator .

# Validate models
docker run --rm -v $(pwd)/models:/models sea-validator validate --format json /models
```

## Error Code Filtering

Filter specific error codes in your CI:

```python
import json
import sys

with open('results.json') as f:
    results = json.load(f)

# Only fail on specific error codes
critical_codes = ['E001', 'E002', 'E400']

critical_violations = [
    v for v in results['violations']
    if v.get('code') in critical_codes and v['severity'] == 'error'
]

if critical_violations:
    print(f"❌ {len(critical_violations)} critical errors found")
    for v in critical_violations:
        print(f"  [{v['code']}] {v['message']}")
    sys.exit(1)
else:
    print(f"✅ No critical errors (checked {len(results['violations'])} violations)")
```

## Slack/Discord Notifications

### Slack Webhook

```python
import json
import requests

with open('results.json') as f:
    results = json.load(f)

if results['error_count'] > 0:
    message = {
        "text": f"❌ SEA DSL Validation Failed",
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": f"*{results['error_count']} errors found in SEA models*"
                }
            },
            {
                "type": "section",
                "fields": [
                    {
                        "type": "mrkdwn",
                        "text": f"*Violations:*\n{len(results['violations'])}"
                    }
                ]
            }
        ]
    }

    requests.post(
        'YOUR_SLACK_WEBHOOK_URL',
        json=message
    )
```

## Best Practices

1. **Always use JSON format** for machine parsing
2. **Archive validation results** as artifacts
3. **Set appropriate exit codes** for pipeline failures
4. **Filter by severity** if needed (error vs warning)
5. **Include error codes** in reports for tracking
6. **Add validation to PR checks** for early feedback
7. **Cache build artifacts** to speed up validation
8. **Use specific error codes** for critical checks

## Troubleshooting

### Build Failures

If the SEA CLI fails to build:

```bash
# Check Rust version
rustc --version  # Should be 1.75+

# Clean and rebuild
cargo clean
cargo build --release --bin sea --features cli
```

### JSON Parsing Errors

If JSON output is malformed:

```bash
# Validate JSON
cat results.json | python3 -m json.tool

# Check for stderr mixed with stdout
./sea validate --format json models/ 2>/dev/null > results.json
```

### Exit Code Issues

```bash
# Capture exit code explicitly
./sea validate models/
EXIT_CODE=$?
echo "Exit code: $EXIT_CODE"

# Non-zero means validation failed
if [ $EXIT_CODE -ne 0 ]; then
    echo "Validation failed"
fi
```

## See Also

- [Error Code Catalog](error_codes.md) - Complete error code reference
- [CLI Documentation](../reference/cli.md) - Full CLI usage
- [API Specification](api_specification.md) - Programmatic validation
