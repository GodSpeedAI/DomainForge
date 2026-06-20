# Migrating from `@godspeedai/sea` to `@godspeedai/domainforge` (npm)

The TypeScript/npm package was renamed:

| Before                       | After                          |
|------------------------------|--------------------------------|
| `@godspeedai/sea` (npm)      | `@godspeedai/domainforge`      |
| native binary `sea-core.*.node` | `domainforge-core.*.node`   |

## Install

```bash
# old: npm install @godspeedai/sea
npm install @godspeedai/domainforge
```

## Import

```ts
// old: import { Graph, Entity } from '@godspeedai/sea';
import { Graph, Entity } from '@godspeedai/domainforge';
```

The API surface is otherwise unchanged.
