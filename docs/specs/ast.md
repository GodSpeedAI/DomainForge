# AST Specification

## Overview

The Abstract Syntax Tree (AST) represents the structure of a SEA DSL file. It is defined in `sea-core/src/parser/ast.rs`.

## Structure

The `Ast` struct contains:

- `metadata`: File-level metadata (namespace, version, owner).
- `declarations`: A list of `AstNode`s.

### AstNode Variants

- `Entity`: Defines an entity.
- `Resource`: Defines a resource.
- `Flow`: Defines a flow of resources.
- `Policy`: Defines a policy rule.
- `Dimension`: Defines a custom dimension.
- `UnitDeclaration`: Defines a custom unit.

## JSON Schema

The AST structure is formally defined in `schemas/ast-v1.schema.json`. This schema can be used to validate exported ASTs.

## Export and Import

- **Export**: The `PrettyPrinter` (`sea-core/src/parser/printer.rs`) converts an AST back to DSL source code.
- **Import**: The `SeaParser` (`sea-core/src/parser/mod.rs`) parses DSL source code into an AST.
