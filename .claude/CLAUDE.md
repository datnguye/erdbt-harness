# erdbt-harness

A Claude Code **plugin** (not a custom agent) that drives an agentic, human-gated
workflow for building dbt data models from business requirements.

This file is DEV-TIME context for building *this repo*. It is not shipped to end
users — everything under `plugins/erdbt/` is.

## Why this exists

Teams do not lose a data model to bad SQL. They lose it at the seams: a
requirement nobody wrote down, an entity that appeared between reviews, a
cardinality someone assumed, a hand-tuned model a generator silently clobbered.
By the time a number is wrong in a report, the reasoning that produced it is
gone — it lived in a chat transcript, or in someone's head.

So this repo's product is **not** the generated dbt code. It is the record of
how that code was reasoned into being: the artifacts directory holds the
requirements with verbatim quotes, the concept model, the logical model, and
the drift report, all as code, all in git, all reviewable without opening a
`.sql` file. The dbt models are the by-product. The audit trail is the
deliverable.

Everything under **Hard constraints** follows from that:

- **Five gates, each a normal git/PR review.** A custom approval UI is a second
  place for decisions to hide. Teams already know how to review a diff.
- **Agents advise, they never write or approve.** A subagent that could sign off
  on its own work turns five gates into zero.
- **The IR is the only state between phases.** If it is not in the artifacts
  directory, it did not happen — roles share no memory and no conversation
  history.
- **ERDs are rendered, never authored.** A hand-edited diagram that disagrees
  with its model is worse than none, because it is what people actually look at.
- **Phase 4 never writes silently.** A generator that clobbers a hand-tuned
  model is one nobody trusts twice.

Status worth stating plainly: the skills and agents ship and describe every
phase in full, but the `erdbt` subcommands they call — `intake`, `ir check`,
`ir diff`, `erd`, `preview`, `dbt-merge`, `conventions`, `physical` — are not
implemented. The docs describe substantially more than the software does. That
is a known gap, not an oversight; the skills say so and refuse to fabricate
output a missing subcommand would have produced.

## The five phases

Each phase is gated by a human sign-off. The gate is **normal git/PR review**,
not a custom approval UI. The plugin's job is to produce clean, reviewable
diffs — not to gate them itself.

| # | Phase | Command | Produces |
|---|-------|---------|----------|
| 1 | Requirements intake | `/erdbt-intake` | Structured requirements summary file from a business requirements doc |
| 2 | Concept model | `/erdbt-concept` | `concept.ir.yaml` — entities + relationships **only**, no attributes — plus `erd/concept.json` |
| 3 | Logical model | `/erdbt-logical` | `logical.ir.yaml` + `erd/logical.json` — attributes, keys, cardinality, named to the target project's conventions |
| 4 | dbt generation + merge | `/erdbt-generate` | staging/intermediate/mart SQL + YAML, merged into the **actual** existing dbt project |
| 5 | Physical ERD | `/erdbt-physical` | `erd/physical.json` via dbterd, plus the committed `drift.md` |

Phase 2 must be reviewed and approved before phase 3 begins. Phase 4 is diffed
and reviewed, **never silently written**.

## Hard constraints

- **Rust** for any CLI logic. **Bash** for hooks and utility scripts.
  **No Python anywhere in this repo.**
- **DuckDB** is the local preview/sanity-check engine — materialize the IR
  against it before merge.
- **dbt v2 (Fusion)** is the target project format. Detect the manifest format
  at runtime — `dbt-fusion 2.0.0-preview.209` emits `target/manifest.json`
  (schema v12), **not** the Parquet manifest the original brief assumed. See
  `.claude/skills/dbt-fusion-manifest/SKILL.md`.
- The **IR is the single source of truth**, persisted as code:
  `concept.ir.yaml` and `logical.ir.yaml` are separate files, each owned by its
  phase, cross-validated by `erdbt ir check`. The schema ships as a plugin
  skill (`content/skills/ir-schema/`), not a dev-time note.
- **The artifacts directory is resolved, never hardcoded.** `erdbt/` is the
  default; a `.vscode/hub-e/hub.json` naming an `artifactsDir` moves it to
  `<artifactsDir>/<spoke>/`, and one without that key to
  `.vscode/hub-e/<spoke>/`. The rule lives in one shared block
  (`content/shared/artifacts-dir.md`) included by every phase skill — authored
  paths stay written as the default, and the skill resolves at runtime. The
  `spec` key — the brief phase 1 reads when none is passed — lives in the
  intake skill itself, not in this block. Agents never resolve it:
  `shared/advisory.md` forbids them writing there at all.
- **ERDs are rendered, never authored.** All three phases emit the
  `@datnguye/erd-flow` payload — dbterd's `json` target — to `<artifacts>/erd/*.json`,
  so one viewer renders any phase and a structural change lands in the diff.
- **No custom agent loop.** Claude Code's own loop is the orchestrator. Skills
  only tell Claude when to call which `erdbt-core` subcommand and how to
  interpret its output.
