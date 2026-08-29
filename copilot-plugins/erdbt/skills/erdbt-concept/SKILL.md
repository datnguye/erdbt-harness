---
name: erdbt-concept
description: 'Phase 2 of the erdbt harness: build a conceptual model of entities and relationships only, with no attributes or types. Use after requirements intake is signed off.'
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# Phase 2 — Concept model

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

Once the entities and relationships are drafted, dispatch `erdbt-modeler` to
check them against the requirements, and `erdbt-reviewer` on the diff. Both at
once — they read different things.

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
