---
name: ir-schema
description: "The erdbt IR format — the single source of truth across all five phases. Read before writing any concept, logical, generation, or diff logic."
---

# The erdbt IR

The IR contract now **ships with the plugin**. It is authored at
`src/erdbt-core/content/skills/ir-schema/SKILL.md` and rendered into
`plugins/erdbt/skills/ir-schema/`, because the agents preload it at runtime —
a dev-time-only copy would not exist on an installed user's machine.

Read the authored file for the shape and the rules. Edit it there, never here
and never in `plugins/`.

## Why it moved

Anything two roles must agree on has to be a skill they can both preload.
`erdbt-modeler`, `erdbt-tester`, `erdbt-reviewer`, and `erdbt-planner` all
declare `skills: [ir-schema]`, so the file has to be part of what gets
installed.

Keeping a second copy here would be exactly the duplication
`.claude/principle_coding_pattern.md` warns about: when a fact could live in two
places, delete one rather than writing code to sync them.
