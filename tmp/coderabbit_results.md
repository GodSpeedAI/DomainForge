Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: docs/new_docs/templates/template_tutorial.md
Line: 17
Type: potential_issue

[x] Task:
In docs/new_docs/templates/template_tutorial.md around lines 17 and 19, the "Result/Verify:" and the other section at line 19 are missing the leading "##" header markup used elsewhere; update those lines to prepend "## " so they match the document's section header style and maintain proper markdown hierarchy.



============================================================================
File: docs/new_docs/templates/template_howto.md
Line: 16
Type: potential_issue

[x] Task:
In docs/new_docs/templates/template_howto.md around lines 16 and 18, the section headers are plain text instead of Markdown headings; change the lines to use Markdown heading syntax (prefix with "##") so "Common pitfalls/Troubleshooting:" becomes "## Common pitfalls/Troubleshooting" and the other section at line 18 likewise uses "##" prefix, preserving the text but removing the trailing colon if needed for consistency.



============================================================================
File: docs/new_docs/tutorials/wasm-in-browser.md
Line: 14
Type: potential_issue

[x] Task:
In docs/new_docs/tutorials/wasm-in-browser.md around lines 14,19,36 and 65, the tutorial has a path mismatch: the wasm-pack build command writes to ../examples/wasm-demo/pkg but the HTML import expects ./pkg/sea_core.js relative to index.html, so update the instructions to make paths consistent — either change the build output to ./pkg (when running from the sea-core/examples/wasm-demo directory) or explicitly state that index.html should be created at examples/wasm-demo/index.html and run wasm-pack from sea-core/examples/wasm-demo so the existing ../examples/wasm-demo/pkg path resolves; adjust the line that currently says "Create index.html." to "Create examples/wasm-demo/index.html." and ensure the HTML import path and build --out-dir match the same pkg location (and update any references at line 65 accordingly).



============================================================================
File: docs/new_docs/how-tos/create-custom-units.md
Line: 49
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/create-custom-units.md around lines 46–49, the doc runs "sea validate --format human examples/custom_units.sea" but never tells the user to create that file; add an explicit step before line 46 that instructs the reader to create examples/custom_units.sea (or point to the repo path of an existing example) and populate it with the DSL code produced in Steps 1–2 so the validation command has a file to run against.



============================================================================
File: docs/new_docs/how-tos/create-custom-units.md
Line: 7
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/create-custom-units.md around line 7, the prerequisite line using cargo install --path sea-core --features cli assumes a local sea-core directory and will fail for users who installed SEA via package managers or binaries; update the docs to either add an alternative installation command for non-source users (e.g., install the CLI from the official package/binary or via cargo install from crates.io) or clearly state that the cargo --path instruction only applies to developers building from source and provide links/instructions for other installation methods.



============================================================================
File: docs/new_docs/how-tos/create-custom-units.md
Line: 102 to 103
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/create-custom-units.md around lines 102 to 103, the three relative links may be incorrect for the current directory layout; verify that ../tutorials/first-sea-model.md, ../reference/grammar-spec.md, and ../reference/primitives-api.md actually exist relative to docs/new_docs/how-tos/, and if any are missing or relocated, update each link to the correct relative path (or use site-root-relative paths like /docs/... if your site supports it), then run the repo/site link checker (or build the docs locally) to confirm no broken links remain.



============================================================================
File: docs/plans/last_mile.md
Line: 1 to 70
Type: potential_issue

[x] Task:
In docs/plans/last_mile.md lines 1-70, the title and status language contradict the checked "✅ Completed" markers and the line saying "Release pipelines are non-existent"; pick one consistent framing (retrospective or plan) and update the file accordingly: if retrospective, rename the title to "V1 Stabilization & Release: Completion Report", change executive-summary language and all "Target/Outcome" lines to past-tense completed phrasing and keep checkboxes checked; if forward-looking, keep the current title, remove or uncheck all ✅ markers, change "Outcome:" to "Target Outcome:" or "Expected Result:", and change line 9 to state "Release pipelines: Pending" (or similar), ensuring Phase 3 no longer claims workflows are in place; make the checkbox state and the brief status sentence about release pipelines consistent across the document.



