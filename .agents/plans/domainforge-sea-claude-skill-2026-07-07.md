# DomainForge SEA Claude Skill Plan

Outcome: Create a portable, repo-local Claude skill that enables agents to author DomainForge `.sea` files and operate the DomainForge CLI without depending on project-local docs.

Artifacts:

- `.claude/skills/domainforge-sea/SKILL.md`
- `.claude/skills/domainforge-sea/references/write-in-sea.md`
- `.claude/skills/domainforge-sea/references/cli-commands.md`
- `.claude/skills/domainforge-sea/references/sea-dsl-ai-cheatsheet.yaml`
- `.claude/skills/domainforge-sea/agents/openai.yaml`
- `.agents/current_state.md` and `.agents/next_steps.md` handoff updates

Acceptance:

- Skill frontmatter has precise trigger metadata.
- Skill is self-contained by bundling the requested source docs.
- Skill body tells agents when to load each bundled reference.
- Skill includes CLI workflows, SEA authoring rules, validation expectations, and common pitfalls.
- Proof verifies required files, metadata, and reference copies exist.
