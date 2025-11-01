---
adr: ADR-008
title: "Lockstep Versioning Strategy"
diagram_type: C4Component
section: Decision
description: "Core components implementing lockstep versioning"
generated: 2025-11-01
---

# ADR-008 — Lockstep Architecture Components

Component view describing how the lockstep strategy is enforced architecturally.

```mermaid
%% Source: ADR-008 - Lockstep Versioning Strategy
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-008 Components — Lockstep Release Architecture
  Container(releasePlatform, "Lockstep Release Platform", "CI Orchestrator", "Coordinates multi-package releases")
  Component(versionManifest, "Version Manifest", "Source of Truth", "Stores single semver per release")
  Component(changelogAggregator, "Changelog Aggregator", "Docs Generator", "Combines changes across languages")
  Component(compatValidator, "Compatibility Validator", "QA Automation", "Ensures bindings align with core")
  Component(notificationHub, "Notification Hub", "Comms", "Notifies teams and users")
  Rel(releasePlatform, versionManifest, "Reads/updates version number")
  Rel(releasePlatform, changelogAggregator, "Publishes release notes")
  Rel(compatValidator, releasePlatform, "Blocks release on failures")
  Rel(notificationHub, releasePlatform, "Sends release notifications")
```

- Related: [Release pipeline implementation](ADR-008-component-release-pipeline.md)
