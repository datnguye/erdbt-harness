<div style="display: flex; align-items: center; justify-content: space-between;">
  <div>
    <h1 style="margin: 0;">erdbt-harness</h1>
    <p style="margin: 0; font-weight: bold;">From business requirements to dbt models, one human-gated phase at a time</p>
  </div>
</div>

5 phases from a requirements doc to real dbt models. The gate is git review, not a custom approval UI. Ships for Claude Code and GitHub Copilot CLI.

[![Crates.io](https://img.shields.io/crates/v/erdbt-core.svg?logo=rust&logoColor=white)](https://crates.io/crates/erdbt-core)
![rust-cli](https://img.shields.io/badge/CLI-Rust-DEA584?labelColor=14354C&logo=rust&logoColor=white)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-plugin-D97757?logo=anthropic&logoColor=white)](https://docs.claude.com/en/docs/claude-code)
[![Copilot CLI](https://img.shields.io/badge/Copilot%20CLI-plugin-24292e?logo=github&logoColor=white)](https://github.com/features/copilot)

[![erdbt-harness stars](https://img.shields.io/github/stars/datnguye/erdbt-harness.svg?logo=github&style=for-the-badge&label=Star%20this%20repo)](https://github.com/datnguye/erdbt-harness)

## Quick Start

```bash
/plugin marketplace add datnguye/erdbt-harness
/plugin install erdbt@datnguyex
```

```bash
copilot plugin marketplace add datnguye/erdbt-harness
copilot plugin install erdbt@datnguyex
```

Both trees render from one source, so they cannot drift.

## The 5 Phases

| # | Command | Produces |
|---|---------|----------|
| 1 | `/erdbt-intake` | A structured requirements summary from a business brief |
| 2 | `/erdbt-concept` | Entities and relationships only — no attributes, no types |
| 3 | `/erdbt-logical` | Attributes, keys, and cardinality, named to your conventions |
| 4 | `/erdbt-generate` | staging/intermediate/mart SQL and YAML, merged into the real project |
| 5 | `/erdbt-physical` | The ERD from what got built, plus a drift report |

Each phase stops at a sign-off and shows a diff. Phase 4 never writes silently.

## Installation

Neither plugin install brings the binary along:

```bash
cargo install erdbt-core
erdbt --version
```

Working on this repo instead? `cargo run -- <subcommand>` skips the install.

## What You End Up With

```
your-dbt-project/
  erdbt/                    # NEW — the modeling record, all of it as code
    requirements.yaml       #   phase 1  what the business asked for, quoted
    concept.ir.yaml         #   phase 2  entities + relationships, signed off
    logical.ir.yaml         #   phase 3  attributes, keys, cardinality
    erd/
      concept.json          #   phase 2  ─┐
      logical.json          #   phase 3   ├ one payload shape, 3 phases
      physical.json         #   phase 5  ─┘ (dbterd json, post-merge)
    drift.md                #   phase 5  what got built vs what was designed

  models/                   # MERGED — your existing dbt project, extended
    staging/                #   stg_*.sql + schema.yml
    intermediate/           #   int_*.sql + schema.yml
    marts/                  #   *.sql + schema.yml

  dbt_project.yml           # untouched
  target/manifest.json      # recompiled, so phase 5 can read it
```

**That directory is the reasoning** — why each entity exists, the requirement that justifies it, what is still open. Reviewable without opening a `.sql` file. The dbt code is the by-product; git already shows what changed there.

`erdbt/` is the default, not a fixed address. Each skill resolves the directory once and says which it picked.

## The Team

6 subagents do the reading-heavy work in their own context and hand back findings:

| Role | Does |
|------|------|
| `erdbt-analyst` | Turns a requirements doc into entities, measures, grain, open questions |
| `erdbt-modeler` | Pressure-tests entities, grain, and cardinality against the evidence |
| `erdbt-dbt-engineer` | Reports your real conventions, structural soundness, merge collisions |
| `erdbt-tester` | Builds against DuckDB; checks grain, keys, and physical drift |
| `erdbt-reviewer` | Reads the diff for what should block sign-off |
| `erdbt-planner` | Says where the run stands and what the next gate needs |

None writes a file, none approves anything. **[How they work as a team](docs/ways-of-working.md)**.

## Driven by Something Else

An editor or wrapper can launch the phases instead of you. It configures 2 things — where the artifacts go, and which document phase 1 reads. That is the whole contract; nothing about the 5 phases changes.

## Status

The skills and agents ship and describe every phase. The subcommands they call — `intake`, `ir check`, `ir diff`, `erd`, `preview`, `dbt-merge`, `conventions`, `physical` — are not implemented yet, so the phases run by hand for now. The skills say so rather than fabricate output.

## Developing

`plugins/` and `.claude-plugin/` are generated. Author in `src/erdbt-core/content/`:

```bash
cargo run -- render      # regenerate the plugin tree
cargo run -- check       # fail if the committed tree drifts
cargo run -- bump patch  # raise the version and re-render
```

## Contributing

Bugs, features, docs, typos — all welcome. See the **[Contributing Guide](CONTRIBUTING.md)**.

**Show your support:** ⭐ star the repo | 📢 share it | ☕ [buy me a coffee](https://www.buymeacoffee.com/datnguye)

[![buy me a coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg?logo=buy-me-a-coffee&logoColor=white&labelColor=ff813f&style=for-the-badge)](https://www.buymeacoffee.com/datnguye)

## Contributors

<a href="https://github.com/datnguye/erdbt-harness/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=datnguye/erdbt-harness" />
</a>

## Support

🐛 [Issues](https://github.com/datnguye/erdbt-harness/issues) | 💬 [Discussions](https://github.com/datnguye/erdbt-harness/discussions)

---

<div align="center">

**Made with ❤️ by [Dat Nguyen](https://github.com/datnguye)**

</div>

## License

MIT
