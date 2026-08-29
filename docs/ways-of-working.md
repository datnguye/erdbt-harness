# Ways of working

How the erdbt roles work as a team: who does what, who hands to whom, and who
is not allowed to decide anything.

## Table of contents

- [Ways of working](#ways-of-working)
  - [Table of contents](#table-of-contents)
  - [The shape of the team](#the-shape-of-the-team)
  - [Who does what](#who-does-what)
  - [The workflow](#the-workflow)
  - [What gets persisted](#what-gets-persisted)
  - [Shared knowledge](#shared-knowledge)
  - [Handoffs](#handoffs)
  - [Rules the whole team obeys](#rules-the-whole-team-obeys)
  - [When something else is orchestrating](#when-something-else-is-orchestrating)
  - [When not to use the team](#when-not-to-use-the-team)

## The shape of the team

There is no manager agent. Claude Code's own loop is the orchestrator — it holds
the artifact, makes every edit, and stops at each gate. The roles are
**advisors**: each reads something in its own context window and returns
findings.

```
              human  ── owns every gate, the only one who approves
                │
      Claude Code loop  ── writes the IR, shows the diff, stops
                │
   ┌────────┬───┴────┬──────────┬──────────┬─────────┐
analyst  modeler  engineer   tester   reviewer   planner
   │        │        │          │         │         │
   └────────┴────────┴──── findings ──────┴─────────┘
```

Nothing points back up through a gate. That is the point: a subagent that could
approve its own work would turn five human gates into zero.

## Who does what

| Role | Discipline | Reads | Returns | Writes |
|------|-----------|-------|---------|--------|
| `erdbt-analyst` | Business analysis | The requirements doc, PRD, stakeholder notes | Entities, measures, grain, open questions — each with a verbatim quote | Nothing |
| `erdbt-modeler` | Data modeling | The IR plus the requirements it claims to come from | Where model and evidence disagree, ranked by cost-to-fix-later | Nothing |
| `erdbt-dbt-engineer` | dbt architecture and implementation | The target project's `manifest.json` | Observed conventions with counts, structural findings, merge collisions, what the project does not say | Nothing |
| `erdbt-tester` | Verification | The generated models, DuckDB, the physical ERD | What built, what failed with the error, grain and key violations, phase-5 drift | Nothing |
| `erdbt-reviewer` | Gate review | The phase diff | Two lists — blocking and worth-noting — plus the thing it could not verify | Nothing |
| `erdbt-planner` | Coordination | The repo state and git history | Current phase, pending gate, what that gate still needs, ranked next actions | Nothing |

The `Writes` column is the same for every role on purpose.

## The workflow

Requirements enter as prose and leave as a merged dbt project. Each phase ends
at a human gate — a normal git or PR review, not an approval UI.

**Phase 1 — intake.** The analyst reads the source document and extracts
entities, measures, grain statements, and open questions, quoting evidence
verbatim rather than paraphrasing. The main loop writes
the requirements file from those findings. The analyst's handoff list — what
the document does *not* say — is what stops phase 2 from guessing. **Gate.**

**Phase 2 — concept.** The loop drafts entities and relationships only, no
attributes. Modeler and reviewer run in parallel: the modeler checks every
entity against its evidence and every cardinality against what the requirements
actually fix, the reviewer reads the diff for scope creep. **Gate.**

**Phase 3 — logical.** The dbt engineer reports the target project's naming
conventions with counts, and judges its structure — layer boundaries, staging
purity, `ref` discipline, stated grain, key tests — while the modeler reviews
attributes and resolves cardinality. Neither waits on the other. The loop
writes the attributes, following the conventions observed rather than its own
preference. The reviewer reads the finished diff. **Gate.**

**Phase 4 — generate and merge.** The engineer reports merge collisions and the
tester materializes against DuckDB, in parallel. Either a collision or a failed
build stops the merge; it goes to the gate as a finding, never worked around.
**Gate.**

**Phase 5 — physical.** The loop renders `erd/physical.json` from the
compiled manifest via dbterd. The tester structurally diffs it against the
concept and logical IR, and the drift report is committed as `drift.md`
rather than left in chat. The planner closes the run by reporting what the five
phases left outstanding. **Gate.**

## What gets persisted

Every phase leaves a committed artifact. Nothing that matters lives only in a
chat transcript.

```
<artifacts>/
  requirements.yaml     # phase 1
  concept.ir.yaml       # phase 2 — entities and relationships
  logical.ir.yaml       # phase 3 — attributes, keys, resolved cardinality
  erd/
    concept.json        # rendered from concept.ir.yaml
    logical.json        # rendered from logical.ir.yaml
    physical.json       # dbterd's json target, after the merge
  drift.md              # phase 5 — physical vs concept
```

`<artifacts>` is `erdbt/` unless the project says otherwise: a
`.vscode/hub-e/hub.json` naming an `artifactsDir` moves them to
`<artifactsDir>/<spoke>/`, and one without that key to
`.vscode/hub-e/<spoke>/`. Each phase resolves it once and says where it landed,
because a run split across two directories is worse than one in the wrong
place.

The generated dbt SQL and YAML land in the **target project**, not here — git
is the record of what was written, and the project's own history shows what a
human changed afterward.

**Concept and logical are separate files.** Each phase owns one and never edits
an earlier one; a concept change during phase 3 is a phase 2 revision and a new
gate. The cost of splitting them is that the two can disagree, so
`erdbt ir check` cross-validates: every logical entity must trace to a concept
entity, or be declared under `added_in_logical` with the many-to-many it
resolves. What used to be a reviewer noticing is now a check that fails.

**All three ERDs share one payload shape** — the
[`@datnguye/erd-flow`](https://github.com/datnguye/erd-flow) contract, which is
dbterd's `json` target. One viewer renders any phase, and because the payload is
JSON rather than an image, a rewired relationship shows up in the PR diff
instead of requiring someone to compare two pictures.

The ERD files are **generated, never authored**. A hand-edited diagram that
disagrees with its IR is worse than no diagram, because it is the artifact
people actually look at.

## Shared knowledge

Roles do not share memory or conversation history, so anything two of them must
agree on is a **skill**, preloaded into whichever roles need it. The full text
is injected at startup, not just the description.

| Skill | Preloaded into | Holds |
|-------|---------------|-------|
| `ir-schema` | modeler, tester, reviewer, planner | The IR shape, what `phase` gates, why `unknown` is a real cardinality, why evidence is verbatim |
| `dbt-conventions` | dbt engineer, tester | How to read a Fusion manifest, how to report conventions with counts, and what sound project structure looks like |

These ship with the plugin. There is no official dbt Labs skill to defer to —
the only dbt entry in Anthropic's marketplace is a third-party MCP-backed CLI —
so the dbt knowledge this team relies on is carried in `dbt-conventions` and
versioned with everything else.

A role that needs knowledge another role already has is a sign the knowledge
belongs in a skill. Duplicating it into two agent prompts is how the two
quietly start disagreeing.

## Handoffs

A handoff carries **evidence, not conclusions** — a quote, a file and line, a
manifest field with a count. A finding the next person cannot check is one they
have to redo.

- **Analyst → modeler**: verbatim requirements. The modeler checks the model
  against those quotes, so a paraphrase upstream becomes an unfalsifiable claim
  downstream.
- **Engineer → loop**: conventions with counts. "17 of 20 use `_key`" is
  actionable; "the project uses `_key`" hides that it is not settled.
- **Modeler / engineer / tester → reviewer**: the reviewer reads the diff, not
  the other roles' reports. It is meant to be the second opinion, and it stops
  being one if it inherits the first.
- **Everyone → human**: attributed. Say which role found what, so the human can
  weigh a DuckDB failure differently from a naming preference.

Anything a phase discovers that the next phase needs goes in the IR. If it is
not in the artifacts directory, it did not happen — the roles share no memory and no
conversation history.

## Rules the whole team obeys

**Advise, never decide.** No role writes the IR, the requirements file, or
anything in the target dbt project. They report; the main loop edits.

**No role approves.** Not even the reviewer, whose entire job is review. A
subagent saying "looks good" has approved nothing, and relaying that as approval
is how a gate gets crossed without a human.

**Evidence travels with the finding.** A file and line, a quote, a manifest
field, an error message.

**Say what you could not check.** An admitted gap costs the caller far less than
a confident guess.

**Parallel where independent.** Conventions and requirements review read
different files; dispatching both in one message costs one wait instead of two.

**Skip the team when the work is small.** Delegating a two-entity model costs
more than reading it yourself.

## When something else is orchestrating

The five phases can be driven by an editor or wrapper rather than by hand: a
worktree per session, pre-flight checks before a phase starts, a terminal per
command, and a recorded sign-off per gate. Such a tool keeps a
`.vscode/hub-e/hub.json`, and the phases read it for two things — where the
artifacts go, and which document phase 1 reads.

It changes nothing above. The orchestrator schedules phases and writes down
that a human approved one; it does not review, does not decide, and is not a
seventh role. The team still advises, the loop still writes, and the person at
the diff is still the only one who approves — the tool records that approval
rather than granting it.

The skills resolve both settings themselves, so no role needs to know which
case it is in.

## When not to use the team

- The model is small enough to read in one sitting.
- The phase is mid-flight and nothing is drafted yet — there is nothing to
  review.
- You need a decision. That is the human's, at the gate, every time.
