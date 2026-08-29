//! The Claude Code plugin format: a plugin tree plus a marketplace manifest.

use std::collections::BTreeMap;
use std::fs;

use serde_json::json;
use serde_yaml::Mapping;

use crate::frontmatter;
use crate::sources::{Doc, Sources, CONTENT};

pub const BANNER: &str = "<!-- Generated from src/erdbt-core/ by erdbt render. Do not edit. -->";

/// The plugin calls `erdbt` subcommands, so the binary has to be on PATH
/// separately from the plugin install. Same text for every host.
pub const CLI_INSTALL: &str = "\n### The `erdbt` CLI\n\nThe skills shell out to an `erdbt` binary, which the plugin install does not\nbring with it. From a checkout of this repo sitting beside your project:\n\n```\ncargo install --path ../erdbt-harness/src/erdbt-core\n```\n\nOnce the crate is on crates.io, `cargo install erdbt-core` will do the same\nthing without the checkout. Either way the crate is `erdbt-core` and the binary\nit puts on your PATH is `erdbt` — `erdbt --version` should answer.\n";

/// Directories this format owns entirely.
pub const DIRS: &[&str] = &["plugins"];

/// Generated files outside any owned directory.
pub const LOOSE: &[&str] = &[".claude-plugin/marketplace.json"];

/// Agent frontmatter this renderer passes through, beyond name/description/tools.
///
/// `hooks`, `mcpServers`, and `permissionMode` are deliberately absent: plugin
/// agents are refused those for security, so emitting one would author a
/// setting that silently does nothing.
pub const AGENT_KEYS: &[&str] = &[
    "model",
    "color",
    "effort",
    "maxTurns",
    "disallowedTools",
    "skills",
    "memory",
    "background",
    "isolation",
];

