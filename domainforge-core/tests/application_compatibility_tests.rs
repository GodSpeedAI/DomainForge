//! Pre-change compatibility oracles for SEA application-contract grammar work.
//!
//! AST and formatter hashes for legacy fixtures are captured from the compiler
//! *before* grammar changes and must never be regenerated after that work.

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
            "4947477664dd224bae50e67ff927a4450bc3bce301ea636a212d558922107ed2"
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
            "507c2767371aec4c751c769588d717fd57eda473c0ae07c814c5a94577b70f9a"
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
