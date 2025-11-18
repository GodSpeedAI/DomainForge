Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: scripts/prepare_rust_debug.sh
Line: 18
Type: potential_issue

[x] Task:
In scripts/prepare_rust_debug.sh around line 18, the script invokes find_and_link_test_binary.sh but later hardcodes ${ROOT_DIR}/target/debug/deps/sea_debug_test (line 21) which may be wrong; change the call to capture the linking script's stdout (the actual symlink path) into a variable, trim/validate that path exists and is a file/symlink, and only then pass that validated path to the Python helper; if the linking script fails or prints nothing, emit an error and exit with non‑zero status so the helper is not invoked with an incorrect path.



============================================================================
File: README_PYTHON.md
Line: 223 to 237
Type: refactor_suggestion

[x] Task:
In README_PYTHON.md around lines 223 to 237, add direct shell equivalents and notes alongside the existing just targets: keep the just python-setup/just python-test/just python-clean examples but annotate they require just, then insert a manual workflow showing how to create a venv (python -m venv .venv), activate it (include the Linux/macOS and Windows activation commands), install dev deps (pip install -e . or pip install -r requirements-dev.txt if used), run tests with pytest (pytest tests/), and show cleanup by removing the .venv directory (rm -rf .venv) so users without just have clear, copy-pastable commands.



============================================================================
File: scripts/find_and_link_test_binary.sh
Line: 30
Type: potential_issue

[x] Task:
In scripts/find_and_link_test_binary.sh around lines 30 and 34, TEST_NAME is interpolated directly into a grep -E regex which lets regex metacharacters in user-supplied names alter matching; create an escaped version of TEST_NAME (backslash-escape all regex special chars) before composing the grep pattern and use that escaped variable in place of raw TEST_NAME, ensuring you still append the suffix part like '(-[0-9a-f]+)?(\\.exe)?$'; apply the same escape-and-use change to the other occurrence at line 34.



============================================================================
File: sea-core/docs/specs/projections/sbvr_kg_mapping.md
Line: 31
Type: potential_issue

[x] Task:
In sea-core/docs/specs/projections/sbvr_kg_mapping.md around line 31, replace the ambiguous phrase "units default to units" with an explicit statement of the exact default value and how it is produced/consumed: state whether the default is the literal string "units" or a named constant (give the constant identifier and its module/file), and show the concrete mapping call or function used (e.g., name the function that parses/constructs the unit and the concrete return value for the default, such as unit_from_string("units") => ""). Ensure the doc uses the exact literal or constant identifier and a short example of the resulting internal value so integrators can implement the reverse mapping unambiguously.



============================================================================
File: scripts/find_and_link_test_binary.sh
Line: 21
Type: nitpick

[x] Task:
In scripts/find_and_link_test_binary.sh around line 21, the cargo invocation uses an unquoted variable (cargo test -p ${CRATE_PATH} --no-run); update it to quote the variable (cargo test -p "${CRATE_PATH}" --no-run) to follow shell best practices and avoid word-splitting or globbing issues, leaving the rest of the command unchanged.



============================================================================
File: scripts/find_and_link_test_binary.sh
Line: 5
Type: refactor_suggestion

[x] Task:
In scripts/find_and_link_test_binary.sh around line 5, there is a redundant shell option invocation: the script already sets errexit and nounset with set -o errexit and set -o nounset on lines 2-3, so remove the duplicate set -eu on line 5 (or replace it with nothing) to avoid duplication while preserving the same shell behavior.



============================================================================
File: sea-core/src/bin/sea.rs
Line: 94 to 98
Type: potential_issue

[x] Task:
In sea-core/src/bin/sea.rs around lines 94 to 98, the code uses fragile string matching on the error message ("SHACL validation failed") to decide behavior; change the code to use a typed error variant instead: update sea_core::import_kg_rdfxml to return a Result (or similar) with a ShaclValidation variant, then match the Result here and handle Err(ImportError::ShaclValidation(msg)) by printing the message and exiting, while treating other Err variants as fallbackable parse/format errors; ensure imports and signatures are adjusted accordingly and do not rely on substring matching.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 377 to 415
Type: refactor_suggestion

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md lines 377 to 415: The subagent prompt template and rules are ambiguous about which fields are mandatory versus optional; update the documentation to explicitly label required fields (Goal, Deliverable, Files/Folders context), mark Constraints and Tools as conditional (required only when applicable), and add a short "Template Strictness" block that enumerates "Always: Goal, Context, Deliverable" and "When applicable: Constraints, Tools" plus the "Never" items; also include minimal format guidance (max nested depth, preferred list styles, expected output size) as a single compact note so authors know formatting and scope without expanding the template.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 673 to 704
Type: nitpick




