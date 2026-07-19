import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { Graph } from '../index';

/**
 * Cross-binding byte parity (M0 gate finding 2): the TypeScript binding
 * must produce byte-identical canonical JSON to the Rust golden. The same
 * constants appear in:
 *   - Rust:   domainforge-core/tests/application_cross_binding_golden_tests.rs
 *   - Python: tests/test_parser.py::test_cross_binding_contract_bytes_match_rust_golden
 *   - WASM:   domainforge-core/tests/wasm_tests.rs (cross_binding_golden_hashes)
 *
 * If serialization intentionally changes, regenerate all four in lockstep.
 */
const CONTRACT_GOLDEN_SHA256 =
    'sha256:57c81f0cddc0cec87eaef86cca6692134376076620c9c73844b016869cc31640';

describe('cross-binding byte parity (ADR-013 M0 gate finding 2)', () => {
    const fixtureRoot = join(__dirname, '..', 'fixtures', 'application_generation', 'flagship');
    const sourcesJson = JSON.stringify({
        'flagship/command-write.sea': readFileSync(join(fixtureRoot, 'command-write.sea'), 'utf8'),
        'flagship/query-read.sea': readFileSync(join(fixtureRoot, 'query-read.sea'), 'utf8'),
    });

    it('typescript contract bytes match the rust golden hash', () => {
        const raw = Graph.resolveApplicationContractJson('flagship/query-read.sea', sourcesJson);
        const digest = 'sha256:' + createHash('sha256').update(raw, 'utf8').digest('hex');
        expect(digest).toBe(CONTRACT_GOLDEN_SHA256);
    });
});
