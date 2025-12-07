# Error Code Catalog

This document provides a comprehensive catalog of all error codes used in the SEA DSL validation system.

## Error Code Format

Error codes follow the format `EXXX` where:

- `E` indicates an error
- `XXX` is a three-digit number organized by category

## Error Categories

- **E001-E099**: Syntax and Parsing Errors
- **E100-E199**: Type System Errors
- **E200-E299**: Unit and Dimension Errors
- **E300-E399**: Scope and Reference Errors
- **E400-E499**: Policy Validation Errors
- **E500-E599**: Namespace and Module Errors

---

## E001-E099: Syntax and Parsing Errors

*Note:* Codes in the range **E011–E099** are currently reserved for future syntax/parse error additions and will be documented as the project expands. For now, E001–E010 cover the commonly used parsing diagnostics.

### E001: Undefined Entity

**Description**: Reference to an entity that has not been defined.

**Example**:

```sea
Flow "Materials" from "Warehouse" to "Factory" quantity 100
// Error: Entity "Warehouse" is not defined
```

**Common Fixes**:

- Define the entity before referencing it
- Check for typos in the entity name
- Ensure the entity is in the correct namespace

**Suggestion**: The error will suggest similar entity names if available using fuzzy matching.

---

### E002: Undefined Resource

**Description**: Reference to a resource that has not been defined.

**Example**:

```sea
Flow "Steel" from "Supplier" to "Factory" quantity 100
// Error: Resource "Steel" is not defined
```

**Common Fixes**:

- Define the resource before using it
- Check for typos in the resource name
- Verify the resource is in scope

**Suggestion**: The error will suggest similar resource names if available.

---

### E003: Unit Mismatch (moved)

**Description**: This entry was previously documented in the Syntax/Parsing section and has been moved to the Unit/Dimension category (E200–E299). See `E200: Dimension Mismatch` and `E203: Incompatible Dimensions` for details and suggested fixes.

---

*Note:* The old `E004: Type Mismatch` entry has been moved into the Type System section (E100–E199). See `E100: Incompatible Types` for details and suggested fixes.

---

### E005: Syntax Error

**Description**: General syntax error in DSL parsing.

**Example**:

```sea
Entity "Warehouse" in logistics
Flow "Materials" from "Warehouse" to // Missing destination
```

**Common Fixes**:

- Check for missing or extra tokens
- Verify proper DSL syntax
- Look for unclosed quotes or brackets

**Suggestion**: Shows line and column of the error.

---

### E006: Invalid Expression

**Description**: An expression that is syntactically valid but semantically invalid.

**Example**:

```sea
Entity "API"
Resource "Payload" units
Flow "Payload" from "API" to "API"

Policy invalid_expr as:
  Payload and 42
  // Error: mixes a resource reference and a number without a valid operator
```

**Common Fixes**:

- Review expression logic
- Check operator precedence
- Verify operand types

---

### E007: Duplicate Declaration

**Description**: An entity, resource, or other item is declared more than once.

**Example**:

```sea
Entity "Warehouse" in logistics
Entity "Warehouse" in manufacturing
// Error: "Warehouse" is already defined
```

**Common Fixes**:

- Use unique names for each declaration
- Remove duplicate declarations
- Use namespaces to avoid conflicts

**Suggestion**: Shows both declaration locations.

---

### E008: Undefined Variable

**Description**: Reference to a variable that is not in scope.

**Example**:

```sea
Entity "API"
Resource "Payload" units
// Missing entity definition for Backend
Flow "Payload" from "API" to "Backend"
// Error: "Backend" is referenced but never declared
```

**Common Fixes**:

- Define the variable before use
- Check variable scope
- Verify spelling

**Suggestion**: Suggests similar variable names in scope.

---

### E009: Invalid Quantity

**Description**: A quantity value is invalid or malformed.

**Example**:

```sea
Flow "Steel" from "Mine" to "Refinery" quantity -100
// Error: Negative quantity
```

**Common Fixes**:

- Use positive quantities
- Check numeric format
- Verify units

---

### E010: Invalid Identifier

**Description**: An identifier (name) is invalid or uses reserved keywords.

**Example**:

