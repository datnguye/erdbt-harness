---
description: "Phase 4 of the erdbt harness: generate staging, intermediate, and mart SQL plus YAML from the logical model and merge into the real dbt project. Use after the logical model is signed off."
---

# Phase 4 — dbt generation and merge

{{ unimplemented }}

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

{{ artifacts-dir }}

{{ ir }}

Before merging, dispatch `erdbt-dbt-engineer` for collisions and `erdbt-tester`
for the DuckDB build. A collision or a failed build stops the merge — report it
at the gate rather than working around it.

{{ delegation }}

{{ gate }}
