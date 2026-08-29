---
name: erdbt-generate
description: 'Phase 4 of the erdbt harness: generate staging, intermediate, and mart SQL plus YAML from the logical model and merge into the real dbt project. Use after the logical model is signed off.'
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# Phase 4 — dbt generation and merge

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

Generate dbt models from the logical IR and merge them into the **real** dbt
project. This is the phase that touches the human's actual repository, so it is
the phase with the least improvisation.

## Preview before you merge

Always materialize against DuckDB first:

```
erdbt preview erdbt/logical.ir.yaml
```

This builds the model graph locally and reports what fails. A DuckDB failure is
a modeling bug — fix the IR and re-run, never patch the generated SQL to make a
broken model compile.

## Merge

```
erdbt dbt-merge erdbt/logical.ir.yaml --project <path> --dry-run
```

Read the reported plan. Then re-run without `--dry-run` to write.

`dbt-merge` writes staging, intermediate, and mart layers plus their YAML. It
**never** overwrites a file a human has edited — those surface as conflicts for
you to show at the gate, not to resolve by clobbering. A hand-tuned model that
gets silently regenerated is how a team loses trust in a generator permanently.

## Never

- Never write into the dbt project with the Write or Edit tools *while
  `dbt-merge` exists* — it owns those paths, and a direct write bypasses the
  conflict detection. Until it ships, you are the merge: do by hand what it
  would do, and do the conflict check yourself. Read every path before you
  write it, and never overwrite a file the human has touched — show it at the
  gate as a conflict instead.
- Never run `dbt run` against a production target to "check" the merge. DuckDB
  preview is the check.
- Never commit. Show the diff and let the human review it.

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

Before merging, dispatch `erdbt-dbt-engineer` for collisions and `erdbt-tester`
for the DuckDB build. A collision or a failed build stops the merge — report it
at the gate rather than working around it.

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
