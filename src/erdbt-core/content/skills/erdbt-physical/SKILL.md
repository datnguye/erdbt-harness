---
description: "Phase 5 of the erdbt harness: run dbterd against the updated manifest to produce the physical ERD and close the loop against the concept model. Use after dbt generation is merged."
---

# Phase 5 — Physical ERD

{{ unimplemented }}

Render the ERD from what actually got built, and compare it to what phase 2
said would be built.

## Steps

1. Compile the target project so the manifest reflects the merge. Detect the
   artifact format rather than assuming it — `dbt-fusion 2.0.0-preview.209`
   writes `target/manifest.json`, and Fusion artifacts are still moving.
2. Run:

   ```
   erdbt physical --project <path> --out erdbt/erd/physical.json
   ```

   This wraps `dbterd`'s `json` target against the compiled manifest, so the
   result is the same payload shape `@datnguye/erd-flow` renders for the
   concept and logical ERDs. One viewer, three phases, one diff format.
3. Compare `erdbt/erd/physical.json` against `erdbt/concept.ir.yaml` and
   `erdbt/logical.ir.yaml`. Write the comparison to `erdbt/drift.md` and commit
   it: entities that never made it, relationships the warehouse has that no
   concept model predicted, cardinality that came out different.

## The comparison is the deliverable

Rendering the payload is the easy half. The reason this phase exists is
`erdbt/drift.md` — it is the only place the loop actually closes, and a phase 5
that emits a diagram and reports nothing has skipped its job.

Commit the drift report. A finding that lives only in a chat transcript is not
a deliverable: the next person to open this project sees the file or sees
nothing.

Drift is not automatically a bug. A relationship the warehouse has and the
concept model lacks may be a real discovery about the business. Report it,
attribute it, and let the human decide whether the fix belongs in the IR or in
the dbt project.

{{ artifacts-dir }}

{{ ir }}

Dispatch `erdbt-tester` for the drift comparison; it reads both the rendered ERD
and the concept model. `erdbt-planner` closes the run by reporting what the five
phases left outstanding.

{{ delegation }}

{{ gate }}
