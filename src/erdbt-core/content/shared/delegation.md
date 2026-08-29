## Delegating to a role

This plugin ships specialist subagents. Use them for the reading-heavy parts of
a phase — they work in their own context and return findings, which keeps the
main conversation's context for the artifact and the gate.

| Role | Use it for |
|------|-----------|
| `erdbt-analyst` | Reading a requirements doc and extracting entities, measures, grain, and open questions |
| `erdbt-modeler` | Pressure-testing entities, grain, and cardinality against the requirements |
| `erdbt-dbt-engineer` | Reading the target project's manifest for conventions, structure, and merge collisions |
| `erdbt-tester` | Materializing against DuckDB, checking grain, keys, and physical drift |
| `erdbt-reviewer` | Reading the phase diff for what should block sign-off |
| `erdbt-planner` | Auditing where the run stands and what the next gate still needs |

Independent roles run in one message, in parallel. Conventions and requirements
review do not depend on each other; asking for both at once costs one wait.

**They advise, you write.** A role returns findings — you decide what to change,
you make the edit, and you present it at the gate. A subagent that reports "this
looks good" has not approved anything, and relaying it as approval is how a gate
gets crossed without a human. Say who found what, so the human can weigh it.

Skip the roles when the work is small. Delegating a two-entity model costs more
than reading it yourself.
