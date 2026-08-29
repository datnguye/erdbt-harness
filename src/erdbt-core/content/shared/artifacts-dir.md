## Where the artifacts live

Every path this skill names is written `erdbt/...`, but `erdbt/` is the
**default**, not a fixed location. Resolve it once, at the start of the phase,
and use what you resolve for every read and write:

1. If `.vscode/hub-e/hub.json` exists and holds an `artifactsDir`, the directory
   is `<artifactsDir>/<spoke>/`, where `spoke` is that file's `spoke` value.
2. If it exists without an `artifactsDir`, the directory is
   `.vscode/hub-e/<spoke>/`.
3. Otherwise the directory is `erdbt/`.

An editor or wrapper that orchestrates these phases keeps that file; a project
run from a plain terminal has no such file and gets the default. Say which
directory you resolved the first time you write to it, so the human can see
where the model is landing.

Read the file to resolve the path — never assume which case applies, and never
write to two of them. If the file exists but does not parse, stop and say so
rather than falling back to the default; a project that configured a location
and then got `erdbt/` has its model split across two directories.
