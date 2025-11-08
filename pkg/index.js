import init, * as wasm from './sea_core.js';

let wasmModule = null;
let initPromise = null;

async function ensureInitialized() {
  if (wasmModule) return wasmModule;
  if (initPromise) return initPromise;

  initPromise = init().then(() => {
    wasmModule = wasm;
    return wasmModule;
  });

  return initPromise;
}

export async function loadWasm() {
  return ensureInitialized();
}

export const Entity = new Proxy({}, {
  construct(target, args) {
    return (async () => {
      const mod = await ensureInitialized();
      return new mod.Entity(...args);
    })();
  },
  get(target, prop) {
    if (prop === 'name') return 'Entity';
    return async (...args) => {
      const mod = await ensureInitialized();
      const EntityClass = mod.Entity;
      if (typeof EntityClass[prop] === 'function') {
        return EntityClass[prop](...args);
      }
      return EntityClass[prop];
    };
  }
});

export const Resource = new Proxy({}, {
  construct(target, args) {
    return (async () => {
      const mod = await ensureInitialized();
      return new mod.Resource(...args);
    })();
  },
  get(target, prop) {
    if (prop === 'name') return 'Resource';
    return async (...args) => {
      const mod = await ensureInitialized();
      const ResourceClass = mod.Resource;
      if (typeof ResourceClass[prop] === 'function') {
        return ResourceClass[prop](...args);
      }
      return ResourceClass[prop];
    };
  }
});

export const Flow = new Proxy({}, {
  construct(target, args) {
    return (async () => {
      const mod = await ensureInitialized();
      return new mod.Flow(...args);
    })();
  },
  get(target, prop) {
    if (prop === 'name') return 'Flow';
    return async (...args) => {
      const mod = await ensureInitialized();
      const FlowClass = mod.Flow;
      if (typeof FlowClass[prop] === 'function') {
        return FlowClass[prop](...args);
      }
      return FlowClass[prop];
    };
  }
});

export const Instance = new Proxy({}, {
  construct(target, args) {
    return (async () => {
      const mod = await ensureInitialized();
      return new mod.Instance(...args);
    })();
  },
  get(target, prop) {
    if (prop === 'name') return 'Instance';
    return async (...args) => {
      const mod = await ensureInitialized();
      const InstanceClass = mod.Instance;
      if (typeof InstanceClass[prop] === 'function') {
        return InstanceClass[prop](...args);
      }
      return InstanceClass[prop];
    };
  }
});

export const Graph = new Proxy({}, {
  construct(target, args) {
    return (async () => {
      const mod = await ensureInitialized();
      return new mod.Graph(...args);
    })();
  },
  get(target, prop) {
    if (prop === 'name') return 'Graph';
    return async (...args) => {
      const mod = await ensureInitialized();
      const GraphClass = mod.Graph;
      if (typeof GraphClass[prop] === 'function') {
        return GraphClass[prop](...args);
      }
      return GraphClass[prop];
    };
  }
});

export async function preloadWasm() {
  return ensureInitialized();
}
