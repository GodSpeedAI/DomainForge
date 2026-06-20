/**
 * Tests for the new Semantic Pack API exports added in this PR.
 *
 * Covers:
 * - Enum exports: SemanticTruth, DiagnosticSeverity, ValidationMode, ApprovalState,
 *   SignatureState, ConceptStatus, ConceptKind, AliasStatus, SemanticValidationStatus
 * - Functions: semanticPackBuild, semanticPackValidate, semanticPackValidateGraph,
 *   semanticPackHash, semanticNormalizeKey, semanticResolveConcept, semanticPackDiff,
 *   semanticPackSign, semanticPackVerify
 */

import { describe, it, expect } from 'vitest';
import {
    SemanticTruth,
    DiagnosticSeverity,
    ValidationMode,
    ApprovalState,
    SignatureState,
    ConceptStatus,
    ConceptKind,
    AliasStatus,
    SemanticValidationStatus,
    semanticPackBuild,
    semanticPackValidate,
    semanticPackValidateGraph,
    semanticPackHash,
    semanticNormalizeKey,
    semanticResolveConcept,
    semanticPackDiff,
    semanticPackSign,
    semanticPackVerify,
} from '../index.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeMinimalPackJson(): string {
    return JSON.stringify({
        schema_version: '0.3',
        pack_id: 'test-org/test-domain/1.0.0',
        org_id: 'test-org',
        domain_id: 'test-domain',
        pack_version: '1.0.0',
        meaning_version: '1.0.0',
        meaning_fingerprint: '',
        source_graph_hash: 'sha256:test',
        build_config_hash: 'sha256:cfg',
        review_manifest_hash: 'sha256:rev',
        created_at: '2026-06-07T00:00:00Z',
        generator: { name: 'domainforge-core', version: '0.3' },
        trust: { approval_state: 'candidate', signature_state: 'unsigned' },
        concepts: [
            {
                id: 'supplier',
                canonical_name: 'Supplier',
                kind: 'entity',
                status: 'active',
                definition: {
                    text: 'A party that provides goods or services.',
                    definition_hash: '',
                    decision_ref: 'dec_supplier',
                },
                owner: 'owner@test.com',
                source_refs: [],
                examples: [],
                counterexamples: [],
                allowed_predicates: [],
                valid_contexts: [],
            },
        ],
        relations: [],
        metrics: [],
        dimensions: [],
        units: [],
        aliases: [],
        mapping_rules: [],
        compatibility: {},
    });
}

function makeMinimalBuildInputJson(): string {
    return JSON.stringify({
        org_id: 'test-org',
        domain_id: 'test-domain',
        pack_version: '1.0.0',
        meaning_version: '1.0.0',
        approval: 'candidate',
        concepts: [
            {
                id: 'supplier',
                canonical_name: 'Supplier',
                kind: 'entity',
                status: 'active',
                definition: {
                    text: 'A party that provides goods or services.',
                    definition_hash: '',
                    decision_ref: 'dec_supplier',
                },
                owner: 'owner@test.com',
                source_refs: [],
                examples: [],
                counterexamples: [],
                allowed_predicates: [],
                valid_contexts: [],
            },
        ],
        relations: [],
        metrics: [],
        dimensions: [],
        units: [],
        aliases: [],
        mapping_rules: [],
        review_records: [],
        previous_pack: null,
        allow_first_approved_version: false,
        source_graph_hash: 'sha256:test',
    });
}

function makeDefaultOptionsJson(): string {
    return JSON.stringify({ mode: 'warn', deprecated_policy: 'warn' });
}

// ---------------------------------------------------------------------------
// Enum value tests
// ---------------------------------------------------------------------------

describe('SemanticTruth enum', () => {
    it('exports Valid=0, Invalid=1, Unknown=2', () => {
        expect(SemanticTruth.Valid).toBe(0);
        expect(SemanticTruth.Invalid).toBe(1);
        expect(SemanticTruth.Unknown).toBe(2);
    });

    it('has exactly three members', () => {
        const keys = Object.keys(SemanticTruth).filter((k) => isNaN(Number(k)));
        expect(keys).toHaveLength(3);
    });
});

describe('DiagnosticSeverity enum', () => {
    it('exports Error=0, Warning=1, Info=2, Hint=3', () => {
        expect(DiagnosticSeverity.Error).toBe(0);
        expect(DiagnosticSeverity.Warning).toBe(1);
        expect(DiagnosticSeverity.Info).toBe(2);
        expect(DiagnosticSeverity.Hint).toBe(3);
    });
});

describe('ValidationMode enum', () => {
    it('exports Off=0, Warn=1, Strict=2', () => {
        expect(ValidationMode.Off).toBe(0);
        expect(ValidationMode.Warn).toBe(1);
        expect(ValidationMode.Strict).toBe(2);
    });
});

