# OMC Stalling Fix — Report/Write Tasks

**Date:** 2026-06-12
**Problem:** Model stalls indefinitely on report-writing and documentation tasks after oh-my-claudecode installation.
**Trigger:** Any task whose output is a markdown/JSON report, not code changes.

## Root Cause

Two rules in CLAUDE.md create an infinite delegation loop for write-heavy tasks:

1. **"NEVER do code changes directly — delegate to executor/writer"** (line 13-37)
2. **"Never claim completion without Architect approval"** (line 211-219)

For report-writing tasks:
- Writing IS the task, but the model classifies it as "documentation" and tries to delegate to `writer`
- Architect verification before completion adds another delegation round-trip
- The model keeps gathering evidence (feels productive) instead of writing (feels like it should be delegated)

## Fix

For tasks where the deliverable IS a document/report (not code), the operator should:

1. **Tell the model explicitly**: "Write this yourself, don't delegate"
2. Or the operator can add a line to their CLAUDE.md:

```
### Exception: Direct Writing Tasks
When the task deliverable is a report, document, or analysis (not code), write directly.
Do not delegate to writer agent. Architect verification is NOT required for reports.
```

3. Or use a keyword like "write this now" to signal bypass.

## Pattern to Watch

If you see the model launching 3+ explore/librarian agents for a writing task, it's stalling.
Interrupt and say "write it now" or "stop gathering, just write".