pub fn plugin_json(src: &Sources) -> Result<String, String> {
    let p = &src.plugin;
    let manifest = json!({
        "name": p.name,
        "version": p.version,
        "description": p.description,
        "author": {"name": p.author.name, "email": p.author.email, "url": p.author.url},
        "homepage": p.homepage,
        "repository": p.repository,
        "license": p.license,
        "keywords": p.keywords,
    });
    Ok(serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n")
}

pub fn marketplace_json(src: &Sources) -> Result<String, String> {
    let p = &src.plugin;
    let author = json!({"name": p.author.name, "email": p.author.email, "url": p.author.url});
    let manifest = json!({
        "name": "datnguyex",
        "owner": author,
        "description": "Agent plugins by Dat Nguyen.",
        "metadata": {"pluginRoot": "./plugins"},
        "plugins": [{
            "name": p.name,
            "source": format!("./plugins/{}", p.name),
            "description": p.description,
            "version": p.version,
            "author": author,
            "homepage": p.homepage,
            "repository": p.repository,
            "license": p.license,
            "keywords": p.keywords,
            "category": "productivity",
        }],
    });
    Ok(serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n")
}

/// Render a plugin skill.
///
/// Kept to the fields of the Agent Skills spec (name, description,
/// allowed-tools) so the same file also survives a claude.ai upload or the
/// Skills API, both of which hard-error on any other key. Licensing is carried
/// by plugin.json and the marketplace entry, not by each skill.
pub fn skill(doc: &Doc) -> Result<String, String> {
    let mut meta = Mapping::new();
    meta.insert("name".into(), doc.slug.clone().into());
    meta.insert("description".into(), doc.description().into());
    if let Some(tools) = doc.tools() {
        meta.insert("allowed-tools".into(), tools.into());
    }
    frontmatter::render(&meta, &format!("{BANNER}\n\n{}", doc.body))
}

/// Render a plugin agent.
///
/// Unknown keys are ignored by the loader rather than rejected, but the same
/// discipline as skills applies: emit only what the agent schema defines, so a
/// stray authoring field never reads as a setting that does nothing. `hooks`,
/// `mcpServers`, and `permissionMode` are refused for plugin agents, so they
/// are not emitted even when authored.
pub fn agent(doc: &Doc) -> Result<String, String> {
    let mut meta = Mapping::new();
    meta.insert("name".into(), doc.slug.clone().into());
    meta.insert("description".into(), doc.description().into());
    for key in AGENT_KEYS {
        if let Some(value) = doc.meta.get(*key) {
            meta.insert((*key).into(), value.clone());
        }
    }
    if let Some(tools) = doc.tools() {
        meta.insert("tools".into(), tools.into());
    }
    frontmatter::render(&meta, &format!("{BANNER}\n\n{}", doc.body))
}

/// Render a slash command. Commands carry description and argument-hint only.
pub fn command(doc: &Doc) -> Result<String, String> {
    let mut meta = Mapping::new();
    meta.insert("description".into(), doc.description().into());
    if let Some(hint) = doc.string("argument-hint").filter(|h| !h.trim().is_empty()) {
        meta.insert("argument-hint".into(), hint.into());
    }
    if let Some(tools) = doc.tools() {
        meta.insert("allowed-tools".into(), tools.into());
    }
    frontmatter::render(&meta, &format!("{BANNER}\n\n{}", doc.body))
}

pub fn readme(src: &Sources) -> String {
    let p = &src.plugin;
    let mut out = format!("{BANNER}\n\n# {}\n\n{}\n", p.name, p.description);
    if !src.commands.is_empty() {
        out.push_str("\n## Commands\n\n");
        for doc in &src.commands {
            out.push_str(&format!("- `/{}` — {}\n", doc.slug, doc.description()));
        }
    }
    if !src.agents.is_empty() {
        out.push_str("\n## Agents\n\n");
        for doc in &src.agents {
            out.push_str(&format!("- `{}` — {}\n", doc.slug, doc.description()));
        }
    }
    out.push_str(&format!(
        "\n## Install\n\n```\n/plugin marketplace add datnguye/erdbt-harness\n/plugin install {}@datnguyex\n```\n",
        p.name
    ));
    out.push_str(CLI_INSTALL);
    out
}

/// Hooks are authored as-is under src/hooks and copied verbatim.
pub fn hooks(
    src_root: &std::path::Path,
    root: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let dir = src_root.join(CONTENT).join("hooks");
    if !dir.exists() {
        return Ok(());
    }
    let mut paths: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .map(|entry| {
            entry
                .map(|e| e.path())
                .map_err(|e| format!("{}: {e}", dir.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|p| p.is_file());
    paths.sort();
    for path in paths {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        out.insert(format!("{root}/hooks/{name}"), text);
    }
    Ok(())
}

/// Commands and agents flatten to a single `<slug>.md`, so a sibling file has
/// nowhere to be written. Refusing it beats shipping a plugin that silently
/// omits a file the author put there.
fn reject_extras(doc: &Doc, kind: &str) -> Result<(), String> {
    match doc.extras.first() {
        Some(extra) => Err(format!(
            "{}: a {kind} ships as a single file, so this sibling would be dropped",
            extra.display()
        )),
        None => Ok(()),
    }
}

/// Return a mapping of repo-relative path -> file contents.
pub fn emit(src: &Sources, src_root: &std::path::Path) -> Result<BTreeMap<String, String>, String> {
    let root = format!("plugins/{}", src.plugin.name);
    let mut out = BTreeMap::new();
    hooks(src_root, &root, &mut out)?;
    out.insert(
        format!("{root}/.claude-plugin/plugin.json"),
        plugin_json(src)?,
    );
    let skills_dir = src_root.join(CONTENT).join("skills");
    for doc in &src.skills {
        out.insert(format!("{root}/skills/{}/SKILL.md", doc.slug), skill(doc)?);
        for extra in &doc.extras {
            let rel = extra
                .strip_prefix(skills_dir.join(&doc.slug))
                .map_err(|_| format!("{}: outside its skill directory", extra.display()))?
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let text =
                fs::read_to_string(extra).map_err(|e| format!("{}: {e}", extra.display()))?;
            out.insert(format!("{root}/skills/{}/{rel}", doc.slug), text);
        }
    }
    for doc in &src.commands {
        reject_extras(doc, "command")?;
        out.insert(format!("{root}/commands/{}.md", doc.slug), command(doc)?);
    }
    for doc in &src.agents {
        reject_extras(doc, "agent")?;
        out.insert(format!("{root}/agents/{}.md", doc.slug), agent(doc)?);
    }
    out.insert(format!("{root}/README.md"), readme(src));
    out.insert(LOOSE[0].to_string(), marketplace_json(src)?);
    Ok(out)
}