============================================================================
File: docs/new_docs/how-tos/create-custom-units.md
Line: 59
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/create-custom-units.md around lines 59 and 70, both the Python and TypeScript examples reference examples/custom_units.sea without explaining where to get or create it; consolidate the file-creation step used in the validation section into a single, earlier step that applies to all three examples (Python, TypeScript, and validation) by adding a short preface or subsection before the first code block that shows creating examples/custom_units.sea (its contents and path) and update the references at lines 59 and 70 to assume that pre-created file rather than introducing it locally in only one example.



============================================================================
File: .github/workflows/publish-python.yml
Line: 18 to 22
Type: potential_issue

[x] Task:
In .github/workflows/publish-python.yml around lines 18 to 22, the workflow references a custom action at .github/actions/decrypt-secrets and expects it to output pypi-api-token; ensure the composite action exists at .github/actions/decrypt-secrets/action.yml, add an outputs: declaration that defines pypi-api-token, and populate that output (e.g., from steps that decrypt secrets) so it is available as steps.secrets.outputs.pypi-api-token; also verify the repository has the secrets.SOPS_AGE_KEY secret configured in Settings → Secrets so the action can run.



============================================================================
File: docs/new_docs/how-tos/install-cli.md
Line: 85
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/install-cli.md at line 85, the verification checklist references running sea validate-kg which requires the undocumented shacl feature; update the installation instructions to either: a) explicitly list the shacl feature in the install command (e.g., add shacl to --features, such as --features cli,shacl), or b) modify the checklist to remove or adjust the sea validate-kg step to match the currently documented --features cli installation; pick one option and make the docs consistent (if choosing to add the feature, update the install command at line 18 and mention that shacl is required for validation).



============================================================================
File: docs/new_docs/playbooks/releasing-beta.md
Line: 48 to 56
Type: potential_issue

[x] Task:
docs/new_docs/playbooks/releasing-beta.md lines 48-56: the git tag step is currently placed after publishing which risks published packages not matching git history if a later step fails; move the "7. Git Tag" step to immediately after "3. Build & Test" (so version bump → build/test → git tag → publish to registries → GitHub release), renumber subsequent steps accordingly, and update any step references or cross-links in the document to reflect the new ordering.



============================================================================
File: .github/workflows/publish-python.yml
Line: 18 to 22
Type: potential_issue

[x] Task:
.github/workflows/publish-python.yml lines 18-22: the Decrypt secrets step can fail silently by terminating the workflow on SOPS decryption errors; either add explicit error handling by setting continue-on-error: true on this step so the workflow proceeds (and downstream steps can check for missing secrets), or modify the decrypt-secrets action to catch errors and always produce a well-defined output (e.g., an empty secrets output and a non-failing exit) so the workflow can safely continue without publishing.



============================================================================
File: docs/new_docs/how-tos/install-cli.md
Line: 24
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/install-cli.md around line 24, the instruction uses bash-specific syntax export PATH=... which breaks on Windows; move this step into a "Linux/macOS" subsection or provide platform-conditional variants. Update the document so step 2 is either relocated under a clearly labeled "Linux / macOS" heading or replace the single line with two platform-specific instructions: one for Linux/macOS (bash/zsh) and one for Windows (PowerShell and CMD) with the appropriate PATH modification commands and brief note indicating which shells each applies to.



============================================================================
File: docs/new_docs/playbooks/adding-new-primitive.md
Line: 53 to 58
Type: potential_issue

[x] Task:
In docs/new_docs/playbooks/adding-new-primitive.md around lines 53 to 58, the example Graph struct uses a bare // comment that is ambiguous; replace it with a clear placeholder comment such as "// ... other primitive collections (clusters, etc.)" to indicate there are additional fields and avoid confusion for developers copying the example. Ensure the comment succinctly conveys that other Graph fields exist and use an ellipsis or brief list of examples (clusters, nodes, edges) as appropriate.



============================================================================
File: docs/new_docs/how-tos/install-cli.md
Line: 31
Type: potential_issue

[x] Task:
In docs/new_docs/how-tos/install-cli.md around line 31 (also apply fixes for lines 61-63), the smoke-test commands reference sea-core/examples/basic.sea which is unavailable to users who install from the GitHub release; update the docs either by moving those example-based verification commands into a clearly labeled "From Source Only" subsection or by replacing them with release-safe verification steps (e.g., running a built-in CLI self-check or validating a minimal inline or generated example file), and if release artifacts include examples, explicitly document the path where they are installed so the verification commands work for release installs.



============================================================================
File: docs/new_docs/explanations/policy-evaluation-logic.md
Line: 52 to 55
Type: potential_issue

