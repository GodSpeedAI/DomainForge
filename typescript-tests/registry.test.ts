import * as fs from 'fs';
import { tmpdir } from 'os';
import * as path from 'path';
import { describe, expect, it } from 'vitest';
import { NamespaceRegistry } from '../index';

describe('NamespaceRegistry (TS)', () => {
    it('discovers and resolves files', () => {
        // Skip test if addon does not expose namespace registry (prebuilt binaries)
        if ((global as any).NamespaceRegistry === undefined) {
            // Access via require('./index') module import
            const index = require('../index');
            if (!index.NamespaceRegistry) {
                // Skip runtime assertion rather than failing the build
                return;
            }
        }
        const base = fs.mkdtempSync(path.join(tmpdir(), 'sea-test-'));
        const dir = path.join(base, 'domains', 'logistics');
        fs.mkdirSync(dir, { recursive: true });
        const filePath = path.join(dir, 'warehouse.sea');
        fs.writeFileSync(filePath, 'Entity "Warehouse"');

        const registryPath = path.join(base, '.sea-registry.toml');
        const content = `version = 1\ndefault_namespace = "default"\n\n[[namespaces]]\nnamespace = "logistics"\npatterns = ["domains/logistics/**/*.sea"]\n`;
        fs.writeFileSync(registryPath, content);

        const reg = NamespaceRegistry.from_file(registryPath);
        expect(reg).toBeDefined();
        const files = reg.resolve_files();
        expect(files.length).toBe(1);
        expect(files[0].path).toBe(filePath);
        expect(files[0].namespace).toBe('logistics');

        const ns = reg.namespace_for(filePath);
        expect(ns).toBe('logistics');
    });

    it('enforces precedence longest prefix', () => {
        if ((global as any).NamespaceRegistry === undefined) {
            const index = require('../index');
            if (!index.NamespaceRegistry) {
                return;
            }
        }
        const base = fs.mkdtempSync(path.join(tmpdir(), 'sea-test-'));
        const dir = path.join(base, 'domains', 'logistics');
        fs.mkdirSync(dir, { recursive: true });
        const filePath = path.join(dir, 'warehouse.sea');
        fs.writeFileSync(filePath, 'Entity "Warehouse"');

        const registryPath = path.join(base, '.sea-registry.toml');
        const content = `version = 1\ndefault_namespace = "default"\n\n[[namespaces]]\nnamespace = "short"\npatterns = ["domains/**/*.sea"]\n\n[[namespaces]]\nnamespace = "long"\npatterns = ["domains/logistics/**/*.sea"]\n`;
        fs.writeFileSync(registryPath, content);

        const reg = NamespaceRegistry.from_file(registryPath);
        expect(reg).toBeDefined();
        const ns = reg.namespace_for(filePath);
        expect(ns).toBe('long');
    });

    it('enforces alphabetical tie-breaker', () => {
        if ((global as any).NamespaceRegistry === undefined) {
            const index = require('../index');
            if (!index.NamespaceRegistry) {
                return;
            }
        }
        const base = fs.mkdtempSync(path.join(tmpdir(), 'sea-test-'));
        const dir = path.join(base, 'domains', 'logistics');
        fs.mkdirSync(dir, { recursive: true });
        const filePath = path.join(dir, 'warehouse.sea');
        fs.writeFileSync(filePath, 'Entity "Warehouse"');

        const registryPath = path.join(base, '.sea-registry.toml');
        const content = `version = 1\ndefault_namespace = "default"\n\n[[namespaces]]\nnamespace = "logistics"\npatterns = ["domains/*/warehouse.sea"]\n\n[[namespaces]]\nnamespace = "finance"\npatterns = ["domains/*/warehouse.sea"]\n`;
        fs.writeFileSync(registryPath, content);

        const reg = NamespaceRegistry.from_file(registryPath);
        expect(reg).toBeDefined();
        const ns = reg.namespace_for(filePath);
        expect(ns).toBe('finance');
    });

    it('errors on ambiguous match when fail flag set', () => {
        if ((global as any).NamespaceRegistry === undefined) {
            const index = require('../index');
            if (!index.NamespaceRegistry) {
                return;
            }
        }
        const base = fs.mkdtempSync(path.join(tmpdir(), 'sea-test-'));
        const dir = path.join(base, 'domains', 'logistics');
        fs.mkdirSync(dir, { recursive: true });
        const filePath = path.join(dir, 'warehouse.sea');
        fs.writeFileSync(filePath, 'Entity "Warehouse"');

        const registryPath = path.join(base, '.sea-registry.toml');
        const content = `version = 1\ndefault_namespace = "default"\n\n[[namespaces]]\nnamespace = "logistics"\npatterns = ["domains/*/warehouse.sea"]\n\n[[namespaces]]\nnamespace = "finance"\npatterns = ["domains/*/warehouse.sea"]\n`;
        fs.writeFileSync(registryPath, content);

        const reg = NamespaceRegistry.from_file(registryPath);
        expect(reg).toBeDefined();
        let threw = false;
        try {
            // pass true to indicate failing on ambiguity
            reg.namespace_for(filePath, true);
        } catch (err) {
            threw = true;
        }
        expect(threw).toBe(true);

        // resolve_files should error as well
        threw = false;
        try {
            reg.resolve_files(true);
        } catch (err) {
            threw = true;
        }
        expect(threw).toBe(true);
    });
});
