---
description: "Phase 1 — turn a business requirements doc into a structured requirements summary."
argument-hint: "<path-to-requirements-doc>"
---

Run phase 1 of the erdbt harness using the `erdbt-intake` skill.

Source document: $ARGUMENTS

If that is empty, the skill resolves the brief itself. Read it, write
`requirements.yaml` in the resolved artifacts directory, and stop at the gate
for review.