[x] Task:
In docs/new_docs/explanations/policy-evaluation-logic.md around lines 52 to 55, the description of the ValidationResult trace is vague; update the text to specify the exact trace schema and format users will receive: state that ValidationResult is an object with fields {policyName: string, result: "Pass" | "Fail" | "Unknown", trace: TraceEntry[]}, and define TraceEntry as an object with fields {id: string, type: string, displayName?: string, path?: string[], details?: Record} (explain types and optionality), and note that the trace is a JSON array of these entries and whether entries are full objects or references (IDs with optional summary fields). Keep the description concise and include one short example JSON output inline to illustrate the format.



============================================================================
File: docs/new_docs/playbooks/releasing-beta.md
Line: 62 to 63
Type: potential_issue

[x] Task:
In docs/new_docs/playbooks/releasing-beta.md around lines 62-63, replace the vague "Deprecate the version or publish a patch immediately" with explicit commands: for NPM add the exact npm deprecate command (e.g. npm deprecate @namespace/package@0.x.0 "Critical bug; use version 0.x.1 instead"), and for PyPI show both options — using the yank tool (install and run yank remove  ) and the alternative of publishing a hotfix release (bump to 0.x.1 and upload via twine), so readers have concrete commands to follow.



============================================================================
File: docs/new_docs/reference/python-api.md
Line: 183 to 184
Type: potential_issue

[x] Task:
In docs/new_docs/reference/python-api.md around lines 183 to 184, the two list items have an extra leading space that breaks Markdown list formatting; remove the extra leading space so each line begins with a hyphen (-) at column 1 (e.g., "- add_policy(policy): ..." and "- add_association(...") to restore proper list rendering.



============================================================================
File: .github/workflows/release-pypi.yml
Line: 27 to 32
Type: potential_issue

[x] Task:
.github/workflows/release-pypi.yml lines 27-32 (and usage at line 56): the decrypt-secrets step is gated to Linux only which means steps.secrets.outputs.pypi-api-token is missing on macOS/Windows and causes inconsistent/failing secret handling; remove the conditional "if: runner.os == 'Linux'" so the decrypt-secrets step runs on all runners (or alternatively implement platform-specific token handling), ensuring the step produces the expected outputs used later (e.g., steps.secrets.outputs.pypi-api-token) on every platform.



============================================================================
File: docs/new_docs/reference/python-api.md
Line: 207
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/python-api.md around line 207, remove the leftover git merge conflict marker line ">>>>>>> codex/review-and-merge-chore/fix-gaps-with-main" so the file no longer contains conflict markers; ensure no other conflict markers (<<<<<<< or =======) remain on adjacent lines and save the cleaned file.



============================================================================
File: docs/new_docs/tutorials/python-binding-quickstart.md
Line: 34 to 35
Type: potential_issue

[ ] Task:
docs/new_docs/tutorials/python-binding-quickstart.md lines 34-35: The SEA example declares two objects ("entity Web" and "resource DB") but the expected output at line 71 shows "Entities: 1"; verify the actual API behavior (does model.entities include resources or only entities) and then either (A) if the API counts both, update the expected output to "Entities: 2" (and adjust any other example outputs that assume only one entity), or (B) if the API excludes resources from model.entities, add a one-line clarification in the docs near the example explaining that resource objects are not counted in model.entities and leave the expected output as-is; apply the same correction to the other occurrence referenced (line 71).



============================================================================
File: docs/new_docs/tutorials/python-binding-quickstart.md
Line: 85 to 90
Type: potential_issue

[ ] Task:
In docs/new_docs/tutorials/python-binding-quickstart.md around lines 85-90, the pytest example uses flow.payload and flow.encrypted which are not shown or validated in the main script example (lines ~52-55); either update the main example to construct or demonstrate Flow objects that include and populate payload and encrypted attributes (add a simple Flow creation or parsing example showing those properties and their types), or, if those properties are not part of the public API, change the test to assert only on confirmed properties returned by domainforge.parse_file or consult the API maintainer to confirm and then update both example and test to match the validated contract.



============================================================================
File: docs/new_docs/tutorials/python-binding-quickstart.md
Line: 86
Type: potential_issue

[ ] Task:
In docs/new_docs/tutorials/python-binding-quickstart.md around line 86, the example calls domainforge.parse_file("production.sea") but the earlier example only shows domainforge.parse(), so verify whether parse_file() is part of the public Python API; if it does not exist, replace the call with the correct API (e.g., domainforge.parse(open("production.sea").read()) or domainforge.parse("production.sea") depending on the API signature) or update the docs to import/define parse_file(); ensure the docstring and example match the actual method name and usage.