```sea
Entity "Entity" in logistics
// Error: "Entity" is a reserved keyword
```

**Common Fixes**:

- Use different names
- Avoid reserved keywords
- Follow naming conventions

---

## E100-E199: Type System Errors

### E100: Incompatible Types

**Description**: Types cannot be used together in the given context.

**Note**: This entry also covers `Type Mismatch` cases (previously E004) such as assigning a string where a number is expected.

**Example**:

```sea
Policy upload_capacity as:
  "high" + 5
// Error: cannot add a string and a number
```

**Common Fixes**:

- Ensure types match in assignments
- Use appropriate type conversions
- Check function signatures and type requirements

---

### E101: Invalid Type Conversion

**Description**: Attempted type conversion is not allowed.

**Common Fixes**:

- Use valid conversion functions
- Check if conversion is necessary
- Verify source and target types

---

### E102: Type Inference Failed

**Description**: The type system cannot infer the type of an expression.

**Common Fixes**:

- Add explicit type annotations
- Provide more context
- Simplify the expression

---

### E103: Invalid Operand Type

**Description**: An operator is used with an incompatible operand type.

**Common Fixes**:

- Check operator requirements
- Use correct operand types
- Review operator documentation

---

### E104: Invalid Comparison Type

**Description**: Types cannot be compared with the given operator.

**Common Fixes**:

- Use comparable types
- Check comparison operator
- Add type conversions if needed

---

## E200-E299: Unit and Dimension Errors

### E200: Dimension Mismatch

**Description**: Operations on quantities with incompatible dimensions.

**Example**:

```sea
Policy dimension_collision as:
  10 "kg" + 5 "meters"
// Error: Cannot add Mass and Length
```

**Common Fixes**:

- Use quantities with the same dimension
- Add unit conversions
- Review calculation logic

---

*Note*: `E003: Unit Mismatch` was historically listed in the Syntax/Parsing category (E003) and has been consolidated under the Unit/Dimension category. Consult `E200`/`E203` for dimension/unit mismatches.

### E201: Invalid Unit

**Description**: A unit is not recognized or is invalid in context.

**Common Fixes**:

- Use standard unit names
- Check unit definitions
- Verify unit spelling

---

### E202: Unit Conversion Failed

**Description**: Cannot convert between the specified units.

**Common Fixes**:

- Ensure units are convertible
- Check conversion factors
- Verify dimension compatibility

---

### E203: Incompatible Dimensions

**Description**: Dimensions cannot be used together in the operation.

**Common Fixes**:

- Use compatible dimensions
- Review operation requirements
- Add dimension conversions

---

## E300-E399: Scope and Reference Errors

### E300: Variable Not In Scope

**Description**: A variable is referenced outside its scope.

**Example**:

```sea
Policy scoped_check as:
  exists e in entities: (e.name = "service")

Policy leak_scope as:
  e.name = "api" // Error: `e` is not in scope here
```

**Common Fixes**:

- Move variable reference inside scope
- Expand variable scope
- Define variable in outer scope

**Suggestion**: Shows available variables in scope.

---

### E301: Undefined Reference

**Description**: General reference to an undefined item.

**Common Fixes**:

- Define the referenced item
- Check spelling and case
- Verify namespace

---

### E302: Circular Reference

**Description**: A circular dependency exists between items.

**Example**:

```sea
Entity "A"
Entity "B"
Flow "Data" from "A" to "B" quantity 1
Flow "Data" from "B" to "A" quantity 1
// Error: circular dependency between A and B
```

**Common Fixes**:

- Break the circular dependency
- Restructure definitions
- Use forward declarations if supported

---

### E303: Invalid Reference

**Description**: A reference is malformed or points to an invalid target.

**Common Fixes**:

- Check reference syntax
- Verify target exists
- Review reference path

---

## E400-E499: Policy Validation Errors

### E400: Policy Evaluation Failed

**Description**: A policy could not be evaluated.

**Common Fixes**:

- Check policy expression syntax
- Verify policy inputs
- Review policy logic

---

### E401: Invalid Policy Expression

**Description**: A policy expression is invalid or malformed.

**Common Fixes**:

- Review policy syntax
- Check expression operators
- Verify operand types

---

### E402: Determinism Violation

