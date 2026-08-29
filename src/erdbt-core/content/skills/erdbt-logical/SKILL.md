---
description: "Phase 3 of the erdbt harness: expand the concept model into attributes, keys, and cardinality, following the target dbt project's naming conventions. Use after the concept model is signed off."
---

# Phase 3 — Logical model

{{ unimplemented }}

Expand the signed-off `erdbt/concept.ir.yaml` into a new file,
`erdbt/logical.ir.yaml`, carrying attributes, keys, and resolved cardinality.

**Never edit `concept.ir.yaml`.** It was signed off at a gate. If the concept
model is wrong, that is a phase 2 revision and a new gate — not a quiet fix
from inside phase 3.

## Borrow conventions, do not invent them

Before naming anything, read the target dbt project's manifest and catalog:

```
erdbt conventions --project <path-to-dbt-project>
```

This reports the naming patterns already in use — surrogate key style, date
column suffixes, boolean prefixes, casing. **Follow what it reports**, even
where your own preference differs. A model that names it `is_active` in a
project that uses `active_flag` is a model the team has to relearn.

If the project is empty or has no manifest, say so and propose conventions
explicitly at the gate rather than quietly picking your own.

## Steps

1. For each entity, add `attributes` — `name`, `type`, `description`,
   `nullable`, and `source` (where the value comes from).
2. Mark exactly one `primary_key` per entity. Add `foreign_keys` for each
   relationship the concept model recorded.
3. Resolve every `unknown` cardinality from phase 2, or escalate it. Shipping an
   `unknown` into phase 4 generates a join no one can verify.
4. Resolve many-to-many relationships into an associative entity, naming it by
   the project's convention. An associative entity has no concept counterpart,
   so record it under `added_in_logical` with the relationship it resolves —
   that is the one routine reason a logical entity may not trace back, and
   `erdbt ir check` fails on any that is not declared.
5. Run `erdbt ir check`. It fails when an entity in `logical.ir.yaml` has no
   counterpart in `concept.ir.yaml`, when an `unknown` cardinality survives, or
   when a relationship points at an entity that no longer exists. Splitting the
   two files is what makes this check necessary — do not present phase 3 at its
   gate until it passes.
6. Render the ERD: `erdbt erd logical` writes `erdbt/erd/logical.json`, now
   with real columns and PK/FK badges. Commit it alongside the IR.
7. Run `erdbt ir diff` for review.

## Types

Use the warehouse's own type names, not generic ones. Keep numeric precision
explicit for anything monetary — a money column that lands as a float is a bug
that surfaces a quarter later in a reconciliation report.

{{ artifacts-dir }}

{{ ir }}

{{ no-invention }}

Dispatch `erdbt-dbt-engineer` for the project's conventions and `erdbt-modeler`
for the attribute and cardinality review in the same message; neither waits on
the other. `erdbt-reviewer` goes last, on the finished diff.

{{ delegation }}

{{ gate }}
