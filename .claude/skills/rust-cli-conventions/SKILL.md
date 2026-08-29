---
name: rust-cli-conventions
description: "How erdbt-core is structured: module layout, error handling, and the render/check/formats/clean/skills/bump command contract. Read before adding a subcommand."
---

# erdbt-core conventions

The crate is a renderer, not a framework. It reads `src/erdbt-core/`, produces a
`BTreeMap<path, contents>`, and writes it. Everything else is plumbing.

## Modules

The renderer is a **lib** (`erdbt_core`, rooted at `render/lib.rs`); `main.rs`
is a thin **bin** over it. The split exists so `tests/` can import the modules —
an integration test only sees a crate's public API, so a pure binary would be
untestable from outside.

The pipeline reads left to right: **input → formats → output**, one directory
each. `lib.rs` re-exports the leaves, so call sites say `sources::load`, not
`input::sources::load`.

| Module | Owns |
|--------|------|
| `input::frontmatter` | Splitting and rebuilding YAML frontmatter |
| `input::sources` | Loading `plugin.yaml`, content, shared blocks; `{{ }}` expansion |
| `formats::<name>` | One output format: `emit`, plus the `DIRS`/`LOOSE` paths it owns |
| `formats::registry` | Selects emitters, merges their output, rejects clashes |
| `output::tree` | Writing, cleaning, and drift-checking that map |
| `output::bump` | Raising the authored version |
| `main` | Argument parsing and exit codes |

## Adding an output format

`claude` and `copilot` ship today. Adding a third costs two edits.

1. Add `render/formats/<name>.rs` exposing `emit(&Sources, &Path)` plus `DIRS`
   and `LOOSE` — the paths that format owns.
2. Declare it in `formats/mod.rs` and add an `Emitter` entry to `emit::ALL`.

Nothing else changes. `tree` clears and drift-checks whatever the registry
reports, `--format` and `erdbt formats` pick it up for free, and two emitters
claiming one path is a build error rather than last-writer-wins.

The authored content in `content/` is format-agnostic on purpose: `{{ }}`
expansion exists so a skill that references a shared block still flattens into
a single standalone file for tools that have no plugin concept.

The seam that matters is `emit` → `tree`: rendering is pure and returns a map,
writing is the only thing that touches disk. Keep it that way — `check` is
implementable in three lines precisely because rendering has no side effects.

## Rules

- `Result<T, String>` throughout. This is a build tool whose errors are read by
  one human at a terminal; an error enum with thirty variants buys nothing over
  a good message.
- Only `main` prints or sets an exit code. Library modules return.
- `BTreeMap`, never `HashMap`, for rendered output — deterministic order means
  a re-render produces byte-identical files, which is what makes `check`
  trustworthy in CI.
- No `unwrap` on anything derived from user input or file contents. `unwrap` on
  a path component you just constructed is fine.

## Tests

`tests/` sits at the repo root and mirrors `src/`. Cargo only auto-discovers a
package's own `tests/`, so each target is declared by path in `Cargo.toml` —
adding a test file means adding a `[[test]]` entry.

- `tests/erdbt-core/unit/**` — **mirrors `render/` exactly**: a module at
  `render/input/sources.rs` is tested at `unit/input/sources.rs`. Adding a
  module means adding its mirror file and a `[[test]]` entry.
- `tests/erdbt-core/integration/cli.rs` — e2e, runs the built binary via
  `CARGO_BIN_EXE_erdbt`
- `tests/erdbt-core/common/mod.rs` — fixtures, included with `#[path]`

Anything a test needs is `pub`. `TempDir` cleans up on drop, so a failing
assertion never leaves state behind to poison the next run.

## Adding a subcommand

Add the variant to `Command`, add its arm to `run`, and stop. If the arm needs
more than a dozen lines, the logic belongs in `build` or `emit` and the arm
calls it.

## Not in this crate

Any Python. The whole point of porting lazy's renderer to Rust was to keep this
repo single-language; a helper script in Python undoes that for the sake of
saving twenty lines.