============================================================================
File: docs/new_docs/playbooks/releasing-beta.md
Line: 13 to 19
Type: potential_issue




============================================================================
File: docs/new_docs/how-tos/parse-sea-files.md
Line: 8
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/parse-sea-files.md around lines 8 and 72, the npm package name is missing; update line 8 to instruct users to install the exact scoped package (for example: npm install @domainforge/sea) and update line 72 to import from that same package name (e.g., import ... from "@domainforge/sea"), ensuring both the prerequisite install command and the import statement use the identical, correct package identifier.



============================================================================
File: docs/new_docs/how-tos/parse-sea-files.md
Line: 7
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/parse-sea-files.md around lines 7, 16 and 25, the cargo install command lacks a binary name and the doc later uses the sea CLI; confirm which binary is produced and update the doc accordingly: either (a) change line 7 to include the explicit binary name if Cargo.toml defines [[bin]] name = "sea" (e.g., note that the install produces sea), or (b) if the binary is sea-core, change lines 16 and 25 to use sea-core validate and sea-core project, or (c) document that users must create an alias/wrapper and show the alias name; update the three lines so the install command and subsequent CLI invocations use the same binary name.



============================================================================
File: docs/new_docs/reference/primitives-api.md
Line: 195 to 197
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/primitives-api.md around lines 195 to 197, the TypeScript type for Quantity is documented as { value: string | number; unit?: string } while the note says "values are strings in JSON to preserve precision", creating a contradiction; update the TypeScript signature to only allow string for value (e.g., { value: string; unit?: string }) if precision must be preserved, or else change the note to reflect that numbers are permitted and explain the precision implications—pick one and make both the type and explanatory text consistent.



============================================================================
File: .github/workflows/release-npm.yml
Line: 18 to 22
Type: potential_issue

[ ] Task:
In .github/workflows/release-npm.yml around lines 18 to 22, the workflow references a local action at ./.github/actions/decrypt-secrets and expects it to output npm-token; verify that ./.github/actions/decrypt-secrets/action.yml exists, defines an outputs section with a key named npm-token, and that the action implementation actually writes the NPM registry token to that output (e.g., via echo "::set-output::npm-token=..."/appropriate composite/action output method); if missing or misnamed, add or correct the action file to produce npm-token, and ensure the repository secret SOPS_AGE_KEY is present so the decryption step can succeed.



============================================================================
File: docs/new_docs/reference/primitives-api.md
Line: 166 to 168
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/primitives-api.md around lines 166 to 168, the Rust constructor is listed as RelationType::new but the rest of the section and language bindings use Relation; update the Rust name for consistency: either change the Rust constructor to Relation::new to match the section and bindings, or if the canonical Rust type is RelationType, rename the section header and the other bindings references to RelationType; ensure the chosen canonical name is used consistently across the line and adjacent examples.



============================================================================
File: docs/new_docs/reference/error-codes.md
Line: 89 to 106
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/error-codes.md around lines 89 to 106, the E004 "Type Mismatch" entry is in the Syntax/Parsing section (E001–E099) but belongs to the Type System category (E100–E199); move the E004 block into the E100–E199 section (or merge its content into the existing E100 "Incompatible Types" entry) and remove the duplicate in the syntax section, updating any cross-references or index entries so E004 is categorized under the Type System range.



============================================================================
File: docs/new_docs/templates/template_reference.md
Line: 3 to 4
Type: potential_issue

[ ] Task:
In docs/new_docs/templates/template_reference.md around lines 3 to 4, the template currently uses plain "Title:" and "Summary:" labels; replace them with explicit placeholders and/or markdown headers so authors know what to fill in — for example, change to a markdown header for the title (e.g., "# " or "Title: ") and a clarified summary placeholder (e.g., "Summary: " or "## Summary" followed by the placeholder), ensuring the template shows concrete fill-in examples and consistent markdown formatting.



============================================================================
File: docs/new_docs/reference/error-codes.md
Line: 554 to 560
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/error-codes.md around lines 554-560 (and also 571-577), the error metadata token used in the TypeScript example ("__metadata__: ") is inconsistent with the WASM/JavaScript example ("__SEA_DSL_ERROR_METADATA__: "); verify the actual error string/token emitted by each binding implementation, then update the documentation to use the correct token for each binding (or normalize the code examples to a single canonical token if the implementations are changed to match), and if both different tokens are intentional add a short note explaining the difference and the rationale so readers know which parser to use for which binding.



