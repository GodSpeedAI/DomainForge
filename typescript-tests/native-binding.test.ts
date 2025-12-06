import { describe, expect, it } from 'vitest';
import { validateNativeExports } from '../lib/validate_native_exports';

describe('validateNativeExports helper', () => {
    it('throws a helpful error when required exports are missing', () => {
        const fakeBinding = { Graph: {} }; // minimal missing many required symbols
        const required = ['Graph', 'Entity', 'Resource'];
        expect(() => validateNativeExports(fakeBinding, required)).toThrowError(/Missing required export\(s\): Entity, Resource|missing required export/i);
    });

    it('does not throw when required exports are present', () => {
        const fakeBinding = {
            Graph: {},
            Entity: {},
            Resource: {},
        };
        const required = ['Graph', 'Entity', 'Resource'];
        expect(() => validateNativeExports(fakeBinding, required)).not.toThrow();
    });
});
