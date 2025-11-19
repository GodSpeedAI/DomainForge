default := "ai-validate"

# Run all validation checks required for CI and AI agents
ai-validate:
    cargo test -p sea-core
