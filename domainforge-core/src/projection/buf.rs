//! Buf.build CLI Integration
//!
//! This module provides optional integration with the buf.build CLI for
//! linting, breaking change detection, and code generation.
//!
//! All buf operations gracefully degrade if the buf CLI is not installed.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Result type for buf operations.
pub type BufResult<T> = Result<T, BufError>;

/// Errors from buf CLI operations.
#[derive(Debug, Clone)]
pub enum BufError {
    /// Buf CLI not found in PATH
    NotInstalled,
    /// Buf command failed with exit code
    CommandFailed { exit_code: i32, stderr: String },
    /// IO error
    IoError(String),
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for BufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufError::NotInstalled => write!(
                f,
                "buf CLI not found. Install from: https://buf.build/docs/installation"
            ),
            BufError::CommandFailed { exit_code, stderr } => {
                write!(f, "buf command failed (exit {}): {}", exit_code, stderr)
            }
            BufError::IoError(msg) => write!(f, "IO error: {}", msg),
            BufError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for BufError {}

/// Buf.build CLI integration.
///
/// Provides optional integration with the buf CLI for enhanced proto workflows.
/// All operations gracefully degrade if buf is not installed.
pub struct BufIntegration {
    /// Path to buf binary (or None to search PATH)
    buf_path: Option<PathBuf>,
}

impl Default for BufIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl BufIntegration {
    /// Create a new BufIntegration with default settings.
    pub fn new() -> Self {
        Self { buf_path: None }
    }

    /// Create with a specific buf binary path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            buf_path: Some(path.into()),
        }
    }

    /// Check if buf CLI is available.
    pub fn is_available(&self) -> bool {
        self.get_version().is_ok()
    }

    /// Get buf CLI version.
    pub fn get_version(&self) -> BufResult<String> {
        let output = self
            .run_buf(&["--version"])
            .map_err(|_| BufError::NotInstalled)?;

        Ok(output.trim().to_string())
    }

    /// Lint proto files in a directory.
    ///
    /// Returns Ok with lint output, or gracefully degrades if buf not installed.
    pub fn lint(&self, proto_dir: &Path) -> BufResult<BufLintResult> {
        // Check if buf.yaml exists, create temporary one if not
        let buf_yaml = proto_dir.join("buf.yaml");
        let temp_config = if !buf_yaml.exists() {
            let config = Self::generate_buf_yaml();
            std::fs::write(&buf_yaml, &config).map_err(|e| BufError::IoError(e.to_string()))?;
            Some(buf_yaml.clone())
        } else {
            None
        };

        let result = self.run_buf(&["lint", proto_dir.to_str().unwrap_or(".")]);

        // Clean up temporary config
        if let Some(path) = temp_config {
            let _ = std::fs::remove_file(path);
        }

        match result {
            Ok(output) => Ok(BufLintResult {
                success: true,
                issues: vec![],
                raw_output: output,
            }),
            Err(BufError::CommandFailed { stderr, .. }) => Ok(BufLintResult {
                success: false,
                issues: Self::parse_lint_issues(&stderr),
                raw_output: stderr,
            }),
            Err(e) => Err(e),
        }
    }

    /// Check for breaking changes between two proto directories.
    pub fn breaking_check(&self, new_dir: &Path, old_dir: &Path) -> BufResult<BufBreakingResult> {
        let result = self.run_buf(&[
            "breaking",
            new_dir.to_str().unwrap_or("."),
            "--against",
            old_dir.to_str().unwrap_or("."),
        ]);

        match result {
            Ok(output) => Ok(BufBreakingResult {
                is_compatible: true,
                violations: vec![],
                raw_output: output,
            }),
            Err(BufError::CommandFailed { stderr, .. }) => Ok(BufBreakingResult {
                is_compatible: false,
                violations: Self::parse_breaking_violations(&stderr),
                raw_output: stderr,
            }),
            Err(e) => Err(e),
        }
    }

    /// Generate a buf.yaml configuration file content.
    pub fn generate_buf_yaml() -> String {
        r#"version: v1
breaking:
  use:
    - FILE
lint:
  use:
    - DEFAULT
  except:
    - PACKAGE_VERSION_SUFFIX
    - PACKAGE_DIRECTORY_MATCH
"#
        .to_string()
    }

    /// Generate a buf.gen.yaml for code generation.
    pub fn generate_buf_gen_yaml(languages: &[BufLanguage]) -> String {
        let mut plugins = String::new();

        for lang in languages {
            plugins.push_str(&lang.to_plugin_config());
        }

        format!(
            r#"version: v1
plugins:
{}"#,
            plugins
        )
    }

    fn run_buf(&self, args: &[&str]) -> BufResult<String> {
        let buf_cmd = self
            .buf_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "buf".to_string());

        let output = Command::new(&buf_cmd).args(args).output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BufError::NotInstalled
            } else {
                BufError::IoError(e.to_string())
            }
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(BufError::CommandFailed {
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    fn parse_lint_issues(output: &str) -> Vec<BufLintIssue> {
        output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| BufLintIssue {
                message: line.to_string(),
                file: None,
                line: None,
                rule: None,
            })
            .collect()
    }

    fn parse_breaking_violations(output: &str) -> Vec<String> {
        output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
}

/// Result of buf lint operation.
#[derive(Debug, Clone)]
pub struct BufLintResult {
    /// Whether lint passed with no issues
    pub success: bool,
    /// List of lint issues found
    pub issues: Vec<BufLintIssue>,
    /// Raw output from buf
    pub raw_output: String,
}

impl BufLintResult {
    /// Create a skipped result (when buf not available).
    pub fn skipped(reason: &str) -> Self {
        Self {
            success: true,
            issues: vec![],
            raw_output: format!("Skipped: {}", reason),
        }
    }
}

/// A single lint issue.
#[derive(Debug, Clone)]
pub struct BufLintIssue {
    /// Issue message
    pub message: String,
    /// File path (if available)
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<u32>,
    /// Lint rule ID (if available)
    pub rule: Option<String>,
}

/// Result of buf breaking check.
#[derive(Debug, Clone)]
pub struct BufBreakingResult {
    /// Whether the changes are compatible
    pub is_compatible: bool,
    /// List of breaking violations
    pub violations: Vec<String>,
    /// Raw output from buf
    pub raw_output: String,
}

/// Supported languages for buf code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufLanguage {
    Go,
    Rust,
    Python,
    TypeScript,
    Java,
    CSharp,
    Cpp,
    Swift,
}

