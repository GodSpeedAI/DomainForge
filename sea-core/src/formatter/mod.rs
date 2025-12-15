//! SEA Code Formatter
//!
//! This module provides functionality to format SEA-DSL source code with
//! consistent whitespace, indentation, and styling.
//!
//! # Example
//!
//! ```rust,ignore
//! use sea_core::formatter::{format, FormatConfig};
//!
//! let source = r#"Entity   "Foo"  in    bar"#;
//! let formatted = format(source, FormatConfig::default()).unwrap();
//! assert_eq!(formatted, "Entity \"Foo\" in bar\n");
//! ```

pub mod comments;
pub mod config;
pub mod printer;

pub use comments::CommentedSource;
pub use config::{FormatConfig, IndentStyle};
pub use printer::{format, format_preserving_comments, FormatError};