**Description**: A policy or expression is non-deterministic when determinism is required.

**Common Fixes**:

- Remove non-deterministic operations
- Use deterministic alternatives
- Review determinism requirements

**Suggestion**: Provides hints on making the code deterministic.

---

### E403: Invalid Modality

**Description**: A policy modality (must, should, may) is invalid in context.

**Common Fixes**:

- Use correct modality
- Review policy requirements
- Check modality syntax

---

## E500-E599: Namespace and Module Errors

### E500: Namespace Not Found

**Description**: A referenced namespace does not exist.

**Common Fixes**:

- Define the namespace
- Check namespace spelling
- Verify namespace imports

---

### E501: Ambiguous Namespace

**Description**: A namespace reference is ambiguous (multiple matches).

**Common Fixes**:

- Use fully qualified names
- Add namespace aliases
- Resolve namespace conflicts

---

### E502: Invalid Namespace

**Description**: A namespace definition or reference is invalid.

**Common Fixes**:

- Check namespace syntax
- Verify namespace rules
- Review namespace structure

---

### E503: Module Not Found

**Description**: A referenced module cannot be found.

**Common Fixes**:

- Check module path
- Verify module exists
- Review import statements

---

## Using Error Codes

### In CLI

```bash
# Get detailed error information
sea validate myfile.sea

# JSON output with error codes
sea validate --format json myfile.sea

# LSP format for IDE integration
sea validate --format lsp myfile.sea
```

### In Python

```python
from sea_dsl import Graph

try:
    graph = Graph.parse(source)
except Exception as e:
    print(f"Error {e.code}: {e}")
    if hasattr(e, 'suggestion'):
        print(f"Suggestion: {e.suggestion}")
```

### In TypeScript

```typescript
import { Graph } from "sea-dsl";

try {
  const graph = Graph.parse(source);
} catch (e: any) {
  const metadata = JSON.parse(e.message.split("__metadata__: ")[1]);
  console.error(`Error ${metadata.code}: ${metadata.errorType}`);
  if (metadata.suggestion) {
    console.log(`Suggestion: ${metadata.suggestion}`);
  }
}
```

### In WASM/JavaScript

```javascript
import { Graph } from "sea-dsl-wasm";

try {
  const graph = Graph.parse(source);
} catch (e) {
  // Different bindings emit metadata tokens with different markers.
  // Node/NAPI wrappers commonly use "__metadata__: " while WASM/json errors may use
  // "__SEA_DSL_ERROR_METADATA__: ". Try both when extracting metadata.
  const raw = e.message || e.toString();
  const token = raw.includes("__SEA_DSL_ERROR_METADATA__:") ? "__SEA_DSL_ERROR_METADATA__: " : "__metadata__: ";
  const [msg, meta] = raw.split(token);
  const metadata = JSON.parse(meta);
  console.error(`Error ${metadata.code}: ${msg}`);
  if (metadata.suggestion) {
    console.log(`Suggestion: ${metadata.suggestion}`);
  }
}
```

---

## Contributing

When adding new error codes:

1. Choose the appropriate category range
2. Add the error code to `ErrorCode` enum in `validation_error.rs`
3. Implement `as_str()` and `description()` methods
4. Update this documentation
5. Add tests for the new error code
6. Update language binding error converters if needed

---

## See Also

- [Diagnostic Formatters](../README.md#diagnostic-formatters)
- [Fuzzy Matching](../README.md#fuzzy-matching)
- [Language Bindings](../README.md#language-bindings)

## Binding surfaces

Python bindings surface validation errors as `ValidationError` with `.code` and `.message` fields. TypeScript bindings throw an `Error` where the message includes the code (e.g., `E201`). Use the tables above to map messages to remediation steps.

## Integration tips

- Enable `--format json` when using the CLI to capture structured error reports for CI pipelines.
- Log the `help` string alongside `code` in downstream tooling to aid non-DSL users.

## Related documentation

- `docs/new_docs/reference/cli-commands.md` — `validate` and `explain` commands that emit these codes.
- `docs/new_docs/how-tos/debugging-parser-failures.md` — troubleshooting workflows.
- `docs/new_docs/reference/primitives-api.md` — context for identifiers referenced in errors.
