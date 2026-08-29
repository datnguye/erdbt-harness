# Coding principles

Decisions specific to this repo that the code alone does not explain. General
Rust style lives in `.claude/skills/rust-cli-conventions/SKILL.md`; YAGNI and
the ladder come from the lazy plugin.

## Rendering is pure; only `output::tree` touches disk

`input` reads, `formats` computes, `output` writes. An emitter takes `&Sources`
and returns `{path: contents}` — it never learns where the output lands.

That one seam is why the rest is cheap: `check` is `render` plus a comparison,
so it cannot drift from `render` — it *is* `render`. Every format test asserts
on a map in memory, with no temp dirs to clean up.

## Generated output is owned, never merged

Everything under `plugins/` and `.claude-plugin/` is generated. The generator
clears the directories it owns before writing, so a renamed source cannot leave
an orphan behind.

`owned_dirs` derives what to clear *from the render itself*, not from a
hardcoded list, so a partial render (`--format claude`) can never wipe output it
did not produce.

## One source of truth, always

The IR is the only state between phases. `plugin.yaml` is the only version. The
render is the only writer of `plugins/`.

When a fact could live in two places, delete one rather than writing code to
sync them. `Cargo.toml` stays at `0.0.0` not because the crate version is
unknowable, but because a second version is a second thing that can be wrong.

## Deterministic output is a correctness property

A re-render must be byte-identical, or `erdbt check` reports drift that is not
there and CI fails at random. Once a build tool cries wolf, people stop reading
it. Hence `BTreeMap`, sorted directory listings, preserved frontmatter order.

## Render to the spec, not to the source

Authored frontmatter may hold anything; an emitter picks only what its target
format permits. A skill emits `name`, `description`, `allowed-tools` — no
`license`, because the Skills API and a claude.ai upload both hard-error on
unknown keys.

## Optional inputs are absent, not empty

A missing `skills/`, `commands/`, `shared/`, or `hooks/` directory yields no
documents rather than an error. Only `plugin.yaml` is required. That is what
lets the tool run against a half-built tree — the state it is in most often
while someone is authoring.