- **Agents advise; they never write or approve.** The six shipped subagents
  return findings. The main loop makes every edit and stops at the gate, and
  only the human signs off — an agent that could approve its own work would
  turn five gates into zero.
- **Anything two roles must agree on is a shipped skill**, not text duplicated
  into two agent prompts. That is why `ir-schema` and `dbt-conventions` live in
  `content/skills/` rather than `.claude/skills/` — agents preload them at
  runtime, so they have to exist on an installed user's machine.
- **Claude Code and GitHub Copilot CLI both ship**, rendered from the one set
  of authored sources by a registry (`emit::ALL`) with a module per format, so
  cursor/windsurf can be added later without touching the pipeline. A host
  difference is encoded in its emitter, never in the content: the authored
  markdown stays host-neutral. See
  `.claude/skills/rust-cli-conventions/SKILL.md`.

## Layout

```
.claude/                          # DEV-TIME config, for building this repo
  CLAUDE.md
  skills/
    rust-cli-conventions/SKILL.md
    dbt-fusion-manifest/SKILL.md
    ir-schema/SKILL.md            # dev-time copy; the shipped one is in content/
    skill-authoring/SKILL.md
src/erdbt-core/                   # ALL source: the renderer and what it renders
  Cargo.toml
  main.rs                         # CLI entry point (bin)
  plugin.yaml                     # manifest fields, incl. the one version
  render/                         # the renderer (lib: erdbt_core)
    lib.rs                        # re-exports; input -> formats -> output
    input/{sources,frontmatter}.rs
    formats/registry.rs           # Emitter { name, dirs, loose, emit }; ALL
    formats/{claude,copilot}.rs   # one module per output format
    output/{tree,bump}.rs         # write files; carry the version
  content/                        # everything the plugin ships
    shared/*.md                   # blocks included via {{ name }}
    skills/<slug>/SKILL.md        # one per phase, plus ir-schema and
                                  #   dbt-conventions, which agents preload
    commands/<slug>/COMMAND.md
    agents/<slug>/AGENT.md        # the six advisory roles
    hooks/                        # bash + json, copied verbatim
tests/                            # targets declared in Cargo.toml
  erdbt-core/
    common/mod.rs                 # shared fixtures (TempDir, foreign_project)
    unit/                         # mirrors render/ exactly, one file per module
      input/{frontmatter,sources}.rs
      formats/{registry,claude,copilot}.rs
      output/{tree,bump}.rs
    integration/cli.rs            # e2e, drives the built binary
```

Everything below is **generated** — never edit it, `cargo run -- render` writes
it and `cargo run -- check` fails if it drifts:

```
.claude-plugin/marketplace.json    # RUNTIME: what Claude Code users install
plugins/erdbt/                     # Claude Code's layout
  .claude-plugin/plugin.json
  README.md
  commands/<slug>.md  skills/<slug>/SKILL.md  agents/<slug>.md  hooks/
.github/plugin/marketplace.json    # RUNTIME: what Copilot CLI users install
copilot-plugins/erdbt/             # the same content, Copilot's layout
  plugin.json                      # component paths, not Claude's defaults
  README.md
  commands/<slug>.md  skills/<slug>/SKILL.md  hooks/
  agents/<slug>.agent.md           # Copilot derives the agent ID from this
```

Each emitter declares the `dirs` it owns outright and the `loose` files it
writes elsewhere. `tree` clears both before every render, so a renamed source
cannot leave an orphan behind, and `check` compares the two without knowing
which format produced what.

## Versioning

`src/erdbt-core/plugin.yaml` holds the one authored version. Everything that
reports a plugin version — `plugin.json`, the marketplace entry — is generated
from it, so there is nothing to keep in sync by hand.

Bump it with the CLI, never by editing a file:

```
erdbt bump patch          # or minor, major, or an explicit 1.2.3
```

That raises `plugin.yaml` and re-renders. CI fails if a generated manifest
disagrees with the authored version.

Claude Code has no native version-bump command — `claude plugin tag` reads the
version from `plugin.json` and creates a `{name}--v{version}` tag, validating
that `plugin.json` and the marketplace entry agree. `erdbt bump` is what makes
them agree, so the two compose:

```
erdbt bump minor
git commit -am "bump to $(grep '^version:' src/erdbt-core/plugin.yaml | cut -d' ' -f2)"
claude plugin tag plugins/erdbt --push
```

The **crate** version is separate and stays at the `0.0.0` placeholder: the
renderer is a build tool, not the artifact being versioned. `release.yml`
stamps it from the release tag and publishes the crate to crates.io — that is
all it does, and it runs only when you publish a GitHub release.

## Install UX

Matches `datnguye/lazy` exactly:

```
/plugin marketplace add datnguye/erdbt-harness
/plugin install erdbt@datnguyex
```

Copilot CLI installs the same repo through its own manifests:

```
copilot plugin marketplace add datnguye/erdbt-harness
copilot plugin install erdbt@datnguyex
```

The manifests on `main` carry the real plugin version, because `erdbt bump`
commits it there — so an install resolves the version you last bumped.
