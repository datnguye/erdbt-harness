---
name: erdbt-logical
description: 'Phase 3 of the erdbt harness: expand the concept model into attributes, keys, and cardinality, following the target dbt project''s naming conventions. Use after the concept model is signed off.'
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# Phase 3 — Logical model

> **Not yet implemented.** The `erdbt` subcommand this phase calls does not
> exist yet — only `render`, `check`, `formats`, `clean`, `skills`, and `bump`
> ship today. `intake`, `ir check`, `ir diff`, `erd`, `preview`, `dbt-merge`,
> `conventions`, and `physical` do not.
>
> Run the phase manually: do the reasoning this skill describes, write the
> files it names — including the `erd/*.json` payload, by hand, in the
> shape the IR skill documents — and stop at the gate as usual. The artifacts
> are the deliverable whether or not a subcommand produced them.
>
> Do not fabricate output a missing subcommand would have produced, and do not
> report a step as run when it was not. Where a check like `erdbt ir check`
> cannot run, do its comparison by reading both files and say that is what you
> did.

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

## Where the artifacts live

Every path this skill names is written `erdbt/...`, but `erdbt/` is the
**default**, not a fixed location. Resolve it once, at the start of the phase,
and use what you resolve for every read and write:

1. If `.vscode/hub-e/hub.json` exists and holds an `artifactsDir`, the directory
   is `<artifactsDir>/<spoke>/`, where `spoke` is that file's `spoke` value.
2. If it exists without an `artifactsDir`, the directory is
   `.vscode/hub-e/<spoke>/`.
3. Otherwise the directory is `erdbt/`.

An editor or wrapper that orchestrates these phases keeps that file; a project
run from a plain terminal has no such file and gets the default. Say which
directory you resolved the first time you write to it, so the human can see
where the model is landing.

Read the file to resolve the path — never assume which case applies, and never
write to two of them. If the file exists but does not parse, stop and say so
rather than falling back to the default; a project that configured a location
and then got `erdbt/` has its model split across two directories.

## The IR is the source of truth

Everything erdbt knows is versioned in git in the artifacts directory:
`requirements.yaml`, `concept.ir.yaml`, `logical.ir.yaml`, the rendered
`erd/*.json`, and `drift.md`. Nothing carries between phases except those
files.

- Each phase owns its file and never edits an earlier one. If phase 3 needs a
  concept change, that is a phase 2 revision and a new gate — not an edit.
- Never infer a fact a previous phase should have recorded. If it is not in the
  IR, it does not exist — go back a phase.
- Never hand-edit a rendered `erd/*.json` or a generated dbt file to express a
  modeling decision. That decision belongs in the IR, which then re-renders.
- Run `erdbt ir check` before any gate from phase 3 on. It cross-validates
  logical against concept, so an entity that never got signed off cannot slip
  through.
- `erdbt ir diff` is how a reviewer sees what changed. Keep the IR diffable:
  stable key order, one concept per entry.

## Do not invent

Model only what the requirements support. When something is genuinely
ambiguous, record it under `open_questions` in the IR and raise it at the gate
— an open question is a legitimate deliverable, a guess dressed as a fact is
not.

Dispatch `erdbt-dbt-engineer` for the project's conventions and `erdbt-modeler`
for the attribute and cardinality review in the same message; neither waits on
the other. `erdbt-reviewer` goes last, on the finished diff.

## Delegating to a role

This plugin ships specialist subagents. Use them for the reading-heavy parts of
a phase — they work in their own context and return findings, which keeps the
main conversation's context for the artifact and the gate.

| Role | Use it for |
|------|-----------|
| `erdbt-analyst` | Reading a requirements doc and extracting entities, measures, grain, and open questions |
| `erdbt-modeler` | Pressure-testing entities, grain, and cardinality against the requirements |
| `erdbt-dbt-engineer` | Reading the target project's manifest for conventions, structure, and merge collisions |
| `erdbt-tester` | Materializing against DuckDB, checking grain, keys, and physical drift |
| `erdbt-reviewer` | Reading the phase diff for what should block sign-off |
| `erdbt-planner` | Auditing where the run stands and what the next gate still needs |

Independent roles run in one message, in parallel. Conventions and requirements
review do not depend on each other; asking for both at once costs one wait.

**They advise, you write.** A role returns findings — you decide what to change,
you make the edit, and you present it at the gate. A subagent that reports "this
looks good" has not approved anything, and relaying it as approval is how a gate
gets crossed without a human. Say who found what, so the human can weigh it.

Skip the roles when the work is small. Delegating a two-entity model costs more
than reading it yourself.

## The gate

This phase ends at a human sign-off. The gate is normal git/PR review — there
is no approval UI, and you do not own the decision.

- Write your output, show the diff, and **stop**.
- Never begin the next phase in the same turn. The human moves the gate, not you.
- If the human asks for changes, revise in place and show the diff again.
- Never edit a downstream artifact to make an upstream change "fit". Fix the
  upstream file and re-render.
