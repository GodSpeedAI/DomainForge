# Work As An Agentic Developer

Goal: Use DomainForge effectively from an individual agentic development setup such as Claude Code CLI, OpenCode or Open Claw-style CLIs, Hermes, Codex, or another local agent runner.

This guide is the companion to [Write In SEA](write-in-sea.md). That guide explains the `.sea` language. This guide explains how an agentic developer should work with the repository, prompts, state, proof, generated bindings, and the reality that an agent can act faster than it can understand.

## Prerequisites

- A local checkout of DomainForge.
- Rust toolchain and `just` available.
- Optional Python and TypeScript toolchains if the task touches bindings.
- An agentic coding tool that can read files, edit files, and run shell commands.
- The repository instructions in [AGENTS.md](../../AGENTS.md) and [.github/copilot-instructions.md](../../.github/copilot-instructions.md).

Build or refresh the CLI when the task needs live SEA validation:

```bash
cargo install --path domainforge-core --features cli --force
```

Run the default narrow proof:

```bash
just ai-validate
```

Run full cross-language proof before claiming changes that affect public behavior:

```bash
just all-tests
```

## Start With A Task Packet

Before asking an agent to work, write the task as an execution packet. Keep it short, concrete, and proof-oriented.

```text
Task: Add support for X in DomainForge.

Read first:
- AGENTS.md
- .github/copilot-instructions.md
- .agents/current_state.md
- .agents/next_steps.md
- docs/how-tos/write-in-sea.md

Constraints:
- Rust core is canonical.
- Do not overwrite existing uncommitted work.
- If a primitive or public API changes, update Rust, Python, TypeScript, and WASM bindings as applicable.
- Validate with the narrowest relevant command, then run broader proof before completion.

Done when:
- The changed files are named.
- The proof commands and outcomes are reported.
- .agents/current_state.md and .agents/next_steps.md reflect the handoff state.
```

Use this shape in Claude Code CLI, OpenCode/Open Claw, Hermes, Codex, or any comparable runtime. Tool syntax changes; the operating contract should not.

## Respect The Agentic Reality

Agents are useful because they can search, edit, and verify quickly. They are risky because they can also infer missing contracts, overfit stale examples, overwrite local work, or call a summary "done" before proof.

Treat these as default failure modes:

- The agent may trust outdated docs over the Rust grammar.
- The agent may edit generated bindings instead of canonical Rust sources.
- The agent may miss uncommitted work already in the tree.
- The agent may run one narrow test and overclaim system readiness.
- The agent may lose context after a restart unless `.agents/` is current.

Counter those failure modes with concrete instructions, small edits, and proof commands.

## Use The Repository State Store

DomainForge uses `.agents/` as shared local working memory. It is not product documentation, but it is the handoff channel between agent sessions.

Before material work, make the agent read:

```bash
sed -n '1,220p' .agents/current_state.md
sed -n '1,180p' .agents/next_steps.md
```

During or after material work, update:

- `.agents/current_state.md` with checklist items and evidence.
- `.agents/next_steps.md` with only the next two or three steps.
- `.agents/plans/` or `.agents/specs/` when the work is multi-step or architectural.
- `.agents/lessons/` only for reusable mistakes or workflow improvements.

Do not put secrets, tokens, private keys, or raw logs in `.agents/`.

## Choose The Right Proof Command

Use the smallest command that can disprove the current claim, then broaden before completion.

| Change type | First proof | Completion proof |
| --- | --- | --- |
| Documentation only | `rg -n "new guide title|linked guide" docs` | `just ai-validate` when the doc changes supported commands or examples |
| SEA example or grammar docs | `domainforge validate --format human path/to/file.sea` | `just rust-test` |
| Parser or grammar | `just rust-test` | `just all-tests` |
| Rust primitive or public API | `just rust-test` | `just all-tests` |
| Python binding | `just python-test` | `just all-tests` |
| TypeScript binding | `just ts-test` | `just all-tests` |
| CLI behavior | `just cli-test` | `just all-tests` |
| Projection behavior | targeted Rust projection test | `just all-tests` |

If the command fails, continue from the failure output. Do not replace proof with explanation.

## Prompt Patterns

