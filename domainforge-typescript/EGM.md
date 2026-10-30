# domainforge-typescript — TypeScript Distribution (`@godspeedai/domainforge`)

## Overview
Distribution root for the npm package `@godspeedai/domainforge` (version
`0.21.2`, `main: index.js`, `types: index.d.ts`, Apache-2.0,
`engines.node >= 18`, `engines.bun >= 1.1.0`). `index.js` is an auto-generated
napi-rs loader that resolves a compiled addon and re-exports its surface;
`index.d.ts` is the auto-generated type surface. Both are projections of the
canonical Rust core (`../domainforge-core/Cargo.toml`, features `typescript`
and `three_valued_logic`). The `*.node` binaries
(`domainforge-core.linux-x64-gnu.node`, `sea-core.linux-x64-gnu.node`) are
gitignored build outputs, not source.

## Interface
- `[index.js:module.exports]` — re-exports the loaded addon's surface: classes
  `ConceptChange`, `DeclaredDimension`, `DeclaredUnit`, `Dimension`, `Entity`,
  `Expression`, `Flow`, `Graph`, `Instance`, `Mapping`, `Metric`,
  `NamespaceBinding`, `NamespaceRegistry`, `NormalizedExpression`, `Pattern`,
  `Policy`, `Projection`, `Relation`, `Resource`, `ResourceInstance`, `Role`,
  `Unit`; enums `AggregateFunction`, `AliasStatus`, `ApprovalState`,
  `BinaryOp`, `ClaimLevel`, `ConceptKind`, `ConceptStatus`,
  `DiagnosticSeverity`, `FinalDecision`, `PolicyModality`, `Quantifier`,
  `SemanticTruth`, `SemanticValidationStatus`, `Severity`, `SignatureState`,
  `SourceClass`, `UnaryOp`, `ValidationMode`; functions `checkFormat`,
  `evaluateAuthority`, `formatSource`, `semanticNormalizeKey`,
  `semanticPackBuild`, `semanticPackDiff`, `semanticPackHash`,
  `semanticPackSign`, `semanticPackValidate`, `semanticPackValidateGraph`,
  `semanticPackVerify`, `semanticResolveConcept`; and the const
  `__napiBindingTarget`.
- `[index.js:requireNative]` — platform/arch dispatch that `require`s the local
  `./domainforge-core.<platform>-<arch>.node` artifact or the matching
  `@godspeedai/domainforge-*` optional dependency.
- `[index.js:__napiStampBindingTarget]` — stamps `__napiBindingTarget` (`'native'`
  or the WASI flavor) onto the loaded binding.
- `[index.d.ts:Graph]` — `parse`, `parseToAstJson`, `parseSourceMap`,
  `resolveApplicationContractJson`, `domainModelIdentityJson`,
  `worldRefFromIdentityJson`, `verifyWorldRef`, `parseWorldRef`, `add*`,
  `has*`, `get*`, `find*ByName`, `flowsFrom`/`flowsTo`, `all*`, `*Count`,
  `evaluatePolicy`, `setEvaluationMode`, `useThreeValuedLogic`,
  `exportCalm`/`importCalm`, `exportProtobuf`, `exportAiLearning`,
  `exportLean`, `exportRdfProjection`, `exportBpmn`, `exportCmmn`,
  `exportArchimate`, `exportOtelSemconv`, `exportBaml`, `exportDspy`,
  `exportZenml`.
- `[index.d.ts:Expression]` — static builders `literal`, `literalBool`,
  `literalNumber`, `literalString`, `variable`, `quantity`, `time`, `interval`,
  `binary`, `unary`, `cast`, `quantifier`, `memberAccess`, `aggregation`,
  `aggregationComprehension`, `groupBy`; instance methods `normalize`,
  `isEquivalent`, `toStringRepr`, `equals`.
- `[index.d.ts:NormalizedExpression]` — `stableHash`, `stableHashHex`,
  `innerExpression`, `toStringRepr`, `equals`.
- `[index.d.ts:NamespaceRegistry]` — `fromFile`, `discover`, `resolveFiles`,
  `namespaceFor`, `root`, `defaultNamespace`.
- `[index.d.ts:EvaluateAuthorityResult]` — `{ traceJson, decisionJson }`, returned
  by `[index.d.ts:evaluateAuthority]`.
- `[index.d.ts:EvaluationResult]` — `isSatisfied`, optional
  `isSatisfiedTristate`, `violations: Array<Violation>`, returned by
  `[index.d.ts:Graph.evaluatePolicy]`.
- `[index.d.ts:FormatOptions]` — `indentWidth`, `useTabs`, `preserveComments`,
  `sortImports`, consumed by `[index.d.ts:formatSource]` and
  `[index.d.ts:checkFormat]`.
- `[index.d.ts:WindowSpec]` — `{ duration, unit }`.
- Child contracts: none — this directory has no subdirectories; the binding
  implementation lives in `domainforge-core/src/typescript/`.

## Invariants
- `[index.js:__napiStampBindingTarget]` — `__napiBindingTarget` is reserved: if the
  loaded addon already exports a *different* value, the load throws with
  `error.code = 'ERR_NAPI_BINDING_TARGET_CONFLICT'`.
- `[index.js:__napiStampBindingTarget]` — on a non-extensible (sealed/frozen)
  export object the stamp is skipped rather than failing an otherwise
  successful load.
- `[index.js:__napiWasiFlavorRequested]` — an unknown `NAPI_RS_WASI_FLAVOR` throws
  `Unsupported WASI flavor "..."`; the accepted set (`__napiWasiFlavors`) is only
  `wasm32-wasi`.
- `[index.js:forceWasiError]` — with `NAPI_RS_FORCE_WASI=error` or an explicit
  flavor, a WASI load failure throws and never falls back to native.
- `[index.js:requireNative]` — the `@godspeedai/domainforge-*` package version must
  equal `0.21.0`, checked only when `NAPI_RS_ENFORCE_VERSION_CHECK` is set and not
  `'0'`.
- `[index.d.ts:__napiBindingTarget]` — the declared type is a closed union
  `'native' | 'wasm32-wasi' | 'wasm32-wasip1'` (the loader's accepted flavor set
  is narrower; see `__napiWasiFlavors`).

## Side Effects
- `[index.js:requireNative]` — reads the filesystem/`process.report`
  (`isMusl`, `isMuslFromFilesystem`, `isMuslFromReport`) and calls
  `require('child_process').execSync('ldd --version')` (`isMuslFromChildProcess`)
  to detect musl libc.
- `[index.js:requireNative]` — loads native addon binaries at module-import time
  (`require('./domainforge-core.<target>.node')` and optional `@godspeedai/*`
  packages).
- `[index.js:__napiWasiResolveCandidate]` — calls `require.resolve` to probe for
  WASI artifacts and `require('./domainforge-core.wasi.cjs')` to load them.
- `[index.js:module.exports.__napiBindingTarget]` — mutates the module's own
  export object before aliasing `module.exports = nativeBinding`.
- `[index.js:isMuslFromReport]` — sets `process.report.excludeNetwork = true`.
- `[index.js:createLoadErrorChain]` — attaches the aggregated per-candidate load
  errors as `error.cause` on the thrown `Cannot find native binding` error.

## Verification
- Run: `just ts-test`
- Types: `cd domainforge-typescript && bun run test:types`

