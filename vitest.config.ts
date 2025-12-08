import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node', // Keep node environment for NAPI-RS compatibility
    include: ['typescript-tests/**/*.test.ts'],
    // Single-threaded for native binding stability
    testTimeout: 10000,
  },
});
