<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# erdbt

Human-gated ERD-to-dbt harness. Requirements to concept to logical model to dbt, with a sign-off at every gate.

## Commands

- `/erdbt-concept` — Phase 2 — build the concept model: entities and relationships only.
- `/erdbt-generate` — Phase 4 — generate dbt SQL and YAML from the logical model and merge into the project.
- `/erdbt-intake` — Phase 1 — turn a business requirements doc into a structured requirements summary.
- `/erdbt-logical` — Phase 3 — expand the concept model into attributes, keys, and cardinality.
- `/erdbt-physical` — Phase 5 — run dbterd against the updated manifest and report drift from the concept model.

## Agents

- `erdbt-analyst` — Business analyst. Reads a raw requirements doc, PRD, or stakeholder notes and extracts entities, measures, grain, and open questions in the business's own words. Use in phase 1 to turn prose into something engineering can model.
- `erdbt-dbt-engineer` — dbt architect and implementation specialist. Reads the target project's manifest to report naming conventions, structural soundness (layering, staging purity, ref discipline, grain, tests), and merge collisions. Use in phases 3 and 4 to ground the model in the project as it actually is.
- `erdbt-modeler` — Data modeling specialist. Reviews a concept or logical IR for entity/relationship soundness, grain, and cardinality against the source requirements. Use during phases 2 and 3, before the gate, to pressure-test a model — never to write one.
- `erdbt-planner` — Run coordinator. Audits where a model stands across the five phases and reports what the next gate needs before it can be presented. Use to check run state, prepare a gate, or diagnose a stalled model — it tracks the work, it does not perform or approve it.
- `erdbt-reviewer` — Gate reviewer. Reads a phase's diff as a skeptical human reviewer would and reports what should block sign-off. Use before presenting any phase at its gate — it prepares the review, it does not grant it.
- `erdbt-tester` — Verification specialist. Materializes the model against DuckDB and checks generated dbt output compiles and holds its declared grain and keys. Use in phase 4 before merge, and in phase 5 to check the physical result against the concept model.

## Install

```
/plugin marketplace add datnguye/erdbt-harness
/plugin install erdbt@datnguyex
```

### The `erdbt` CLI

The skills shell out to an `erdbt` binary, which the plugin install does not
bring with it. From a checkout of this repo sitting beside your project:

```
cargo install --path ../erdbt-harness/src/erdbt-core
```

Once the crate is on crates.io, `cargo install erdbt-core` will do the same
thing without the checkout. Either way the crate is `erdbt-core` and the binary
it puts on your PATH is `erdbt` — `erdbt --version` should answer.
