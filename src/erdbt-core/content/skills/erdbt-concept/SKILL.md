---
description: "Phase 2 of the erdbt harness: build a conceptual model of entities and relationships only, with no attributes or types. Use after requirements intake is signed off."
---

# Phase 2 — Concept model

{{ unimplemented }}

Turn `erdbt/requirements.yaml` into `erdbt/concept.ir.yaml`: **entities and
relationships only**. Phase 3 will expand it into a separate
`logical.ir.yaml`, so this file stays a clean statement of the business model.

## The hard boundary

No attributes. No data types. No keys. No column names. If you catch yourself
writing `customer_id`, you are in phase 3 and you have skipped a gate.

This restraint is the point of the phase. A reviewer can check thirty entities
and their relationships in one sitting; the same review buried in four hundred
typed columns gets rubber-stamped. Keep the diff reviewable.

## Steps

1. Read `erdbt/requirements.yaml`. If it is missing or unsigned, stop — phase 1
   owns that file.
2. Write each entity with its `name`, one-line `definition` in business terms,
   and the `evidence` that justifies it.
3. Write each relationship as `from`, `to`, `verb`, and `cardinality`
   (`one-to-one`, `one-to-many`, `many-to-many`). Use `unknown` when the
   requirements do not say — never default to `one-to-many`.
4. Render the ERD: `erdbt erd concept` writes `erdbt/erd/concept.json` in the
   payload shape `@datnguye/erd-flow` renders. Commit it — a reviewer reads
   the diagram, and its diff is where a lost or rewired entity shows up.
5. Run `erdbt ir diff` to render the change for review.

{{ artifacts-dir }}

{{ ir }}

{{ no-invention }}

Once the entities and relationships are drafted, dispatch `erdbt-modeler` to
check them against the requirements, and `erdbt-reviewer` on the diff. Both at
once — they read different things.

{{ delegation }}

{{ gate }}
