---
name: skill-authoring
description: "How to author erdbt skills and commands in src/ — frontmatter fields, shared blocks, and what belongs in a skill versus a command. Read before editing anything under src/skills or src/commands."
---

# Authoring erdbt content

Everything under `plugins/` is **generated**. Never edit it — edit `src/erdbt-core/` and
run `erdbt render`. The banner at the top of each generated file says so, and
`erdbt check` fails in CI when someone forgets.

## Layout

See the tree in `.claude/CLAUDE.md`. Authored content lives in
`src/erdbt-core/content/`; the renderer sits beside it in `render/`.

## Frontmatter

Skills carry `name`, `description`, and optional `allowed-tools`.
Nothing else — the Agent Skills spec hard-errors on unknown keys, and the same
file has to survive a claude.ai upload.

Commands carry `description` and optional `argument-hint`.

The `description` is load-bearing: it is what Claude reads to decide whether the
skill applies. Say what the skill does *and when to use it*. "Phase 3 of the
erdbt harness: ... Use after the concept model is signed off" beats "Logical
modeling helper."

## Shared blocks

A line containing only `{{ name }}` is replaced by `src/shared/name.md`. Use it
for text that must be identical across phases — the gate protocol, the IR
contract. Divergent copies of the gate rules is how a phase quietly stops
stopping.

## Skill or command?

The **skill** holds the instructions — how to do the phase, what to refuse, what
the gate means. The **command** is the entry point: one line of context, the
`$ARGUMENTS`, and a pointer to the skill. When a command starts explaining how
to model data, that text belongs in the skill.

## Tone

These skills are read by a model mid-task, not by a student. Prefer imperatives
and explicit refusals ("Never write into the dbt project with Edit") over
description. Where a rule is counterintuitive, give the one-line reason — a rule
whose purpose is visible survives paraphrase.