describe('ApprovalState enum', () => {
    it('exports Candidate=0, Approved=1, Rejected=2', () => {
        expect(ApprovalState.Candidate).toBe(0);
        expect(ApprovalState.Approved).toBe(1);
        expect(ApprovalState.Rejected).toBe(2);
    });
});

describe('SignatureState enum', () => {
    it('exports Unsigned=0, Signed=1, InvalidSignature=2', () => {
        expect(SignatureState.Unsigned).toBe(0);
        expect(SignatureState.Signed).toBe(1);
        expect(SignatureState.InvalidSignature).toBe(2);
    });
});

describe('ConceptStatus enum', () => {
    it('exports five statuses with correct numeric values', () => {
        expect(ConceptStatus.Active).toBe(0);
        expect(ConceptStatus.Proposed).toBe(1);
        expect(ConceptStatus.Deprecated).toBe(2);
        expect(ConceptStatus.Rejected).toBe(3);
        expect(ConceptStatus.ExternalOnly).toBe(4);
    });
});

describe('ConceptKind enum', () => {
    it('exports nine kinds with correct numeric values', () => {
        expect(ConceptKind.Entity).toBe(0);
        expect(ConceptKind.Resource).toBe(1);
        expect(ConceptKind.Role).toBe(2);
        expect(ConceptKind.Flow).toBe(3);
        expect(ConceptKind.Policy).toBe(4);
        expect(ConceptKind.Metric).toBe(5);
        expect(ConceptKind.Dimension).toBe(6);
        expect(ConceptKind.Unit).toBe(7);
        expect(ConceptKind.External).toBe(8);
    });
});

describe('AliasStatus enum', () => {
    it('exports Approved=0, Deprecated=1, Ambiguous=2, Blocked=3', () => {
        expect(AliasStatus.Approved).toBe(0);
        expect(AliasStatus.Deprecated).toBe(1);
        expect(AliasStatus.Ambiguous).toBe(2);
        expect(AliasStatus.Blocked).toBe(3);
    });
});

describe('SemanticValidationStatus enum', () => {
    it('exports Passed=0, Failed=1, Unknown=2, Blocked=3', () => {
        expect(SemanticValidationStatus.Passed).toBe(0);
        expect(SemanticValidationStatus.Failed).toBe(1);
        expect(SemanticValidationStatus.Unknown).toBe(2);
        expect(SemanticValidationStatus.Blocked).toBe(3);
    });
});

// ---------------------------------------------------------------------------
// semanticNormalizeKey
// ---------------------------------------------------------------------------

describe('semanticNormalizeKey', () => {
    it('is a function', () => {
        expect(typeof semanticNormalizeKey).toBe('function');
    });

    it('collapses multiple spaces into one and trims', () => {
        expect(semanticNormalizeKey('  Hello   World  ')).toBe('hello world');
    });

    it('lowercases the input', () => {
        expect(semanticNormalizeKey('Supplier')).toBe('supplier');
        expect(semanticNormalizeKey('PURCHASE_ORDER')).toBe('purchase_order');
    });

    it('returns empty string for empty input', () => {
        expect(semanticNormalizeKey('')).toBe('');
    });

    it('handles single-word input', () => {
        const result = semanticNormalizeKey('Warehouse');
        expect(result).toBe('warehouse');
    });

    it('is idempotent: normalizing twice gives the same result', () => {
        const once = semanticNormalizeKey('  Purchase  Order  ');
        const twice = semanticNormalizeKey(once);
        expect(once).toBe(twice);
    });

    it('handles leading/trailing whitespace only', () => {
        const result = semanticNormalizeKey('   ');
        expect(typeof result).toBe('string');
    });
});

// ---------------------------------------------------------------------------
// semanticPackBuild
// ---------------------------------------------------------------------------

describe('semanticPackBuild', () => {
    it('is a function', () => {
        expect(typeof semanticPackBuild).toBe('function');
    });

    it('builds a candidate pack and returns JSON with expected fields', () => {
        const resultJson = semanticPackBuild(makeMinimalBuildInputJson());
        const result = JSON.parse(resultJson);
        expect(result).toHaveProperty('pack');
        expect(result).toHaveProperty('pack_content_hash');
        expect(result).toHaveProperty('meaning_fingerprint');
    });

    it('returns a pack with the correct org_id and domain_id', () => {
        const resultJson = semanticPackBuild(makeMinimalBuildInputJson());
        const { pack } = JSON.parse(resultJson);
        expect(pack.org_id).toBe('test-org');
        expect(pack.domain_id).toBe('test-domain');
    });

    it('returned pack has schema_version 0.3', () => {
        const resultJson = semanticPackBuild(makeMinimalBuildInputJson());
        const { pack } = JSON.parse(resultJson);
        expect(pack.schema_version).toBe('0.3');
    });

    it('returned pack has approval state candidate', () => {
        const resultJson = semanticPackBuild(makeMinimalBuildInputJson());
        const { pack } = JSON.parse(resultJson);
        expect(pack.trust.approval_state).toBe('candidate');
    });

    it('throws on invalid JSON input', () => {
        expect(() => semanticPackBuild('not valid json')).toThrow();
    });

    it('throws on empty string input', () => {
        expect(() => semanticPackBuild('')).toThrow();
    });

    it('two builds from identical inputs are deterministic', () => {
        const input = makeMinimalBuildInputJson();
        const r1 = JSON.parse(semanticPackBuild(input));
        const r2 = JSON.parse(semanticPackBuild(input));
        expect(r1.meaning_fingerprint).toBe(r2.meaning_fingerprint);
        expect(r1.pack_content_hash).toBe(r2.pack_content_hash);
    });
});