============================================================================
File: sea-core/src/parser/ast.rs
Line: 482
Type: nitpick

[x] Task:
In sea-core/src/parser/ast.rs around line 482, the local variable declaration uses an unnecessary mut ("let mut pairs = SeaParser::parse(Rule::expression, source)"); remove the mut so it reads let pairs = ... because pairs is only consumed once via .next() and never mutated; update any subsequent uses accordingly (e.g., call pairs.next() directly) and run cargo build to ensure no other mutable references rely on it.



============================================================================
File: .github/agents/Coder.agent.md
Line: 259 to 287
Type: potential_issue

[x] Task:
.github/agents/Coder.agent.md lines 259-287: several Markdown headers and numbered list items are merged on the same lines (e.g., line 266 merges the Phase 2 header with item 2, and lines 282–287 have similar compression); fix by inserting proper line breaks so each header is on its own line followed by a blank line (or at least a new line) before the numbered list, and ensure each numbered list item starts on its own line—apply this for Phase 2 and the subsequent sections so headers and list items are separated and each list item occupies its own line while preserving the existing text and numbering.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 729 to 768
Type: nitpick

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md around lines 729 to 768: the initialization section currently delegates a full workspace deep-dive to a subagent but lacks operational safeguards; update the text to add concrete behavior: enforce a 5 minute timeout for the initial subagent run and fall back to partial results if exceeded, require the subagent to resummarize or truncate outputs over 3000 tokens down to a target ≤500 tokens, add caching of the compressed initialization results keyed by repo HEAD hash and reuse them if the hash is unchanged, and specify a refresh cadence (run on workspace open or if cached result is older than 4 hours or repo HEAD changed); incorporate short instructions for the orchestrator to store the compressed result in memory and how to detect/compare the repo HEAD to decide reuse or rerun.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 418 to 469
Type: refactor_suggestion

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md around lines 418 to 469: the Result Integration Protocol currently omits handling for subagent timeouts/failures; add a new section titled "5. Handle Subagent Failures Gracefully" describing (1) detection of failures/timeouts and logging, (2) policy for retries vs immediate escalation to main chat (e.g., one automatic retry with exponential backoff, then escalate if critical), (3) capturing and preserving partial results and including them in the escalation message, (4) clear instructions to recommend fallback actions (manual review, simplified subtask, or different subagent) and to mark decision-critical missing info, and (5) examples/templates for escalation messages and flags indicating whether to retry or escalate.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 631 to 670
Type: refactor_suggestion

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md around lines 631 to 670: the "Vibe Check Integration" section lacks concrete invocation rules (when to run, how to invoke, expected outputs, and how to integrate results); add a short "Vibe Check Invocation Rules" subsection that specifies trigger conditions (e.g., after 3+ subagent delegations or every 30 minutes), invocation method (call the Vibe Check tool with the current session transcript or a summarized transcript if large), expected output format (e.g., clarity score 1–10, 2–3 actionable recommendations, identified risk areas), and an integration rule (e.g., if clarity < 7, apply recommended adjustments immediately), keeping the wording concise and consistent with surrounding style.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 448 to 457
Type: refactor_suggestion

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md around lines 448 to 457: the "Store Compressed Knowledge" section gives an example but lacks concrete guidance; update the doc to specify the memory tool, exact storage tag, payload format, size limit and retention policy — e.g., instruct authors to use the project's Memory Tool (or named implementation) with tag "compressed_decision", store entries as a compact structured payload (fields: decision_title, bullets[2-3], date, rejection_reason(optional)), enforce a max size of ~200 tokens (or X bytes) per entry to preserve token budget, and a retention rule like "retain for current feature cycle, replace on new decision or after N days" (include example retention period). Ensure the doc text replaces the vague example with these specific tool/format/size/retention directives.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 480 to 488
Type: potential_issue

