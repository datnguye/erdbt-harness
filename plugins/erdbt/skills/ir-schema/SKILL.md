---
name: ir-schema
description: The erdbt IR format — concept.ir.yaml, logical.ir.yaml, and the erd-flow payloads rendered from them. Read before writing or reviewing any model, at any phase.
---

<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->

# The erdbt IR

Everything erdbt knows lives in one artifacts directory, as code, in git:

```
<artifacts>/
  requirements.yaml     # phase 1
  concept.ir.yaml       # phase 2 — entities and relationships only
  logical.ir.yaml       # phase 3 — attributes, keys, resolved cardinality
  erd/
    concept.json        # rendered from concept.ir.yaml
    logical.json        # rendered from logical.ir.yaml
    physical.json       # dbterd's json target, after the merge
  drift.md              # phase 5 — physical vs concept
```

Each phase owns its file. A phase never edits an earlier phase's artifact — if
phase 3 needs a concept change, that is a phase 2 revision and a new gate.

`<artifacts>` is `erdbt/` by default. A project whose `.vscode/hub-e/hub.json`
names an `artifactsDir` puts them under `<artifactsDir>/<spoke>/`, and one with
that file but no such key uses `.vscode/hub-e/<spoke>/`. Every path below is
written as the default; resolve the real one before you read or write. The rest
of this schema is unaffected — only the prefix moves.

## concept.ir.yaml

```yaml
version: 1
phase: concept
source: erdbt/requirements.yaml

entities:
  - name: customer
    definition: A party that holds one or more accounts.
    evidence: "Customers may hold multiple accounts."   # verbatim from source

relationships:
  - from: customer
    to: account
    verb: holds
    cardinality: one-to-many    # one-to-one | one-to-many | many-to-many | unknown

open_questions:
  - entity: account
    question: Is an account ever shared by two customers?
```

No attributes. No types. No keys. No column names.

## logical.ir.yaml

```yaml
version: 1
phase: logical
concept: erdbt/concept.ir.yaml     # the file this expands

entities:
  - name: customer                 # must exist in concept.ir.yaml
    primary_key: customer_key
    attributes:
      - name: customer_key
        type: varchar(32)
        description: Surrogate key.
        nullable: false
        source: stg_crm__customers.id

relationships:
  - from: customer
    to: account
    cardinality: one-to-many       # every `unknown` resolved
    foreign_key: account.customer_key

added_in_logical: []               # entities with no concept counterpart
```

## The two files must agree

Splitting concept from logical buys each phase a clean, reviewable file, and
costs the guarantee the single file used to give for free: that no entity
quietly appeared between gates. `erdbt ir check` is what buys it back.

Every entity in `logical.ir.yaml` must trace to one in `concept.ir.yaml`. An
entity with no counterpart is either a skipped gate or a genuine discovery —
and a discovery has to say so out loud:

```yaml
added_in_logical:
  - name: customer_account
    reason: Resolves the customer-to-account many-to-many.
    concept_relationship: customer holds account
```

An associative entity resolving a many-to-many is the one routine reason to
add. Anything else belongs in a phase 2 revision.

Run `erdbt ir check` before presenting phase 3 at its gate. It fails on an
untraced entity, on an unresolved `unknown`, and on a relationship whose
endpoints no longer exist.

`erdbt ir check erdbt/logical.ir.yaml` finds the concept file through the
`concept:` field rather than by guessing at a path — which is why that field is
required, and why the IR lint hook can pass a single edited file and still get
a cross-file check.

## Rules

**`phase` gates the writer.** A phase-3 tool refusing to run against a file
whose `phase` is not `concept` is correct — that is the gate enforcing itself
where a skill cannot skip it.

**`unknown` is a real cardinality value.** Phase 2 emits it whenever the
requirements are silent. Phase 3 must resolve every one; phase 4 must refuse to
generate while any remain. Defaulting `unknown` to `one-to-many` silently
invents a business rule.

**`evidence` is verbatim.** It is what makes the concept model reviewable — a
human checks the entity against the quote without reopening the source doc.
Never paraphrase into it.

**Keep it diffable.** Stable key order, one concept per list entry, no inline
flow mappings. The diff is how a reviewer sees a phase's work, and a reordered
file renders it useless.

## The ERD is rendered, never authored

`erdbt/erd/*.json` are **generated** from the IR — the payload shape
`@datnguye/erd-flow` renders, which is dbterd's `json` target:

```json
{
  "nodes": [{"id": "customer", "name": "customer", "columns": []}],
  "edges": [{"id": "customer-account", "from_id": "account",
             "to_id": "customer", "cardinality": "one-to-many"}],
  "metadata": {"generated_at": "...", "phase": "concept"}
}
```

An edge carries `from_id` as the **FK/child** side and `to_id` as the
referenced **parent**. Concept nodes carry an empty `columns` array; logical
nodes carry the real columns with `is_primary_key` / `is_foreign_key` set.

Never hand-edit an ERD payload. Fix the IR and re-render — a hand-edited
diagram that disagrees with the model is worse than no diagram, because it is
the artifact people actually look at.
