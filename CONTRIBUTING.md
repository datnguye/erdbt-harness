# Contributing to erdbt-harness

## The one rule that surprises people

**`plugins/`, `copilot-plugins/`, `.claude-plugin/` and `.github/plugin/` are generated.** Edits there get reverted — CI re-renders and fails on the diff. Author in `src/erdbt-core/content/` instead:

```bash
cargo run -- render
cargo run -- check
```

One source, 2 hosts. A host difference belongs in its emitter (`src/erdbt-core/render/formats/`), never in the markdown.

## Getting set up

Rust (stable), nothing else. No Python — a hard constraint, not a preference.

```bash
git clone https://github.com/datnguye/erdbt-harness.git
cd erdbt-harness
cargo test
```

## Before you open a PR

CI runs exactly these:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- check
cargo run -- render && git diff --exit-code
```

The last one failing means you edited a generated file.

## Where things live

| Path | What it is |
|------|------------|
| `src/erdbt-core/content/skills/` | One skill per phase, plus `ir-schema` and `dbt-conventions` |
| `src/erdbt-core/content/commands/` | The 5 `/erdbt-*` commands |
| `src/erdbt-core/content/agents/` | The 6 advisory subagents |
| `src/erdbt-core/content/shared/` | Blocks included via `{{ name }}` — anything 2 roles must agree on |
| `src/erdbt-core/render/` | The renderer: sources in, formats out |
| `tests/erdbt-core/` | `unit/` mirrors `render/`; `integration/cli.rs` drives the binary |
| `.claude/skills/` | Dev-time context for this repo — not shipped |

## Authoring skills and agents

Read `.claude/skills/skill-authoring/SKILL.md` first. 2 constraints reviewers will ask about:

- **Agents advise; they never write or approve.** One that could sign off on its own work turns 5 gates into 0.
- **Anything 2 roles must agree on is a shared block or shipped skill**, never pasted into 2 prompts.

## Rust changes

Read `.claude/skills/rust-cli-conventions/SKILL.md` before adding a subcommand. Tests mirror the module tree: `render/output/tree.rs` → `tests/erdbt-core/unit/output/tree.rs`.

## Versioning

`src/erdbt-core/plugin.yaml` holds the one authored version; every manifest is generated from it. Never edit those by hand:

```bash
cargo run -- bump patch   # or minor, major, or 1.2.3
```

The crate version is separate and stays at its `0.0.0` placeholder — the release workflow stamps it from the tag.

## Getting help

🐛 [Issues](https://github.com/datnguye/erdbt-harness/issues) | 💬 [Discussions](https://github.com/datnguye/erdbt-harness/discussions) | ☕ [Buy me a coffee](https://www.buymeacoffee.com/datnguye)

[![buy me a coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg?logo=buy-me-a-coffee&logoColor=white&labelColor=ff813f&style=for-the-badge)](https://www.buymeacoffee.com/datnguye)