[x] Task:
In sea-core/src/parser/ast.rs around lines 480 to 488, the parser currently takes the first parsed pair and ignores any remaining tokens, allowing trailing garbage like "x + y z" to be silently dropped; modify the function to validate the entire input is consumed by checking that there are no additional pairs after the first one and return a GrammarError (e.g. "Trailing input" or similar with context) if extra tokens exist; implement this by calling pairs.next() after extracting the first pair and returning an Err when it yields Some(_), so only fully-consumed inputs succeed.



============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 472 to 627
Type: nitpick

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md lines 472-627: The domain-specific patterns assume DomainForge/SEA DSL expertise and may be misapplied by teams using other architectures; add a brief preface clarifying scope and instructing readers how to adapt paths and modules to their own projects (replace sea-core/, policy/, graph/* with their equivalents) and point them to the generic patterns for framework-agnostic guidance; keep the note concise, placed at the start of the "Domain-Specific Patterns" section, and ensure it mentions adaptation of core modules, language bindings, validation subsystems, and performance components.

============================================================================
File: sea-core/tests/projection_contracts_tests.rs
Line: 40 to 55
Type: refactor_suggestion

[x] Task:
In sea-core/tests/projection_contracts_tests.rs around lines 40 to 55, duplicate test setup (creating Graph, two Entities, one Resource, extracting their ids, adding them to the graph and adding a Flow) should be extracted into a helper function; add a function at the top of the file fn setup_minimal_graph(quantity: i64) -> (Graph, String, String, String) that constructs the Graph, creates Warehouse and Factory Entities and Cameras Resource with namespace "default" and unit_from_string("units"), clones their ids, adds them to the graph, creates a Flow using Decimal::new(quantity, 0) (ensure using cloned ids where needed), returns (graph, e1_id, e2_id, r_id), and replace the duplicated blocks in the three tests with calls to this helper.



============================================================================
File: sea-core/src/calm/sbvr_import.rs
Line: 4 to 12
Type: nitpick

[x] Task:
In sea-core/src/calm/sbvr_import.rs around lines 4 to 12, the nested match for parsing and converting SBVR XMI can be flattened: replace the match chains with the ? operator combined with map_err to convert errors to Strings — first call SbvrModel::from_xmi(xmi).map_err(|e| format!("Failed to parse SBVR XMI: {}", e))? to get the model, then call model.to_graph().map_err(|e| format!("Failed to convert SBVR to Graph: {}", e))? and return the resulting Graph; this removes nested matches and keeps the same error messages.



============================================================================
File: sea-core/tests/projection_contracts_tests.rs
Line: 93 to 94
Type: potential_issue

[x] Task:
In sea-core/tests/projection_contracts_tests.rs around lines 93 to 94, the test currently asserts a SHACL violation by fragile substring matching of the error message ("must be >"); instead, update the assertion to inspect the violation's structured fields (e.g., constraint type or severity) — locate the violation whose constraint_type (or constraint enum) equals "minExclusive" (or the appropriate Constraint::MinExclusive variant) or whose severity matches the expected value, and assert that such a violation exists; replace the .contains("must be >") check with an assertion on the violation.constraint/constraint_type or violation.severity to make the test robust.



============================================================================
File: sea-core/tests/cli_import_tests.rs
Line: 27 to 31
Type: refactor_suggestion

[x] Task:
In sea-core/tests/cli_import_tests.rs around lines 27-31 (and likewise at 58-62, 91-95, 124-128) the binary resolution logic is duplicated and hardcodes the debug folder and misses Windows .exe handling; add a helper function get_sea_binary() near the top of the file that returns the resolved path using CARGO_BIN_EXE_sea if present, otherwise builds a PathBuf from env!("CARGO_MANIFEST_DIR") -> "../target" -> either "debug" or "release" depending on cfg!(debug_assertions) -> either "sea.exe" on Windows or "sea" otherwise, converting to a String with to_string_lossy().to_string(); then replace each of the four duplicated std::env::var(...).unwrap_or_else(...) blocks with a call to get_sea_binary().



============================================================================
File: sea-core/tests/cli_import_tests.rs
Line: 86
Type: nitpick

[x] Task:
In sea-core/tests/cli_import_tests.rs around line 86, there's a leftover debug println! that prints RDF/XML output; remove this debug print to keep tests clean and avoid noisy test output (delete the println! line or replace it with an assertion or logging at debug level if needed).



============================================================================
File: sea-core/tests/projection_contracts_tests.rs
Line: 32 to 35
Type: potential_issue

[x] Task:
In sea-core/tests/projection_contracts_tests.rs around lines 32 to 35, the round-trip test currently only checks counts; add assertions on graph2 to validate data integrity: assert that entity names contain "Warehouse" and "Factory" and their IDs match the original IDs, assert the resource name is "Cameras" and its unit matches the original unit, assert the flow quantity equals 100 and its source/target IDs match the original flow IDs; perform these checks against graph2 (post-round-trip) using the same accessors used earlier so the test ensures names, units, quantities and IDs were preserved.



============================================================================
File: sea-core/src/sbvr.rs
Line: 485 to 489
Type: potential_issue

[x] Task:
In sea-core/src/sbvr.rs around lines 485 to 489, the code currently hardcodes let quantity = Decimal::from(1) because SbvrFactType does not include a quantity field; add an inline comment above this line stating that quantity defaults to 1 due to the SBVR model limitation (SbvrFactType has no quantity), briefly noting this is intentional and may require extending the model in the future if quantities are needed, so future readers understand why the default is used.



============================================================================
File: sea-core/src/sbvr.rs
Line: 387 to 401
Type: potential_issue

[x] Task:
In sea-core/src/sbvr.rs around lines 379 to 401, the code currently initializes rule.priority = None and then unconditionally sets a default based on rule_type, which overwrites any explicit  parsed from the XMI; update the parsing so you (1) parse  during the child-iteration loop and assign rule.priority = Some(parsed_value) when present, and (2) remove the unconditional overwrite and instead apply the rule-type default only if rule.priority is still None after parsing (set rule.priority = Some(default) using the existing mapping: Obligation/Prohibition=>5, Permission=>1, Derivation=>3).



============================================================================
File: sea-core/src/kg_import.rs
Line: 193 to 199
Type: nitpick

[x] Task:
In sea-core/src/kg_import.rs around lines 193 to 199, the else branch that extends kg.shapes is unreachable because augment_shapes_from_store is only invoked when kg.shapes.is_empty(); remove the dead else branch and simplify the block to assign extracted_shapes to kg.shapes when extracted_shapes is not empty (i.e., replace the if/else with a single assignment branch).



============================================================================
File: sea-core/src/kg_import.rs
Line: 176 to 177
Type: nitpick

[x] Task:
In sea-core/src/kg_import.rs around lines 176 to 177, replace the manual starts_with + slicing pattern with strip_prefix to avoid hard-coded length slicing; specifically, call target.strip_prefix("http://domainforge.ai/sea#") and, if Some(rest), format!("sea:{}", rest) (handle the None case as before). This mirrors earlier normalization fixes and prevents brittle index math.



============================================================================
File: docs/plans/polish-plan.md
Line: 1330
Type: potential_issue

[x] Task:
In docs/plans/polish-plan.md around line 1330, the acceptance criterion "Temporal queries run <50ms on 1M historical flows" relies on redb without a validated fallback; update the plan to require running redb benchmarks against a 1M-flow dataset as a gating pre-requisite before Step 8 implementation, and if benchmarks fail to meet <50ms, change the plan to escalate to implementing the PostgreSQL adapter in Phase 18 (or reduce the acceptance-test scale) and record this fallback path and decision criteria in the risks/mitigation section.



============================================================================
File: sea-core/src/kg_import.rs
Line: 181 to 182
Type: potential_issue

[x] Task:
sea-core/src/kg_import.rs around lines 181 to 182: the current fragment extraction uses .replace("Shape", "") which removes every occurrence of "Shape" instead of only a trailing suffix; change the logic to extract the fragment (let fragment = &shape_uri[pos + 1..]) and then remove a trailing "Shape" only (e.g., use fragment.strip_suffix("Shape").unwrap_or(fragment) or equivalent) before formatting the "sea:{}" string, ensuring you preserve the fragment when "Shape" appears non-terminal.



============================================================================
File: scripts/update_launch_program.py
Line: 36 to 39
Type: potential_issue

[x] Task:
In scripts/update_launch_program.py around lines 36 to 39, the startswith check can fail on Windows because workspace_prefix and program_path may use different slash styles or case; normalize both paths (e.g., call os.path.abspath + os.path.normpath + os.path.normcase on workspace_prefix and program_path) before doing the startswith comparison, then compute rel_path using os.path.relpath on the normalized absolute paths and continue to set program_path to os.path.join("${workspaceFolder}", rel_path).replace("\\", "/").



============================================================================
File: docs/plans/polish-plan.md
Line: 24
Type: potential_issue

[x] Task:
In docs/plans/polish-plan.md around line 24, the declared timeline "10-12 weeks" conflicts with the summed durations of steps 1–10 (lines 32,81,144,235,359,446,575,715,901,1071) which total 15.5 weeks; update the document so the aggregate estimate matches the step sums or reduce individual step durations to reach 10–12 weeks. Specifically either (a) change the timeline line to "15–16 weeks" (or "15.5 weeks") and add a note reconciling the math, or (b) revise the per-step durations to collectively sum to 10–12 weeks and reflect those new values in the steps; additionally state explicitly whether cross-language parity testing (lines 1302–1308) is included in the total hours and, if so, allocate/adjust time for it in the step breakdown.



============================================================================
File: docs/plans/polish-plan.md
Line: 334 to 338
Type: nitpick

[x] Task:
In docs/plans/polish-plan.md around lines 334–338 (and propagate to line 345), formalize the glob precedence rule: define "longest glob prefix" as the longest literal (non-wildcard) left-to-right path segment prefix (e.g., com.acme. > com. for com.acme.finance), state that when two patterns have equal literal prefix length but both match (e.g., com. vs .acme for com.acme) the registry MUST treat this as an ambiguous configuration and surface a validation error (do not silently pick lexicographically or first-seen), and add concrete examples showing a resolved case (com.acme. wins over com.) and an ambiguous case (com. vs .acme -> error) so implementers can detect and enforce ambiguity programmatically.



============================================================================
File: scripts/update_launch_program.py
Line: 73 to 74
Type: potential_issue

[x] Task:
In scripts/update_launch_program.py around lines 73-74, the code opens launch_path for writing without ensuring its parent directory exists, which raises FileNotFoundError if .vscode/ is missing; before opening the file, create the parent directory (e.g., call os.makedirs(os.path.dirname(launch_path), exist_ok=True)), and add an import for os at the top if not already present, then proceed to open and json.dump as before.



============================================================================
File: sea-core/src/kg_import.rs
Line: 131 to 137
Type: nitpick

[x] Task:
In sea-core/src/kg_import.rs around lines 131 to 137, replace the manual starts_with + slicing logic with strip_prefix to safely extract the remainder of the string; check path.strip_prefix("http://domainforge.ai/sea#") and path.strip_prefix("http://www.w3.org/2000/01/rdf-schema#") and, when Some(suffix) is returned, format!("sea:{}", suffix) or format!("rdfs:{}", suffix) respectively, otherwise fall back to path.clone(); this removes raw byte-index slicing and uses strip_prefix for safety and clarity.

============================================================================
File: sea-core/src/kg.rs
Line: 338 to 345
Type: potential_issue

[x] Task:
In sea-core/src/kg.rs around lines 338 to 345, the code finds the position of "sh:targetClass" in normalized_block but then slices the original block, causing incorrect extraction when URIs were replaced; change the slice to use normalized_block (i.e., compute rest from &normalized_block[pos + "sh:targetClass".len()..]) so the token extraction uses the same normalized string used for the find.



============================================================================
File: sea-core/src/kg.rs
Line: 286 to 311
Type: potential_issue

[x] Task:
sea-core/src/kg.rs lines 286-311: the current naive parsing uses rfind('.') and split_whitespace() which breaks on literals containing spaces or dots; replace this with a robust solution — either integrate a Turtle parser crate (rio_turtle or sophia) to parse the input into triples, or implement proper tokenization that scans the line honoring quoted literal boundaries and escaped quotes, finds the statement terminator outside quotes, splits subject/predicate/object by token-aware splitting, and only then normalizes URIs via shorten_token; ensure you handle literals with language tags and datatypes and return clear errors for malformed lines.



============================================================================
File: sea-core/src/sbvr.rs
Line: 426 to 431
Type: potential_issue

[x] Task:
In sea-core/src/sbvr.rs around lines 426 to 431, the code unconditionally uses unit_from_string("units") which discards any unit metadata from the SBVR term; update the logic to (1) attempt to extract a unit string from the term (for example a dedicated term.unit field or by parsing the term.definition/metadata if unit info is embedded), pass that extracted unit into unit_from_string, and (2) if no unit can be found, fall back to a sensible default and add a clear comment explaining this limitation and why the default is used; ensure any parsing failures are handled (e.g., return an error or log) rather than silently overwriting real unit data.



============================================================================
File: scripts/update_launch_program.py
Line: 48 to 49
Type: potential_issue

[x] Task:
In scripts/update_launch_program.py around lines 48 to 49, the code directly calls json.load(…) which will crash on a malformed launch.json; wrap the file read and json.load call in a try-except block that catches json.JSONDecodeError (and optionally Exception), log or print a clear error message including the file path and the exception message, and exit with a non-zero status (e.g., sys.exit(1)) to stop further processing; ensure the file is still closed (use the existing with open(...) context) and return or exit after reporting the error.


============================================================================
File: sea-core/src/kg.rs
Line: 609 to 628
Type: potential_issue

[x] Task:
In sea-core/src/kg.rs around lines 609 to 628, the code currently uses unwrap_or(rust_decimal::Decimal::ZERO) when parsing min_exclusive, silently treating invalid thresholds as 0; instead, parse the min_exclusive once before iterating and fail fast on parse errors by returning or propagating an Err (or using expect with a clear message if this function intentionally panics), including the invalid string and parse error; then use the successfully parsed Decimal threshold in the loop (trim the min_exclusive string before parsing) so invalid constraint values are not masked.


============================================================================
File: .github/agents/SubAgentOrchestrator.agent.md
Line: 4 to 29
Type: potential_issue

[x] Task:
.github/agents/SubAgentOrchestrator.agent.md lines 4–29 and around line 631: the frontmatter currently lists tools without distinguishing which are preinstalled versus user-provided; explicitly mark community/third‑party tools (e.g., Vibe Check) as optional and add tier labels (preinstalled/optional) in the frontmatter, then at the Vibe Check invocation near line 631 add a short note that Vibe Check must be added/hosted by the user and implement a fallback/skip behavior when it is not available (e.g., detect availability and skip call or use alternative tool), and update documentation text to show how to add Vibe Check if desired.



============================================================================
File: sea-core/src/kg.rs
Line: 442 to 461
Type: potential_issue

[x] Task:
In sea-core/src/kg.rs around lines 442 to 461, the current parsing is brittle: using split(':').nth(1) can break for IRIs with multiple ':' and the decimal extraction logic (trim_matches + split) yields "100^^xsd:decimal" instead of "100". Replace the naive split with taking the last segment after ':' (e.g., rsplitn(2, ':').next().unwrap_or(&p.object) or equivalent) for from/to/resource_name so you always get the local name; for quantity, extract the numeric literal robustly by first checking for quoted literals (find first and last '"' and take the substring between them), else if there's a type marker ("^^") take the substring before "^^", then trim and pass that to Decimal::from_str; set quantity only if parsing succeeds.



============================================================================
File: sea-core/src/kg_import.rs
Line: 139 to 148
Type: nitpick

[x] Task:
In sea-core/src/kg_import.rs around lines 139 to 148, the code manually checks and slices the datatype IRI prefix "http://www.w3.org/2001/XMLSchema#" to produce an "xsd:"-prefixed string; replace the manual starts_with + slicing with .strip_prefix("http://www.w3.org/2001/XMLSchema#") to more safely detect and remove the prefix, then map that result to format!("xsd:{}", suffix) when present, otherwise return dt.clone() (keeping the existing behavior for "xsd:" and other values).



Review completed ✔
