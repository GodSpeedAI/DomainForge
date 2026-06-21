# Migrating from `@godspeedai/sea` to `domainforge` (npm)

The TypeScript/npm package was renamed:

| Before                       | After                          |
|------------------------------|--------------------------------|
| `@godspeedai/sea` (npm)      | `domainforge`      |
| native binary `sea-core.*.node` | `domainforge-core.*.node`   |

## Install

```bash
# old: npm install @godspeedai/sea
npm install domainforge
```

## Import

```ts
// old: import { Graph, Entity } from '@godspeedai/sea';
import { Graph, Entity } from 'domainforge';
```

The API surface is otherwise unchanged.