============================================================================
File: docs/new_docs/templates/template_howto.md
Line: 9
Type: potential_issue

[ ] Task:
In docs/new_docs/templates/template_howto.md around line 9 there is an empty bullet point ("-") that should not be left blank; either remove the line entirely or replace the dash with meaningful placeholder text (e.g., "Add step here" or a short instruction) so the template contains no empty list items.



============================================================================
File: docs/new_docs/reference/error-codes.md
Line: 14 to 19
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/error-codes.md around lines 14 to 19 (and apply same change across lines 23–517), the Syntax/Parsing category E001–E099 only defines E001–E010 leaving E011–E099 undefined; update the document to either (A) explicitly mark E011–E099 as "reserved for future use" with a brief note and optionally a sentence about how new codes will be added, or (B) populate the missing codes if they were intended to be defined now; make the chosen change consistently for all similar incomplete ranges in the file (and other ranges noted in the comment) so the table clearly states whether unlisted codes are reserved or defined.



============================================================================
File: docs/new_docs/how-tos/export-to-calm.md
Line: 87
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/export-to-calm.md around line 87, the example uses Bash-only process substitution >(gzip > calm.json.gz) which is not cross-platform; replace it with a portable pipeline that writes a compressed file (for example pipe through gzip and redirect output to calm.json.gz) or instruct users to run gzip directly with redirection so the command works on Windows and POSIX shells.



============================================================================
File: docs/new_docs/reference/error-codes.md
Line: 67 to 87
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/error-codes.md around lines 67 to 87 (also apply the same change at 290–309), the E003 "Unit Mismatch" entry conflicts with E200 "Dimension Mismatch" and the categories are inconsistent; either consolidate into a single clearly-scoped error code or give each a distinct, non-overlapping definition and place them in the correct category. Update the document so that: 1) if consolidated, replace both entries with one unified code (move it into E200–E299 Unit/Dimension category), update title, description, example, common fixes and "expected vs found" suggestion; or 2) if kept separate, explicitly state the difference (E003 limited to parsing/token-level unit syntax errors in E001–E099, E200 for semantic dimension mismatches), adjust examples to show the distinct cases, and move E003 to the proper category or renumber it; finally, update any cross-references and the error-code index entries to reflect the chosen resolution.



============================================================================
File: docs/new_docs/reference/configuration.md
Line: 148
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/configuration.md around line 148, the parenthetical "(planned)" after --namespace is ambiguous; update the sentence to explicitly state that the --namespace flag is not currently available (for example: "For CALM imports, the default namespace is default unless overridden via a future --namespace flag (not yet available) or via post-processing.") so readers aren’t misled; keep the note brief and consider adding a link or reference to the roadmap/release notes if you want to track when it becomes available.



============================================================================
File: docs/new_docs/how-tos/define-policies.md
Line: 49 to 59
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/define-policies.md around lines 49 to 59, the Python example references policy.json but doesn't explain where it comes from or its format; update the doc to state whether policies are embedded in .sea or separate, and add a short, actionable step showing how to obtain/generate the JSON (e.g., export command or snippet: run the CLI/export function or a small Rust/Python helper to serialize crate::policy::Policy to JSON), describe the expected JSON schema briefly or link to the Rust type definition, and insert a preceding step pointing to the tool/command or sample file so users can produce policy.json before calling graph.evaluate_policy.



============================================================================
File: docs/new_docs/how-tos/define-policies.md
Line: 63 to 73
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/define-policies.md around lines 63 to 73, the TypeScript example references "policy.json" but doesn't explain where it comes from; update the docs to mirror the Python section by adding a brief sentence before the code block explaining how to obtain or generate policy.json (e.g., exported from the policy authoring tool or produced by the earlier example), or add a one-line example command/path showing the source; ensure the clarification matches wording and placement used in the Python example so readers know how to produce the policy.json referenced by the TypeScript snippet.



============================================================================
File: docs/new_docs/reference/error-codes.md
Line: 597 to 599
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/error-codes.md around lines 597–599 (and similarly lines 612–614), the relative links point to ../README.md and other paths that likely resolve incorrectly from docs/new_docs/reference/; verify that the target files actually exist and update each href to the correct relative path from this file (for example docs/new_docs/README.md or the correct subpath), or replace with the canonical site/absolute URL; if the referenced files do not exist, either create them at the expected location or remove/replace the links with correct targets and ensure all updated paths are tested locally to confirm they resolve.