### Claude Code CLI

Use Claude Code for iterative local edits where filesystem context matters.

```text
Read AGENTS.md, .github/copilot-instructions.md, .agents/current_state.md, and .agents/next_steps.md.
Then inspect the relevant source files before editing.
Do not overwrite unrelated uncommitted changes.
Make the smallest change that satisfies the task.
Run the proof command and report the exact command outcome.
Update .agents/current_state.md and .agents/next_steps.md.
```

### OpenCode Or Open Claw-Style CLI

Use OpenCode/Open Claw-style tools for explicit repo-local tasks. Give them a bounded file set when possible.

```text
Scope:
- domainforge-core/grammar/sea.pest
- domainforge-core/src/parser/
- domainforge-core/tests/parser_*.rs

Task:
Implement the syntax described below. Start at the grammar, then AST, then graph conversion.

Proof:
Run just rust-test. If public bindings changed, run just all-tests.
```

### Hermes Or Agentic System Runners

Use Hermes or other orchestrated agentic systems when the task naturally splits into investigation, implementation, and review. Keep the handoff artifacts explicit.

```text
Produce:
- investigation notes under .agents/reports/
- implementation plan under .agents/plans/ if the change spans multiple modules
- code/docs changes
- proof command output summary
- updated .agents/current_state.md and .agents/next_steps.md

Reviewer rule:
Find bugs, missing tests, stale bindings, and overclaims. Do not praise before proof.
```

## Work Loop

1. Read `AGENTS.md`, `.github/copilot-instructions.md`, `.agents/current_state.md`, and `.agents/next_steps.md`.
2. Inspect `git status --short` before editing.
3. Identify the canonical source. For SEA syntax, start with `domainforge-core/grammar/sea.pest` and `domainforge-core/src/parser/ast.rs`.
4. Make the smallest coherent change.
5. Run the narrow proof command.
6. Run broader proof when the change crosses Rust, Python, TypeScript, or WASM surfaces.
7. Update `.agents/current_state.md` and `.agents/next_steps.md`.
8. Report changed files, proof commands, skipped checks, and remaining risks.

## Keep Canonical Sources Straight

- SEA syntax: `domainforge-core/grammar/sea.pest`.
- AST shape: `domainforge-core/src/parser/ast.rs`.
- Semantic graph behavior: `domainforge-core/src/graph/` and `domainforge-core/src/primitives/`.
- Policy logic: `domainforge-core/src/policy/`.
- Python bindings: `domainforge-core/src/python/` and `tests/`.
- TypeScript bindings: `domainforge-core/src/typescript/` and `typescript-tests/`.
- WASM bindings: `domainforge-core/src/wasm/`.
- CLI behavior: `domainforge-core/src/cli/` and `docs/reference/cli-commands.md`.

When in doubt, make the agent state which file is canonical before it edits anything.

## Common Pitfalls

- **Uncommitted work already exists**: run `git status --short` and avoid reverting or formatting unrelated files.
- **Docs drift from parser behavior**: verify examples with `domainforge validate` or `domainforge parse --ast`.
- **Rust-only fix breaks bindings**: run `just all-tests` when public primitives, graph behavior, or parser output changes.
- **Agent edits generated output**: find the generator or canonical source instead.
- **The prompt is too broad**: split investigation, implementation, and review into separate passes.
- **The final answer overclaims**: require changed files and observed proof command outcomes.

## Handoff Checklist

Use this before ending an agentic development session:

- [ ] `git status --short` reviewed.
- [ ] Changed files named.
- [ ] Proof commands run or skipped with a concrete reason.
- [ ] `.agents/current_state.md` updated with checklist evidence.
- [ ] `.agents/next_steps.md` contains only the next two or three steps.
- [ ] Any known follow-up risk is recorded in `.agents/next_steps.md`, `.agents/reports/`, or the final response.

## Links

- How-to: [Write In SEA](write-in-sea.md)
- How-to: [Parse SEA Files](parse-sea-files.md)
- How-to: [Run Cross-Language Tests](run-cross-language-tests.md)
- Reference: [CLI Commands](../reference/cli-commands.md)
- Repository instructions: [AGENTS.md](../../AGENTS.md)
