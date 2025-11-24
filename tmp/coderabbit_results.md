Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: sea-core/src/cli/validate.rs
Line: 110 to 116
Type: potential_issue

[x] Task:
In sea-core/src/cli/validate.rs around lines 110 to 116, the parameters \_source and \_show_source are declared but unused so the --no-source flag has no effect; either implement source display or remove the flag and parameters. To fix: if keeping the flag, use \_show_source to conditionally include the raw source text in the serialized output (e.g., add a "source" field to the JSON only when \_show_source is true) and ensure \_source is supplied where called; otherwise remove \_source and \_show_source from the function signature, delete the no_source field from ValidateArgs and any CLI wiring for that flag, and remove related dead code. Ensure update of tests and call sites accordingly.

============================================================================
File: sea-core/src/cli/validate_kg.rs
Line: 15 to 16
Type: potential_issue

[x] Task:
In sea-core/src/cli/validate_kg.rs around lines 15 to 16, the current content-based heuristic (checking source.trim_start().starts_with(' RDF/XML, otherwise treat as Turtle), and for files without extensions implement a safer fallback: either attempt to parse as RDF/XML (allowing leading processing instructions/comments) and on parse failure fall back to Turtle, or try parsing with both parsers and pick the one that succeeds; include clear logging when ambiguous or when parsing falls back.

============================================================================
File: sea-core/src/cli/validate.rs
Line: 21 to 25
Type: potential_issue

[x] Task:
In sea-core/src/cli/validate.rs around lines 21 to 25, the ValidateArgs struct exposes strict and round_trip CLI options that are unused; either remove these fields from the public CLI definition or implement their behavior in the validation flow: for removal, delete the fields and any parsing/flag wiring so they are no longer surfaced to users; for implementation, wire strict into validation logic to toggle failure levels (e.g., treat warnings as errors) and wire round_trip to perform a parse-serialize-parse comparison after validation (failing if the round-trip differs), and update help text and tests accordingly.

============================================================================
File: sea-core/src/cli/format.rs
Line: 20
Type: potential_issue

[x] Task:
In sea-core/src/cli/format.rs around line 20, the parse error currently bails with a message that omits the file path; change the bail to include the file path (use the same path variable/Display used for the I/O errors on line 17) by formatting the error message to include the file path (e.g., path.display() or path.to_string_lossy()) so the resulting anyhow::bail! message contains the file path for easier debugging.

============================================================================
File: docs/reference/cli.md
Line: 84 to 86
Type: potential_issue

[x] Task:
In docs/reference/cli.md around lines 84 to 86, the usage for the sea validate-kg command is missing the [OPTIONS] indicator and the documented options themselves; update the usage line to include [OPTIONS] and add the following options under the command description: --shapes , --format , --json, and --severity so the CLI docs match the plan (lines ~128-133).

============================================================================
File: docs/reference/cli.md
Line: 38 to 46
Type: potential_issue

[x] Task:
In docs/reference/cli.md around lines 38 to 46, the import command reference is missing the optional [OUTPUT] parameter and the additional options from the plan; update the usage line to include the optional [OUTPUT] argument and add documentation entries for --validate, --merge , and --namespace under the import command section, briefly describing each option (validate runs input checks, merge specifies a file to merge, namespace sets the target namespace) so the reference matches the plan.

============================================================================
File: docs/reference/cli.md
Line: 51 to 59
Type: potential_issue

[x] Task:
In docs/reference/cli.md around lines 51 to 59, the documentation for the project command is missing three options referenced earlier in the plan; add entries for --output-dir , --pretty, and --validate to the Formats/format section (or the command's options list) so they are documented consistently. For each option, add a one-line description: --output-dir to specify target directory for generated files, --pretty to produce human-readable/formatted output, and --validate to run validation after generation.

============================================================================
File: docs/reference/cli.md
Line: 76 to 78
Type: potential_issue

[x] Task:
In docs/reference/cli.md around lines 76 to 78, the test command documentation is incomplete — update the usage to "sea test [OPTIONS] [PATTERN]" with PATTERN optional and add the missing options from the plan: document --pattern (glob pattern to select test files), --filter (run only tests matching regex), --verbose (enable verbose output), --fail-fast (stop on first failure), and --json (emit machine-readable JSON output); for each option include its flag, argument form if any, and a one-line description consistent with surrounding docs.

============================================================================
File: docs/reference/cli.md
Line: 27 to 32
Type: potential_issue

[x] Task:
In docs/reference/cli.md around lines 27 to 32, the CLI options list is inconsistent with the implementation plan (docs/plans/cli_commands.plan.md lines 73–79): it omits --json, --exit-code, --fix, --config and includes an extra --no-source, and the --format flag overlap with a planned --json flag. Update this docs file to match the agreed API: either replace --format with the planned flags (add --json, --exit-code, --fix, --config) and remove --no-source, or explicitly document how --format and --json coexist; ensure flag names, defaults, and descriptions mirror the plan file exactly and note any behavioral differences (e.g., precedence between --format and --json).

============================================================================
File: docs/reference/cli.md
Line: 64 to 71
Type: potential_issue

[x] Task:
In docs/reference/cli.md around lines 64 to 71, the "format" command docs are incomplete — update the usage to accept multiple files using ... and add the missing options from the plan: --write, --config , --line-length , and --indent ; ensure each option has a one-line description and the usage line and examples reflect the multi-file usage and these flags so the documentation matches the planned behavior.

============================================================================
File: sea-core/src/cli/import.rs
Line: 2
Type: nitpick

Review completed ✔
