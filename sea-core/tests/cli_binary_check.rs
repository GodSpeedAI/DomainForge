#[cfg(feature = "cli")]
use std::path::Path;

#[cfg(feature = "cli")]
fn get_sea_binary() -> String {
    std::env::var("CARGO_BIN_EXE_sea").unwrap_or_else(|_| {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../target");
        if cfg!(debug_assertions) {
            path.push("debug");
        } else {
            path.push("release");
        }
        let binary_name = if cfg!(windows) { "sea.exe" } else { "sea" };
        path.push(binary_name);
        path.to_string_lossy().to_string()
    })
}

#[cfg(feature = "cli")]
#[test]
fn test_cli_binary_present_when_cli_feature_enabled() {
    // This test should be run with `--features cli` to ensure the CLI is built
    let bin = get_sea_binary();
    assert!(Path::new(&bin).exists(), "CLI binary not found at {}", bin);
}

#[cfg(not(feature = "cli"))]
#[test]
fn test_cli_binary_check_skipped_without_feature() {
    // When the CLI feature is disabled, we skip the binary presence check.
    assert!(true);
}
