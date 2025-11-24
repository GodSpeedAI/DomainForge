# Linting Specification

## Overview

The SEA DSL Linter checks for code quality issues and potential errors that are not caught by the parser or type checker.

## Rules

Currently implemented rules:

### Keyword Collision

Checks if an identifier matches a reserved keyword.

- **Reserved Keywords**: `Entity`, `Resource`, `Flow`, `Policy`, `Unit`, `Dimension`.
- **Violation**: Using a keyword as an unquoted identifier.
- **Fix**: Quote the identifier (e.g., `"Policy"` instead of `Policy`).

## Implementation

The linter is implemented in `sea-core/src/parser/lint.rs`.

```rust
let linter = Linter::new();
linter.check_identifier("Entity", false); // Error
linter.check_identifier("Entity", true);  // OK
```
