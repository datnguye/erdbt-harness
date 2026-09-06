# erdbt-harness

A plugin that walks a business requirements doc to real dbt models in five
human-gated phases. No custom approval UI — the gate is git review, which your
team already knows how to do. Ships for both Claude Code and GitHub Copilot CLI.

## Install

In Claude Code:

```
/plugin marketplace add datnguye/erdbt-harness
/plugin install erdbt@datnguyex
```

In GitHub Copilot CLI:

```
copilot plugin marketplace add datnguye/erdbt-harness
copilot plugin install erdbt@datnguyex
```

Same commands, same skills, same gates either way — the two plugin trees are
rendered from one source, so neither can quietly drift from the other.

### The `erdbt` CLI

The skills shell out to an `erdbt` binary, and neither plugin install brings it
along:

```
cargo install erdbt-core
```

The crate is `erdbt-core`; the binary it puts on your PATH is `erdbt`, so
`erdbt --version` is the check that it landed.

Be clear about what that binary is today, though: it renders this repo's
authored sources into the two plugin trees (`render`, `check`, `bump`, `clean`,
`skills`), and it expects to run from a checkout of this repo. The modelling
subcommands the skills call — `intake`, `ir check`, `ir diff`, `erd`,
`preview`, `dbt-merge`, `conventions`, `physical` — are not implemented yet.
The skills know this and refuse to invent output a missing subcommand would
have produced, so the phases stop rather than lie to you.

Working on this repo rather than using it? `cargo run -- <subcommand>` from the
checkout skips the install entirely.

## The five phases

| # | Command | Produces |
|---|---------|----------|
| 1 | `/erdbt-intake` | A structured requirements summary from a business brief |
| 2 | `/erdbt-concept` | Entities and relationships only — no attributes, no types |
| 3 | `/erdbt-logical` | Attributes, keys, and cardinality, named to your project's conventions |
| 4 | `/erdbt-generate` | staging/intermediate/mart SQL and YAML, merged into the real dbt project |
| 5 | `/erdbt-physical` | The ERD from what actually got built, plus a drift report |

Each phase stops at a sign-off and shows you a diff. Phase 4 never writes
silently — a generator that clobbers a hand-tuned model is a generator nobody
trusts twice.

## What you end up with

Everything runs against **your** dbt project. When the five phases are done,
that repo has two new kinds of thing in it — the reasoning, and the dbt code
it produced:

```
your-dbt-project/
  erdbt/                    # NEW — the modeling record, all of it as code
    requirements.yaml       #   phase 1  what the business asked for, quoted
    concept.ir.yaml         #   phase 2  entities + relationships, signed off
    logical.ir.yaml         #   phase 3  attributes, keys, cardinality
    erd/
      concept.json          #   phase 2  ─┐
      logical.json          #   phase 3   ├ one payload shape, three phases
      physical.json         #   phase 5  ─┘ (dbterd json, post-merge)
    drift.md                #   phase 5  what got built vs what was designed

  models/                   # MERGED — your existing dbt project, extended
    staging/                #   stg_*.sql + schema.yml
    intermediate/           #   int_*.sql + schema.yml
    marts/                  #   *.sql + schema.yml

  dbt_project.yml           # untouched
  target/manifest.json      # recompiled, so phase 5 can read it
```

`erdbt/` is the **default**, not a fixed address. A project with a
`.vscode/hub-e/hub.json` gets `<artifactsDir>/<spoke>/` when that file names an
`artifactsDir`, and `.vscode/hub-e/<spoke>/` when it does not — so an editor
orchestrating the phases reads the artifacts back from where it asked for them.
Every skill resolves this once, at the start of its phase, and says which
directory it picked. Run from a plain terminal with no such file and you get
`erdbt/`.

**That directory is the reasoning.** Why each entity exists, the verbatim
requirement that justifies it, which questions are still open, and where the
warehouse ended up disagreeing with the design. It is self-contained and reviewable on
its own — someone can read the concept model without opening a single `.sql`
file.

**`models/` is the dbt code**, merged into the layout and naming conventions
your project already uses. Not a parallel project, not a new directory tree —
phase 3 reads your manifest specifically so the generated models look like they
were written by your team.

Each IR file is owned by one phase and never edited by a later one; `erdbt ir
check` fails when a logical entity has no signed-off concept counterpart. The
three ERDs share the
[`@datnguye/erd-flow`](https://github.com/datnguye/erd-flow) payload shape, so
one viewer renders any phase — and because they are JSON rather than images, a
rewired relationship shows up in the PR diff instead of needing two pictures
compared by eye.

Nothing extra is recorded about the generated SQL. Your project's git history
already shows what erdbt wrote and what a human changed afterwards.

## Driven by something else

The phases are equally happy being launched by an editor or wrapper rather than
typed. Such a tool keeps a `.vscode/hub-e/hub.json`, and the skills read it for
exactly two things:

- **Where the artifacts go** — the resolution above.
- **Which document phase 1 reads.** With a `spec` recorded there,
  `/erdbt-intake` needs no argument and stops asking.

That is the whole contract. The orchestrator owns sessions, pre-flight checks
and gate bookkeeping; this plugin owns what a phase does. Nothing about the
five phases changes, and no subagent needs to know which case it is in — with
no such file, the artifacts land in `erdbt/`, the brief is the path you pass,
and the gate is a plain git review.

## The team

Six specialist subagents do the reading-heavy parts of each phase in their own
context, and hand back findings:

| Role | Does |
|------|------|
| `erdbt-analyst` | Turns a requirements doc into entities, measures, grain, and open questions |
| `erdbt-modeler` | Pressure-tests entities, grain, and cardinality against the evidence |
| `erdbt-dbt-engineer` | Reports your project's real conventions, structural soundness, and merge collisions |
| `erdbt-tester` | Builds against DuckDB; checks grain, keys, and physical drift |
| `erdbt-reviewer` | Reads the diff for what should block sign-off |
| `erdbt-planner` | Says where the run stands and what the next gate still needs |

None of them writes a file and none of them approves anything — they advise,
Claude edits, you sign off. [How they work as a team](docs/ways-of-working.md).

## Status

The skills and agents ship today and describe each phase in full. The `erdbt`
subcommands they call — `intake`, `ir check`, `ir diff`, `erd`, `preview`,
`dbt-merge`, `conventions`, `physical` — are not implemented yet, so the phases
run manually for now: the reasoning and the artifacts are the same, they are
just written by hand rather than generated. The skills say so, and they refuse
to fabricate output a missing subcommand would have produced.

## Developing

Everything under `plugins/` and `.claude-plugin/` is generated. Author in
`src/erdbt-core/content/` and render:

```
cargo run -- render      # regenerate the plugin tree
cargo run -- check       # fail if the committed tree drifts from its sources
cargo run -- bump patch  # raise the version and re-render
```

## License

MIT
