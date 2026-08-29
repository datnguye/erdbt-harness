---
name: dbt-fusion-manifest
description: "How to read the dbt Fusion (v2) manifest for naming conventions and the physical ERD. Read before writing conventions, dbt-merge, or physical logic."
---

# The dbt Fusion manifest

This content now **ships with the plugin** as `dbt-conventions`, authored at
`src/erdbt-core/content/skills/dbt-conventions/SKILL.md`. It covers manifest
reading plus the project-structure checks `erdbt-dbt-engineer` makes as the
team's architect.

Read and edit it there. `erdbt-dbt-engineer` and `erdbt-tester` preload it at
runtime, so it has to be part of what gets installed rather than a dev-time
note.

## The one thing worth repeating here

Detect the artifact format at runtime. Measured against `dbt-fusion
2.0.0-preview.209` on 2026-08-31, `dbt parse` emits `target/manifest.json` at
schema v12 and **no Parquet artifact**, despite the original brief specifying
one. Fusion is in preview; re-verify before trusting either shape.
