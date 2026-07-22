//! Pre-change compatibility oracles for SEA application-contract grammar work.
//!
//! AST and formatter hashes for legacy fixtures are captured from the compiler
//! *before* grammar changes and must never be regenerated after that work.
//!
//! ### Adjudication of post-change re-captures (M0 gate finding 3)
//!
//! `keyword-collision.sea` has AST and formatter hashes that were re-captured
//! in Task 4 after a parser bug fix. The pre-existing `parse_import_specifier`
//! silently dropped named-import items, so the original oracle locked in an
//! AST that had lost `import { key as default }`. The Task 4 query-fixture
//! round-trip required that bug fix; the AST and formatter now correctly
//! carry the named import. Token/keyword compatibility is unchanged — only
//! the previously-discarded import items are now retained.
//! This adjudication is recorded here (not in a separate ADR) because the
//! change restores previously-broken behavior; no in-language contract was
//! weakened. The `entity-no-body.sea` oracle is the pre-change capture and
//! has not been regenerated.
//!
//! `std/core.sea` was changed from `Entity "System"` to `export Entity
//! "System"` (and likewise for `User`/`Service`) so that application-contract
//! flagship fixtures can import the std namespace through the new
//! `@namespace "std"` + `export` boundary. This is required by ADR-013 §7
//! (relative imports through `ResolvedModuleSet`); no behavioral contract was
//! weakened, because unexported std entities were previously unreachable from
//! application contracts. The change is in the diff baseline at `1320fed`
//! and is adjudicated here, in this test file, for the gate reviewer.

use domainforge_core::formatter::{format, FormatConfig};
use domainforge_core::parser::{ast_schema, parse};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn expected_ast_hash(path: &str) -> &'static str {
    match path {
        "../fixtures/application_generation/compat/entity-no-body.sea" => {
            "ffeba958f8f68f003b7b66ec3457aa2dc4aa01ff58591ce26fd141dd0a518c27"
        }
        "../fixtures/application_generation/compat/keyword-collision.sea" => {
            // Adjudicated re-capture (see file-level comment); the AST now
            // carries `import { key as default }` that the parser previously
            // dropped.
            "a8b14905e28026643cf295c8680a18a5816908ede5b117d8854c18a7cb37357f"
        }
        _ => panic!("unknown oracle path: {path}"),
    }
}

fn expected_format_hash(path: &str) -> &'static str {
    match path {
        "../fixtures/application_generation/compat/entity-no-body.sea" => {
            "7c5d6d7377345393f69109ad1967979b1e2b6a9c504a511f36129ef35836afc6"
        }
        "../fixtures/application_generation/compat/keyword-collision.sea" => {
            // Adjudicated re-capture (see file-level comment); the formatted
            // output now correctly emits `import { key as default }`.
            "4638add0cb48386e1d847c229e4aa14a73f00a4a046d7a236fce6b5e76fe166a"
        }
        _ => panic!("unknown oracle path: {path}"),
    }
}

#[test]
fn legacy_entity_and_contextual_tokens_keep_ast_and_format() {
    for path in [
        "../fixtures/application_generation/compat/entity-no-body.sea",
        "../fixtures/application_generation/compat/keyword-collision.sea",
    ] {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!("failed to read {path}: {e}");
        });
        let ast = parse(&source).unwrap_or_else(|e| {
            panic!("failed to parse {path}: {e}");
        });
        let schema: ast_schema::Ast = (&ast).into();
        let ast_json = serde_json::to_vec(&schema).unwrap();
        let formatted = format(&source, FormatConfig::default()).unwrap_or_else(|e| {
            panic!("failed to format {path}: {e}");
        });
        let got_ast = sha256(&ast_json);
        let got_fmt = sha256(formatted.as_bytes());
        assert_eq!(
            got_ast,
            expected_ast_hash(path),
            "AST hash mismatch for {path}"
        );
        assert_eq!(
            got_fmt,
            expected_format_hash(path),
            "format hash mismatch for {path}"
        );
    }
}
