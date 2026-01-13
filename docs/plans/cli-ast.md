# CLI AST Emission Plan

## Summary

The current `sea` CLI emits **AST v2** (legacy map-based structure). SEA consumes **AST v3** (declarations list), so we rely on `tools/ast_v2_to_v3.py` as a bridge. This works, but **flow annotations are not preserved** in v2 output, which makes downstream linting and IR generation incomplete.

## Current Behavior

- CLI emits v2 AST (`entities/resources/flows` maps).
- Flow `attributes` are always `{}` even when annotations are present in the SEA-DSL.
- SEA converts v2 → v3 via `tools/ast_v2_to_v3.py`, but the annotations are already lost.

## Required Fix

**Goal:** Update CLI to emit **AST v3** directly with full annotations on flows.

Steps:
1. Update `sea-core` parser/serializer to expose flow annotations in the AST (not just entities/resources).
2. Add a CLI flag or default output mode for **AST v3**.
3. Ensure emitted AST schema matches `tools/schemas/ast-v3.schema.json` in SEA.
4. Add regression tests: parse a SEA file with `@cqrs`, `@tx`, `@idempotency`, `@read_model` and verify they appear in flow annotations.

## Impact

- Removes the v2 → v3 shim in SEA.
- Enables correct flow linting and CQRS classification without SDS fallback.
- Unblocks future AST v4 migrations cleanly.

it should actually emit AST v3 directly with full flow annotations (including the new custom annotations capability).