============================================================================
File: docs/new_docs/reference/primitives-api.md
Line: 26
Type: potential_issue

[ ] Task:
docs/new_docs/reference/primitives-api.md lines 26 (and referenced lines 53, 81, 108, 133, 162, 195): The document references AttributeValue, AttributeMap (IndexMap), and Decimal but never defines them; add a brief "Types" section immediately before the Entity section that: 1) defines AttributeValue as the union used for attribute payloads (e.g., null, boolean, string, Decimal/number, byte array/base64, list/array of AttributeValue, and map/object of string→AttributeValue), 2) defines AttributeMap as the map type (IndexMap) and any key constraints (string keys, no null keys), and 3) defines Decimal as the arbitrary‑precision decimal type and its JSON serialization format (serialized as a string with an exact decimal representation; mention allowed format and any constraints such as sign, optional fraction, and that clients must preserve precision). Either include these concise structure/serialization rules or add a direct link to the formal definitions in sea-core if they already exist.



============================================================================
File: docs/new_docs/how-tos/parse-sea-files.md
Line: 8
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/parse-sea-files.md around lines 8 and 58, the install instruction and example import disagree: update line 8 to clarify the actual importable package name produced by that install (e.g., change the prerequisite to read something like "Optional: Python bindings (pip install -e sea-core[python] via maturin develop) — this installs the Python package as sea_dsl"), or alternatively change the import on line 58 to match the installed package name; make the prerequisite and the example import consistent so the install command and the import statement reference the same package name.



============================================================================
File: docs/new_docs/reference/calm-mapping.md
Line: 221 to 227
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/calm-mapping.md around lines 221–227, the Policy struct lists only id, name, and expression while the mapping rules later (lines 248–251) reference modality, kind, and priority which are undefined; update the Policy structure to include modality, kind, and priority (with types, e.g., Option or appropriate enums/primitives) and document them, or alternatively remove or change the mapping rules to only reference fields that exist — ensure the chosen fix is applied consistently in both the struct definition and the mapping rules sections so they match.



============================================================================
File: docs/new_docs/tutorials/typescript-binding-quickstart.md
Line: 74 to 76
Type: potential_issue

[ ] Task:
In docs/new_docs/tutorials/typescript-binding-quickstart.md around lines 74 to 76, the test currently reads system.sea synchronously at describe scope which can crash module loading; declare a let model variable in the describe scope and move the fs.readFileSync/parse call into a beforeAll hook (or beforeEach if appropriate), assigning the parsed result to model so test discovery won't fail if the file is missing or unreadable at load time.



============================================================================
File: docs/new_docs/reference/calm-mapping.md
Line: 267 to 278
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/calm-mapping.md around lines 267 to 278, the escaping rule says only string literals with spaces or special characters should be wrapped in double quotes, but the example uses "Camera" with quotes; update the example to remove the quotes so it reads (flow.resource = Camera) to match the stated rule (or alternatively, if you prefer quoting all string literals, change the escaping rule to state that all string literals are quoted and leave the example as-is) — choose one approach and make the example and rule consistent.



============================================================================
File: docs/new_docs/reference/calm-mapping.md
Line: 369 to 374
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/calm-mapping.md around lines 369–374, the semantic equivalence rules use field names that don't match the actual structures (e.g., "resource_name" for Instances and Flows), so update the text to reference the exact field names from the data models and state the chosen matching strategy: either (A) match on internal IDs (e.g., resource_id, entity_id, instance id/serial) with a note that IDs are remapped during import, or (B) match on resolved human-readable names from the graph (e.g., resource_name, entity_name) and explain how names are resolved; replace any incorrect "resource_name" occurrences with the correct field names (resource_id or resource name as per your chosen strategy), correct the Instance rule to use its actual fields (e.g., entity_id and id/serial if matching by ID), and explicitly document which approach is used for all Entities, Resources, Flows, Instances, and Policies so field names and matching semantics are consistent.



============================================================================
File: docs/new_docs/reference/calm-mapping.md
Line: 156 to 162
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/calm-mapping.md around lines 156–162 and 364–373, the Instance struct lists only id, entity_id, and attributes but the semantic-equivalence section references resource_name, entity_name, and an undefined serial; either add a serial field to the Instance definition if the implementation actually includes one (and document its type/meaning), or update the semantic equivalence text to use the actual fields (e.g., match instances by id or by entity_id plus attributes/resource identifier) — choose the correct option based on the implementation and make the docs consistent by removing references to resource_name/entity_name/serial or replacing them with id/entity_id/attributes accordingly.



