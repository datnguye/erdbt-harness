## The IR is the source of truth

Everything erdbt knows is versioned in git in the artifacts directory:
`requirements.yaml`, `concept.ir.yaml`, `logical.ir.yaml`, the rendered
`erd/*.json`, and `drift.md`. Nothing carries between phases except those
files.

- Each phase owns its file and never edits an earlier one. If phase 3 needs a
  concept change, that is a phase 2 revision and a new gate — not an edit.
- Never infer a fact a previous phase should have recorded. If it is not in the
  IR, it does not exist — go back a phase.
- Never hand-edit a rendered `erd/*.json` or a generated dbt file to express a
  modeling decision. That decision belongs in the IR, which then re-renders.
- Run `erdbt ir check` before any gate from phase 3 on. It cross-validates
  logical against concept, so an entity that never got signed off cannot slip
  through.
- `erdbt ir diff` is how a reviewer sees what changed. Keep the IR diffable:
  stable key order, one concept per entry.
