---
description: Phase 4 — generate dbt SQL and YAML from the logical model and merge into the project.
argument-hint: '[--project <path-to-dbt-project>]'
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

Run phase 4 of the erdbt harness using the `erdbt-generate` skill.

Target dbt project: $ARGUMENTS

Preview against DuckDB first, then merge with `erdbt dbt-merge`. Show the diff.
Do not commit.