============================================================================
File: docs/new_docs/reference/typescript-api.md
Line: 180 to 182
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/typescript-api.md around lines 180 to 182, the Graph.parse() signature lacks any documentation of what errors or exceptions it can throw; update the docs to describe the error conditions and the exception type(s) thrown (e.g., SyntaxError or a library-specific ParseError), list common invalid-input cases (empty source, malformed nodes/edges, unterminated multiline strings), and show recommended handling patterns (try/catch example description and suggested error properties to inspect like message and position). Ensure the new text is concise, names the exact exception type(s) thrown by the implementation, and includes guidance for callers on how to detect and respond to parse failures.



============================================================================
File: tests/test_instance.py
Line: 2
Type: potential_issue

[ ] Task:
In tests/test_instance.py around lines 2 (and also at lines 10, 21, 30, 40, 55), update all docstrings and inline comments that currently reference "Instance" to use the new class name "ResourceInstance" for consistency with the code; search the file for any occurrences of "Instance" in docstrings/comments and replace them with "ResourceInstance", keeping sentence structure and capitalization intact.



============================================================================
File: .sops.yaml
Line: 1 to 12
Type: potential_issue

[ ] Task:
In .sops.yaml around lines 1–12 the file contradicts its own guidance by hardcoding an age recipient; either remove the hardcoded key entirely and leave the age field absent (so developers must supply recipients via CLI/env at encrypt time) or replace the value with a clear placeholder (e.g. ) and add a prominent commented header stating this is a per-developer template and must be replaced or injected at runtime; ensure the age field remains a YAML list (or is removed cleanly) so the file stays valid YAML and add one line instructing how to pass the key via sops CLI/env.



============================================================================
File: docs/new_docs/reference/typescript-api.md
Line: 183 to 186
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/typescript-api.md around lines 183 to 186, the signature for static importCalm(json: string): Graph lacks any description of its error semantics; update the documentation to state exactly what happens on invalid or malformed JSON (for example: that it throws a SyntaxError from JSON.parse and/or a GraphImportError for schema/validation failures), list the exception types that callers should expect, and show a short example try/catch pattern; ensure the doc includes whether it ever returns null/undefined or always throws on error so users know to handle exceptions explicitly.



============================================================================
File: tests/test_role_relation_parity.py
Line: 10 to 13
Type: potential_issue

[ ] Task:
In tests/test_role_relation_parity.py around lines 10 to 13, there is a duplicate assertion asserting role.id is not None twice; remove the redundant second assertion (line 13) so the test only checks role.id is not None once, leaving any subsequent unique checks (e.g., ID format) intact.



============================================================================
File: typescript-tests/golden-payment-flow.test.ts
Line: 32 to 43
Type: nitpick

[ ] Task:
In typescript-tests/golden-payment-flow.test.ts around lines 32 to 43, the test checks roles and relations but doesn't verify flow properties despite the test name; add assertions after the relation checks to validate the parsed flows by calling graph.allFlows(), asserting length is 1, and then asserting expected flow properties such as resourceName === 'Money' and quantity === 10 (or the correct expected values) so the test fully validates flows.



============================================================================
File: docs/new_docs/how-tos/extend-grammar.md
Line: 82
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/extend-grammar.md around line 82, the reference to validation_error.rs is missing its repo-relative path; locate the actual validation_error.rs file in the repository (search for the filename) and update line 82 to include the full repo-relative path to that file (e.g., path/from-repo-root/to/validation_error.rs) so readers can directly find the error codes; ensure the path is accurate and keep the original wording otherwise.



============================================================================
File: docs/new_docs/playbooks/debugging-parser-failures.md
Line: 22 to 29
Type: potential_issue

[ ] Task:
In docs/new_docs/playbooks/debugging-parser-failures.md around lines 22 to 29, the guidance incorrectly implies the Pest Debugger is a web-based visual tool on pest.rs; update the text to clearly distinguish the pest.rs fiddle (web-based grammar playground) from the command-line Pest Debugger (provided by the pest_debugger crate), and replace lines 26–29 with a short note directing readers to use the online fiddle for quick grammar experiments and to install/run the pest_debugger CLI for interactive debugging, including a pointer to the crate's repository or docs and a minimal mention of how to invoke it (install via cargo and run against your grammar/input).



