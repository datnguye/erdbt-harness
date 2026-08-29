---
description: "Phase 1 of the erdbt harness: turn a business requirements doc into a structured requirements summary. Use when starting a new data model from a written brief, PRD, or stakeholder document."
---

# Phase 1 — Requirements intake

{{ unimplemented }}

Turn a business requirements document into a structured requirements summary at
`erdbt/requirements.yaml`. You are reading for **entities, facts, and grain** —
not yet for tables.

{{ artifacts-dir }}

## Steps

1. Find the source document. If the human named one, that is it. Otherwise, if
   `.vscode/hub-e/hub.json` holds a `spec`, read that path — a wrapper that
   configured the brief has already answered the question. Failing both, ask;
   do not guess at a file in the repo.
2. Extract into `erdbt/requirements.yaml`:
   - `subject_areas` — the business domains the doc covers
   - `entities` — every noun the business treats as a thing, with the verbatim
     sentence that evidences it
   - `measures` — what the business wants to count, sum, or rate
   - `grain_statements` — any sentence fixing "one row per ___"
   - `open_questions` — every ambiguity you hit
3. Run `erdbt intake --check erdbt/requirements.yaml` to validate the shape.
4. Show the human the summary and the open questions.

## Reading rules

Quote evidence, never paraphrase it into certainty. A requirements doc that
says "customers may have multiple accounts" is evidence of cardinality; a doc
that is silent on it is an open question, not a one-to-many.

Business language wins over your data-modeling instincts. If the business calls
it a "policy", the entity is `policy` — renaming it to `contract` because that
is the textbook term loses the reviewer's ability to check your work.

{{ no-invention }}

Dispatch `erdbt-analyst` on the source document first; it returns the
extraction and the handoff list of what the doc does not say. You write
`erdbt/requirements.yaml` from its findings.

{{ delegation }}

{{ gate }}
