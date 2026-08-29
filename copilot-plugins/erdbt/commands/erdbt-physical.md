---
description: Phase 5 — run dbterd against the updated manifest and report drift from the concept model.
argument-hint: '[--project <path-to-dbt-project>]'
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

Run phase 5 of the erdbt harness using the `erdbt-physical` skill.

Target dbt project: $ARGUMENTS

Render the ERD from the compiled manifest, then report every drift
between the physical result and the concept model.
