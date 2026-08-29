> **Not yet implemented.** The `erdbt` subcommand this phase calls does not
> exist yet — only `render`, `check`, `formats`, `clean`, `skills`, and `bump`
> ship today. `intake`, `ir check`, `ir diff`, `erd`, `preview`, `dbt-merge`,
> `conventions`, and `physical` do not.
>
> Run the phase manually: do the reasoning this skill describes, write the
> files it names — including the `erd/*.json` payload, by hand, in the
> shape the IR skill documents — and stop at the gate as usual. The artifacts
> are the deliverable whether or not a subcommand produced them.
>
> Do not fabricate output a missing subcommand would have produced, and do not
> report a step as run when it was not. Where a check like `erdbt ir check`
> cannot run, do its comparison by reading both files and say that is what you
> did.
