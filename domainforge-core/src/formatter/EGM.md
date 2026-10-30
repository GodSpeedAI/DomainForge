# formatter — SEA Source Code Formatter

## Overview
Parses SEA source into an `Ast` and pretty-prints it back with canonical
indentation and spacing, optionally re-inserting the comments that were extracted
from the original text.

## Interface
- `[printer.rs:format]` — `format(source, config) -> Result<String, FormatError>`; dispatches on `config.preserve_comments`.
- `[printer.rs:format_preserving_comments]` — extract comments, parse, format, then re-insert comments at their associated positions.
- `[printer.rs:FormatError]` — `ParseError(String)` / `InternalError(String)`.
- `[config.rs:FormatConfig]` (`+new/with_indent_style/with_indent_width/with_tabs/indent_string`) — `indent_style`, `indent_width`, `max_line_width`, `trailing_newline`, `preserve_comments`, `sort_imports`.
- `[config.rs:IndentStyle]` — `Spaces` (default) / `Tabs`.
- `[comments.rs:Comment]` — `text`, `line` (1-indexed), `is_trailing`.
- `[comments.rs:extract_comments]` — `BTreeMap<usize, Vec<Comment>>` keyed by line number.
- `[comments.rs:associate_comments_with_lines]` — attaches leading comments to declaration lines.
- `[comments.rs:CommentedSource]` (`+new/leading_comments_for/has_comments`) — source plus comment maps and file-header comments.
- `[mod.rs]` — re-exports `CommentedSource`, `FormatConfig`, `IndentStyle`, `format`, `format_preserving_comments`, `FormatError`.

## Invariants
- `[config.rs:FormatConfig::default]` — defaults are `Spaces`, width 4, max line width 100, `trailing_newline`, `preserve_comments`, and `sort_imports` all true.
- `[printer.rs:FormatError]` — unparseable input returns `FormatError::ParseError`; the formatter does not emit partially formatted text on parse failure.
- `[comments.rs:extract_comments]` — a `//` inside a `"…"` string literal or a `"""…"""` multiline string is not a comment; the scanner tracks string state instead of treating any `//` as a comment start.

## Side Effects
- None observed in source. The formatter is a pure `&str`/`FormatConfig` → `Result<String, FormatError>` transform; writing files is the CLI's job (`src/cli/format.rs`).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
