# Reference: Error Code Catalog

This catalog documents the complete set of error codes and diagnostic codes emitted by DomainForge's validation and compilation engines.

---

## 1. Error Categories Overview

- **E001 – E099**: Syntax and Parsing Errors
- **E100 – E199**: Type System Errors
- **E200 – E299**: Unit and Dimension Errors
- **E300 – E399**: Scope and Reference Errors
- **E400 – E499**: Policy Validation Errors
- **E500 – E599**: Namespace and Module Errors
- **APP001 – APP014**: Application Contract Diagnostics (ADR-013)

---

## 2. E001 – E099: Syntax & Parsing Errors

| Code | Name | Description | Example / Fix |
|---|---|---|---|
| `E001` | `UndefinedEntity` | Reference to an entity that has not been declared. | **Fix**: Declare `Entity "Name"` before referencing in flows or relations. Fuzzy match suggestions are provided. |
| `E002` | `UndefinedResource` | Reference to a resource that has not been declared. | **Fix**: Declare `Resource "Name" <unit>` before referencing in flows. |
| `E005` | `SyntaxError` | General syntax error violating the PEG grammar in `sea.pest`. | **Fix**: Check for unclosed brackets, missing keywords, or invalid token sequences around reported line/col. |
| `E006` | `InvalidExpression` | Malformed policy expression or unsupported operator combination. | **Fix**: Verify operator precedence and syntax in expressions. |
| `E007` | `DuplicateDeclaration` | Duplicate declaration of the same concept name within a namespace. | **Fix**: Rename or remove the duplicate declaration. |
| `E008` | `UndefinedVariable` | Variable referenced in quantifier or aggregation is not in scope. | **Fix**: Ensure variable name matches the collection iterator variable (`forall x in flows: x.quantity > 0`). |
| `E009` | `InvalidQuantity` | Malformed quantity literal or negative quantity where disallowed. | **Fix**: Ensure numbers are formatted correctly with valid unit suffixes. |
| `E010` | `InvalidIdentifier` | Identifier contains illegal characters or starts with a digit. | **Fix**: Use letters, digits, and underscores starting with a letter or underscore. |

---

## 3. E100 – E199: Type System Errors

| Code | Name | Description | Example / Fix |
|---|---|---|---|
| `E100` | `IncompatibleTypes` | Operation attempted between incompatible scalar types (e.g. `string + int`). | **Fix**: Explicitly cast types or check operand types. |
| `E101` | `InvalidTypeCast` | Cast operator (`as`) applied to an unsupported type conversion. | **Fix**: Verify supported cast conversions. |
| `E102` | `TypeInferenceFailure` | The engine could not deduce the resulting type of a complex expression. | **Fix**: Break expression into smaller sub-expressions or add explicit type constraints. |

---

## 4. E200 – E299: Unit & Dimension Errors

| Code | Name | Description | Example / Fix |
|---|---|---|---|
| `E003` / `E200` | `UnitMismatch` | A quantity or flow uses a unit incompatible with the declared resource. | **Fix**: Ensure flow quantity unit matches the unit declared on the resource (e.g. `Flow "Payment" ... quantity 100 USD`). |
| `E201` | `UndefinedUnit` | Reference to an unregistered unit. | **Fix**: Declare the unit using `unit "Name" of "Dimension" ...` or check spelling. |
| `E202` | `UndefinedDimension` | Reference to an unregistered dimension. | **Fix**: Declare the dimension using `dimension "Name"`. |
| `E203` | `IncompatibleDimensions` | Arithmetic attempted across incompatible dimensions (e.g. Mass + Currency). | **Fix**: Dimensional algebra prevents adding disparate dimensions. |

---

## 5. APP001 – APP014: Application Contract Diagnostics

Emitted when compiling ADR-013 Application Contracts:

| Code | Name | Condition & Fix |
|---|---|---|
| `APP001` | `MissingEntityKey` | Entity state body is missing a designated `key` field, or declares multiple keys. Every entity body must have exactly one `key field: type`. |
| `APP002` | `UnresolvedSymbol` | Field type or constraint references an unknown record, enum, or pattern. Declare the referenced type. |
| `APP003` | `ConflictingEffect` | Operation effect conflicts with intent or direction (e.g. `effect mutates` on a read-only transaction). |
| `APP004` | `EntityAsPayload` | Operation input or output references an `Entity` instead of a `record`. Operation payloads must be identityless records. |
| `APP005` | `MissingIdempotency` | Non-read-only operation lacks an `idempotency` clause. Add `idempotency keyed_by <field>` or `idempotency inherent`. |
| `APP006` | `ConcurrencyMismatch` | Declared concurrency strategy is incompatible with transaction boundary. |
| `APP014` | `ModuleResolutionCycle` | Transitive import cycle detected (e.g. A imports B, B imports A) or imported symbol collision. Refactor shared symbols into a separate common module. |

---

## Source Trail
- `domainforge-core/src/validation_error.rs` — ErrorCode enum and formatting
- `domainforge-core/src/application/diagnostic.rs` — ApplicationDiagnosticCode implementation
- `domainforge-core/tests/phase_15_validation_error_tests.rs` — Error diagnostic tests