impl BufLanguage {
    /// Get plugin configuration for buf.gen.yaml.
    pub fn to_plugin_config(&self) -> String {
        match self {
            BufLanguage::Go => r#"  - plugin: buf.build/protocolbuffers/go
    out: gen/go
"#
            .to_string(),
            BufLanguage::Rust => r#"  - plugin: buf.build/community/neoeinstein-prost
    out: gen/rust
"#
            .to_string(),
            BufLanguage::Python => r#"  - plugin: buf.build/protocolbuffers/python
    out: gen/python
"#
            .to_string(),
            BufLanguage::TypeScript => r#"  - plugin: buf.build/community/stephenh-ts-proto
    out: gen/typescript
"#
            .to_string(),
            BufLanguage::Java => r#"  - plugin: buf.build/protocolbuffers/java
    out: gen/java
"#
            .to_string(),
            BufLanguage::CSharp => r#"  - plugin: buf.build/protocolbuffers/csharp
    out: gen/csharp
"#
            .to_string(),
            BufLanguage::Cpp => r#"  - plugin: buf.build/protocolbuffers/cpp
    out: gen/cpp
"#
            .to_string(),
            BufLanguage::Swift => r#"  - plugin: buf.build/apple/swift
    out: gen/swift
"#
            .to_string(),
        }
    }

    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "go" | "golang" => Some(BufLanguage::Go),
            "rust" | "rs" => Some(BufLanguage::Rust),
            "python" | "py" => Some(BufLanguage::Python),
            "typescript" | "ts" => Some(BufLanguage::TypeScript),
            "java" => Some(BufLanguage::Java),
            "csharp" | "cs" | "c#" => Some(BufLanguage::CSharp),
            "cpp" | "c++" => Some(BufLanguage::Cpp),
            "swift" => Some(BufLanguage::Swift),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_buf_yaml() {
        let yaml = BufIntegration::generate_buf_yaml();
        assert!(yaml.contains("version: v1"));
        assert!(yaml.contains("breaking:"));
        assert!(yaml.contains("lint:"));
        assert!(yaml.contains("DEFAULT"));
    }

    #[test]
    fn test_generate_buf_gen_yaml() {
        let yaml = BufIntegration::generate_buf_gen_yaml(&[BufLanguage::Go, BufLanguage::Rust]);
        assert!(yaml.contains("version: v1"));
        assert!(yaml.contains("plugins:"));
        assert!(yaml.contains("protocolbuffers/go"));
        assert!(yaml.contains("neoeinstein-prost"));
    }

    #[test]
    fn test_buf_language_from_str() {
        assert_eq!(BufLanguage::parse("go"), Some(BufLanguage::Go));
        assert_eq!(BufLanguage::parse("rust"), Some(BufLanguage::Rust));
        assert_eq!(BufLanguage::parse("py"), Some(BufLanguage::Python));
        assert_eq!(BufLanguage::parse("ts"), Some(BufLanguage::TypeScript));
        assert_eq!(BufLanguage::parse("invalid"), None);
    }

    #[test]
    fn test_buf_lint_result_skipped() {
        let result = BufLintResult::skipped("buf not installed");
        assert!(result.success);
        assert!(result.issues.is_empty());
        assert!(result.raw_output.contains("Skipped"));
    }

    #[test]
    fn test_buf_error_display() {
        let err = BufError::NotInstalled;
        assert!(err.to_string().contains("buf CLI not found"));

        let err2 = BufError::CommandFailed {
            exit_code: 1,
            stderr: "some error".to_string(),
        };
        assert!(err2.to_string().contains("exit 1"));
    }

    #[test]
    fn test_run_buf_not_found() {
        // Test that it handles missing binary
        let buf = BufIntegration::with_path("non_existent_binary");
        assert!(!buf.is_available());

        // Should return NotInstalled error
        let result = buf.get_version();
        match result {
            Err(BufError::NotInstalled) => (),
            _ => panic!("Expected NotInstalled error, got {:?}", result),
        }
    }
}
