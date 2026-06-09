# Copilot Instructions

Use [AGENTS.md](../AGENTS.md) as the canonical source of repository-wide instructions.

Copilot-specific rules:

- Follow `AGENTS.md` for workflow, testing, routing, and state-management behavior.
- Use `.agents/` as the active shared work-state store.
- Treat `.agents/` as the only current agent-state directory in this repository.
- If `.agents/` or its required subfolders are missing and the task needs them, create them before continuing.
- Keep `.github/copilot-instructions.md` minimal; do not duplicate repository policy here.
- When work spans multiple steps or handoff is likely, update `.agents/current_state.md` and `.agents/next_steps.md`.
- You MUST read and follow the instructions in `AGENTS.md` and `.agents/current_state.md` before taking any action.