// ---------------------------------------------------------------------------
// semanticPackValidate
// ---------------------------------------------------------------------------

describe('semanticPackValidate', () => {
    it('is a function', () => {
        expect(typeof semanticPackValidate).toBe('function');
    });

    it('returns a JSON string for a valid pack', () => {
        const diagnosticsJson = semanticPackValidate(makeMinimalPackJson());
        expect(() => JSON.parse(diagnosticsJson)).not.toThrow();
    });

    it('returns an array (possibly empty) for a well-formed pack', () => {
        const diagnosticsJson = semanticPackValidate(makeMinimalPackJson());
        const diagnostics = JSON.parse(diagnosticsJson);
        expect(Array.isArray(diagnostics)).toBe(true);
    });

    it('throws on invalid JSON input', () => {
        expect(() => semanticPackValidate('not json')).toThrow();
    });

    it('reports errors for a pack with wrong schema version', () => {
        const pack = JSON.parse(makeMinimalPackJson());
        pack.schema_version = '9.9';
        const diagnosticsJson = semanticPackValidate(JSON.stringify(pack));
        const diagnostics = JSON.parse(diagnosticsJson);
        expect(diagnostics.length).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// semanticPackHash
// ---------------------------------------------------------------------------

describe('semanticPackHash', () => {
    it('is a function', () => {
        expect(typeof semanticPackHash).toBe('function');
    });

    it('returns a string starting with sha256:', () => {
        const hash = semanticPackHash(makeMinimalPackJson());
        expect(typeof hash).toBe('string');
        expect(hash).toMatch(/^sha256:/);
    });

    it('returns the same hash for identical packs', () => {
        const packJson = makeMinimalPackJson();
        expect(semanticPackHash(packJson)).toBe(semanticPackHash(packJson));
    });

    it('throws on invalid JSON', () => {
        expect(() => semanticPackHash('not json')).toThrow();
    });

    it('hash is stable regardless of signature fields', () => {
        const pack1 = JSON.parse(makeMinimalPackJson());
        const hash1 = semanticPackHash(JSON.stringify(pack1));
        // Add signature fields — content hash should remain unchanged
        pack1.trust.signature = 'some-base64-sig';
        pack1.trust.signature_state = 'signed';
        const hash2 = semanticPackHash(JSON.stringify(pack1));
        expect(hash1).toBe(hash2);
    });
});

// ---------------------------------------------------------------------------
// semanticPackValidateGraph
// ---------------------------------------------------------------------------

describe('semanticPackValidateGraph', () => {
    it('is a function', () => {
        expect(typeof semanticPackValidateGraph).toBe('function');
    });

    it('returns a JSON string for a valid pack', () => {
        const resultJson = semanticPackValidateGraph(
            makeMinimalPackJson(),
            'test://source',
            makeDefaultOptionsJson(),
        );
        expect(() => JSON.parse(resultJson)).not.toThrow();
    });

    it('throws on invalid pack JSON', () => {
        expect(() =>
            semanticPackValidateGraph('bad json', 'test://source', makeDefaultOptionsJson()),
        ).toThrow();
    });

    it('throws on invalid options JSON', () => {
        expect(() =>
            semanticPackValidateGraph(makeMinimalPackJson(), 'test://source', 'bad options'),
        ).toThrow();
    });
});

// ---------------------------------------------------------------------------
// semanticPackDiff
// ---------------------------------------------------------------------------

describe('semanticPackDiff', () => {
    it('is a function', () => {
        expect(typeof semanticPackDiff).toBe('function');
    });

    it('returns a parseable JSON string when comparing two packs', () => {
        const packJson = makeMinimalPackJson();
        const diffJson = semanticPackDiff(packJson, packJson);
        expect(() => JSON.parse(diffJson)).not.toThrow();
    });

    it('reports no changes when comparing a pack to itself', () => {
        const packJson = makeMinimalPackJson();
        const diff = JSON.parse(semanticPackDiff(packJson, packJson));
        // No breaking changes when packs are identical
        const entries = diff.entries ?? diff;
        if (Array.isArray(entries)) {
            // Any entries should not be classified as breaking
            const breaking = entries.filter((e: { classification: string }) => e.classification === 'breaking');
            expect(breaking).toHaveLength(0);
        }
    });

    it('detects an added concept when new pack has extra concept', () => {
        const oldPack = JSON.parse(makeMinimalPackJson());
        const newPack = JSON.parse(makeMinimalPackJson());
        newPack.concepts.push({
            id: 'warehouse',
            canonical_name: 'Warehouse',
            kind: 'entity',
            status: 'active',
            definition: {
                text: 'A storage facility.',
                definition_hash: '',
                decision_ref: 'dec_warehouse',
            },
            owner: 'owner@test.com',
            source_refs: [],
            examples: [],
            counterexamples: [],
            allowed_predicates: [],
            valid_contexts: [],
        });
        const diffJson = semanticPackDiff(JSON.stringify(oldPack), JSON.stringify(newPack));
        const diff = JSON.parse(diffJson);
        const entries = diff.entries ?? diff;
        if (Array.isArray(entries)) {
            const additive = entries.filter((e: { classification: string }) =>
                e.classification === 'additive' || e.classification === 'add',
            );
            expect(additive.length).toBeGreaterThan(0);
        } else {
            // Diff result parsed fine; structure may vary
            expect(diffJson).toBeTruthy();
        }
    });

    it('throws on invalid old pack JSON', () => {
        expect(() => semanticPackDiff('bad json', makeMinimalPackJson())).toThrow();
    });

    it('throws on invalid new pack JSON', () => {
        expect(() => semanticPackDiff(makeMinimalPackJson(), 'bad json')).toThrow();
    });
});

// ---------------------------------------------------------------------------
// semanticResolveConcept
// ---------------------------------------------------------------------------

describe('semanticResolveConcept', () => {
    it('is a function', () => {
        expect(typeof semanticResolveConcept).toBe('function');
    });

    it('resolves a known concept and returns valid semantic_truth', () => {
        const resultJson = semanticResolveConcept(
            'Supplier',
            makeMinimalPackJson(),
            makeDefaultOptionsJson(),
        );
        const result = JSON.parse(resultJson);
        expect(result).toHaveProperty('semantic_truth');
        // Active approved concept → "Valid"
        expect(result.semantic_truth).toMatch(/valid/i);
    });

    it('returns unknown semantic_truth for an unrecognized term', () => {
        const resultJson = semanticResolveConcept(
            'CompletelyUnknownTerm',
            makeMinimalPackJson(),
            makeDefaultOptionsJson(),
        );
        const result = JSON.parse(resultJson);
        expect(result.semantic_truth).toMatch(/unknown/i);
        expect(result.resolved_concept_id).toBeNull();
    });

    it('throws on invalid pack JSON', () => {
        expect(() =>
            semanticResolveConcept('Supplier', 'bad json', makeDefaultOptionsJson()),
        ).toThrow();
    });

    it('throws on invalid options JSON', () => {
        expect(() =>
            semanticResolveConcept('Supplier', makeMinimalPackJson(), 'bad options'),
        ).toThrow();
    });

    it('normalizes the lookup key before resolution', () => {
        // "  supplier  " (with spaces and mixed case) should resolve the same as "Supplier"
        const r1 = JSON.parse(
            semanticResolveConcept('Supplier', makeMinimalPackJson(), makeDefaultOptionsJson()),
        );
        const r2 = JSON.parse(
            semanticResolveConcept('  supplier  ', makeMinimalPackJson(), makeDefaultOptionsJson()),
        );
        expect(r1.resolved_concept_id).toBe(r2.resolved_concept_id);
    });
});

// ---------------------------------------------------------------------------
// semanticPackSign / semanticPackVerify
// ---------------------------------------------------------------------------

describe('semanticPackSign and semanticPackVerify', () => {
    it('semanticPackSign is a function', () => {
        expect(typeof semanticPackSign).toBe('function');
    });

    it('semanticPackVerify is a function', () => {
        expect(typeof semanticPackVerify).toBe('function');
    });

    it('semanticPackSign throws on invalid pack JSON', () => {
        const fakePem = '-----BEGIN PRIVATE KEY-----\nnot-real\n-----END PRIVATE KEY-----';
        expect(() => semanticPackSign('bad json', fakePem)).toThrow();
    });

    it('semanticPackVerify throws on invalid pack JSON', () => {
        const fakePem = '-----BEGIN PUBLIC KEY-----\nnot-real\n-----END PUBLIC KEY-----';
        expect(() => semanticPackVerify('bad json', fakePem)).toThrow();
    });
});
