---
name: dbt-conventions
description: How to read a dbt project's manifest and judge its structure — layering, naming, ref/source discipline, grain, and testing. Read before reporting conventions, reviewing project structure, or merging generated models.
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# dbt project conventions and structure

Phases 3, 4, and 5 all read the target project's compiled artifacts — phase 3
to borrow naming, phase 4 to merge without collisions, phase 5 to render the
physical ERD.

## Read the manifest, do not assume its format

Detect the artifact rather than assuming it. Measured against `dbt-fusion
2.0.0-preview.209`, `dbt parse` produces `target/manifest.json` at schema v12,
and **no Parquet artifact** — despite older briefs specifying one.

- Look for `target/manifest.json` first; it is what exists today.
- If a build emits Parquet, detect it by extension and branch. Hardcode neither.
- Check `metadata.dbt_schema_version` before parsing anything below. A version
  bump is the signal to re-verify every field here.

Fusion is in preview and its artifacts move. Re-verify against the user's
installed version before trusting this file.

The parts that matter:

- `nodes` — models, keyed `model.<project>.<name>`, each carrying `columns`,
  `config.materialized`, `depends_on.nodes`, and `path`
- `sources` — keyed `source.<project>.<source>.<table>`
- `child_map` / `parent_map` — the graph, for checking layer boundaries

## Report conventions with counts

Derive naming from `nodes[*].columns` and report what is **observed**, never
what is idiomatic:

- surrogate key style — `_key`, `_id`, `_sk`, hashed or sequential
- date and timestamp suffixes, and the timezone convention if visible
- boolean naming — `is_`, `has_`, `_flag`
- casing, pluralization, and the layer prefixes in use
- how sources are named and where `stg_` models pull from

Always give counts. "17 of 20 use `_key`, 3 use `_id`" lets the reader judge;
"the project uses `_key`" hides that it is not settled. A project with four
models must read as thin rather than yielding a confident convention from four
models — say so, and let the human set conventions explicitly at the gate.

## Structure: what good looks like

Judge the project's shape, not only its names. These are the checks worth
making, each with the reason it matters:

**Layers stay in order.** Staging reads sources; intermediate reads staging;
marts read intermediate or staging. A mart selecting straight from a source
means the lineage no longer explains where a number came from.

**Staging is thin.** One staging model per source table, doing renaming,
casting, and light cleanup — no joins, no aggregation, no business logic. A
staging model with a join is an intermediate model in the wrong directory, and
everything downstream inherits its grain by accident.

**`ref` and `source`, never a hardcoded relation.** A literal
`schema.table` in SQL is invisible to the DAG, so dbt cannot order it, test it,
or tell you what breaks when it changes.

**Every model has a declared grain.** A mart nobody can describe as "one row
per ___" is the single most expensive thing to discover late — it surfaces as a
silently fanned-out join in a number someone already reported.

**Primary keys are tested.** `unique` and `not_null` on the key of every model
that has one. These two tests catch the majority of real modeling regressions.

**Relationships are tested where they matter.** `relationships` tests are also
what `dbterd` reads to draw edges — a project without them yields an ERD with
entities and no edges, which is a finding about the project, not a phase
failure.

**Materialization matches use.** Views for cheap staging, tables or incremental
for expensive marts. Report a mismatch as a question, not a verdict; cost and
freshness are the human's call.

## Report structure findings as observations

The target project belongs to the human's team and predates this harness. Where
it departs from the above, report it with the evidence and the cost, and let
them decide. Never rewrite a project's structure to match a convention, and
never make a generated model violate a boundary just to fit an existing one —
that is a finding for the gate.
