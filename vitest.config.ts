import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node', // Keep node environment for NAPI-RS compatibility
    include: ['typescript-tests/**/*.test.ts'],
    // Single-threaded for native binding stability
    pool: 'forks',
    fileParallelism: false,
    testTimeout: 10000,
  },
});
