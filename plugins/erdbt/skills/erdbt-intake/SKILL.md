---
name: erdbt-intake
description: 'Phase 1 of the erdbt harness: turn a business requirements doc into a structured requirements summary. Use when starting a new data model from a written brief, PRD, or stakeholder document.'
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# Phase 1 — Requirements intake

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

Turn a business requirements document into a structured requirements summary at
`erdbt/requirements.yaml`. You are reading for **entities, facts, and grain** —
not yet for tables.

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

## Steps

1. Find the source document. If the human named one, that is it. Otherwise, if
   `.vscode/hub-e/hub.json` holds a `spec`, read that path — a wrapper that
   configured the brief has already answered the question. Failing both, ask;
   do not guess at a file in the repo.
2. Extract into `erdbt/requirements.yaml`:
   - `subject_areas` — the business domains the doc covers
   - `entities` — every noun the business treats as a thing, with the verbatim
     sentence that evidences it
   - `measures` — what the business wants to count, sum, or rate
   - `grain_statements` — any sentence fixing "one row per ___"
   - `open_questions` — every ambiguity you hit
3. Run `erdbt intake --check erdbt/requirements.yaml` to validate the shape.
4. Show the human the summary and the open questions.

## Reading rules

Quote evidence, never paraphrase it into certainty. A requirements doc that
says "customers may have multiple accounts" is evidence of cardinality; a doc
that is silent on it is an open question, not a one-to-many.

Business language wins over your data-modeling instincts. If the business calls
it a "policy", the entity is `policy` — renaming it to `contract` because that
is the textbook term loses the reviewer's ability to check your work.

## Do not invent

Model only what the requirements support. When something is genuinely
ambiguous, record it under `open_questions` in the IR and raise it at the gate
— an open question is a legitimate deliverable, a guess dressed as a fact is
not.

Dispatch `erdbt-analyst` on the source document first; it returns the
extraction and the handoff list of what the doc does not say. You write
`erdbt/requirements.yaml` from its findings.

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
