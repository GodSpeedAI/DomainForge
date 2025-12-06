# TypeScript Binding Quickstart

This guide demonstrates how to use DomainForge in a Node.js/TypeScript environment.

## Prerequisites

- Node.js 16+
- npm or yarn

## Step 1: Installation

```bash
npm install @domainforge/sea
# Or if building locally:
# npm install ../sea-core/typescript
```

## Step 2: Create a Script

Create `index.ts`.

```typescript
import { parse, Model, Flow } from '@domainforge/sea';

const seaContent = `
entity API { type = "service" }
resource Bucket { type = "storage" }
flow upload { from = API, to = Bucket, interaction = "write" }
`;

async function main() {
  try {
    // 1. Parse
    const model: Model = parse(seaContent);
    console.log(`Parsed model with ${model.entities.length} entities.`);

    // 2. Analyze
    model.flows.forEach((flow: Flow) => {
      console.log(`Found flow: ${flow.name}`);
      console.log(`  From: ${flow.from.name}`);
      console.log(`  To: ${flow.to.name}`);
    });

  } catch (error) {
    console.error("Failed to parse SEA model:", error);
  }
}

main();
```

## Step 3: Run

```bash
npx ts-node index.ts
```

**Expected Output:**
```text
Parsed model with 1 entities.
Found flow: upload
  From: API
  To: Bucket
```

## Integration with Vitest

```typescript
// architecture.test.ts
import { describe, it, expect, beforeAll } from 'vitest';
import { parse } from '@domainforge/sea';
import * as fs from 'fs';

describe('Architecture Rules', () => {
  let model;
  beforeAll(() => {
    model = parse(fs.readFileSync('./system.sea', 'utf-8'));
  });

  it('should have all databases encrypted', () => {
    const dbs = model.resources.filter(r => r.type === 'database');
    dbs.forEach(db => {
      expect(db.properties.encrypted).toBe(true);
    });
  });
});
```

## See Also

- [Cross-Language Binding Strategy](../explanations/cross-language-binding-strategy.md)
