# Documentation Guidelines — Diataxis + MECE

This file describes expected structure and minimal content for new documentation pages under `docs/new_docs`.

## General guidelines

- Write for a single audience per document.
- Keep documents focused: one main goal or concept.
- Use examples and short runnable snippets.
- Ensure linkages: each tutorial should link to how-tos and references.

## MECE checklist per category

### Tutorials

- Audience and goal
- Prereqs
- Step-by-step instructions with runnable examples
- Troubleshooting
- Related docs and next steps

### How-Tos

- Short goal/one-liner
- Step-by-step instructions (copy-paste-able)
- Common pitfalls and minimal verification steps

### Reference

- Stable, canonical API signatures and CLI options
- Example code snippets per API
- Version and compatibility notes

### Explanations

- Problem statement and background
- Conceptual diagrams and tradeoffs
- When/how to use the concept and links to examples

### Playbooks

- Clear scope and preconditions
- Step-by-step runbook with commands to run
- Safety checks and rollback steps
- Escalation/contacts

## Tips for maintainers

- Add `tags` at the top or metadata in the document so it can be surfaced by search/index.
- When adding new reference docs, ensure automated tests (where relevant) are added to `tests/` or `typescript-tests/`.
- Keep a single source of truth and avoid duplicated explanations. Link instead of copy-paste when possible.

By following these guidelines we achieve clarity, maintainability, and pragmatic coverage for new users, engineers, and operators.
