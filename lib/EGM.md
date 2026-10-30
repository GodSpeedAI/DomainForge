# lib — Native Binding Export Guard

## Overview
A single CommonJS helper plus its type declaration. It fails loudly when a
napi-rs native addon did not export a symbol the JavaScript wrapper requires,
replacing an opaque `undefined is not a function` with a descriptive error.

## Interface
- `[validate_native_exports.js:validateNativeExports]` —
  `validateNativeExports(nativeBinding, requiredExports)`; takes the loaded native
  binding (`Record<string, unknown> | null | undefined`) and an array of required
  export names, and returns `void`.
- `[validate_native_exports.js:module.exports]` — exports `{ validateNativeExports }`
  as a CommonJS named export.
- `[validate_native_exports.d.ts:validateNativeExports]` — TypeScript declaration:
  `(nativeBinding: Record<string, unknown> | null | undefined, requiredExports:
  string[]) => void`.

## Invariants
- `[validate_native_exports.js:validateNativeExports]` — a `null` or `undefined`
  binding makes every required export "missing", so a failed `require` of the
  addon reports the full required list rather than throwing a `TypeError`.
- `[validate_native_exports.js:Object.prototype.hasOwnProperty.call]` — presence is
  tested with an own-property check, so an inherited `Object.prototype` member
  (e.g. `toString`) never satisfies a required export name.
- `[validate_native_exports.js:missing.length]` — the function throws only when the
  missing list is non-empty; a fully-populated binding passes silently (returns
  `undefined`, no truthy return value).
- `[validate_native_exports.js:new Error]` — the thrown message is exactly
  `Native binding missing required export(s): <names>. Ensure the native addon was
  built and exposes these symbol(s).` with names comma+space joined.

## Side Effects
- `[validate_native_exports.js:throw new Error]` — throws when required exports are
  missing; the only observable effect, no I/O or global mutation.

## Verification
- Run: `just ts-test`
- Run: `bun test`