============================================================================
File: .github/actions/decrypt-secrets/action.yml
Line: 30
Type: potential_issue

[ ] Task:
.github/actions/decrypt-secrets/action.yml around line 30: the step using "sudo mv sops /usr/local/bin/sops" can fail on runners without sudo; replace this with moving sops into a user-writable bin (e.g., create $HOME/.local/bin if missing, mv the sops binary there and chmod +x it) and then add that directory to the PATH for subsequent steps (write it to GITHUB_PATH). Ensure the directory creation, move, and chmod are done without sudo and that the new path is exported via GITHUB_PATH so sops is available.



============================================================================
File: docs/new_docs/reference/wasm-api.md
Line: 87 to 91
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/wasm-api.md around lines 87 to 91, the API list only documents Entity, Resource, Flow, Instance, and Graph but index.js now exports six additional entities (ResourceInstance, Metric, Mapping, Projection, Role, Relation); update this section to add brief entries for each new export including a one-line description, the constructor/signature pattern (parameter names and types), and a short usage example showing typical instantiation and common methods; ensure formatting matches the existing bullet list style and include a note about any special validation or relationships (e.g., how Mapping links Entities/Resources, Projection usage for views, Relation/Role semantics) so consumers can construct and use these objects consistently.



============================================================================
File: index.js
Line: 313
Type: potential_issue

[ ] Task:
In index.js around line 313, the JS expects nativeBinding to export ResourceInstance, Metric, Mapping, Projection, Role, and Relation but the native Rust/C++ binding may not export them; verify the native module's export list and add those symbols to the native exports (or implement/wire the missing types) and rebuild the native addon so they are present, and optionally add a short runtime existence check in index.js that throws a clear error if any required symbol is missing to fail fast during module load.



============================================================================
File: docs/new_docs/reference/grammar-spec.md
Line: 42 to 46
Type: potential_issue

[ ] Task:
In docs/new_docs/reference/grammar-spec.md around lines 42 to 46, the quoted rule currently rejects escaped quotes; update the rule so its inner repetition accepts either a backslash-escape sequence or a non-quote character. Concretely, change the repetition to allow a backslash followed by an escape character (at least: ", \, n, r, t) as one alternative and any character that is not an unescaped quote as the other, preserving the surrounding double quotes.



============================================================================
File: docs/new_docs/how-tos/extend-grammar.md
Line: 40 to 42
Type: potential_issue

[ ] Task:
In docs/new_docs/how-tos/extend-grammar.md around lines 40–42, replace the vague TypeScript and WASM paths with exact file locations: update the TypeScript note to reference sea-core/src/typescript/primitives.rs and sea-core/src/typescript/index.d.ts, and update the WASM note to reference the concrete export file(s) such as sea-core/src/wasm/mod.rs (or sea-core/src/wasm/lib.rs if your crate uses lib.rs) so readers can find and modify the exact binding files.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 646 to 649
Type: potential_issue

[ ] Task:
In sea-core/src/typescript/primitives.rs around lines 646 to 649, the getter uses self.inner.attributes().get(&key) directly which is asymmetric with set_attribute and the Entity pattern that call self.inner.get_attribute(&key); check whether the inner type actually exposes a get_attribute(&str) -> Option/Option method — if it does, replace the direct attributes().get(...) call with self.inner.get_attribute(&key) and convert/serialize its return to String as needed; if it does not, either add a get_attribute method on the inner type to encapsulate access or add a clarifying comment explaining why direct map access is intentional and safe (and ensure behavior matches set_attribute semantics).



============================================================================
File: sea-core/src/parser/mod.rs
Line: 83 to 90
Type: potential_issue




============================================================================
File: .github/actions/decrypt-secrets/action.yml
Line: 43 to 46
Type: potential_issue

[ ] Task:
.github/actions/decrypt-secrets/action.yml lines 43-46: the current sed patterns expect values wrapped in double quotes and will return empty strings for unquoted YAML values; replace the sed-based extraction with a grep that locates the key (e.g. '^KEY:') piped into cut -d' ' -f2- to capture everything after the colon and first space, and then strip any optional surrounding quotes from the captured value (retain the existing || echo "" fallback).



============================================================================
File: tests/test_golden_payment_flow.py
Line: 38 to 42
Type: nitpick



                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               
Review completed ✔
