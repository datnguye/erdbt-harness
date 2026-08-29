//! The GitHub Copilot CLI plugin format: a plugin tree plus a marketplace manifest.
//!
//! Copilot CLI's plugin layout is close enough to Claude Code's that the same
//! authored content ships to both: `plugin.json` at the plugin root, one
//! `SKILL.md` per skill directory, commands as flat markdown, and a
//! `marketplace.json` under `.github/plugin/`. What differs is spelled out
//! here — the agent file extension, the agent frontmatter keys, and the
//! plugin-root variable a hook command expands.

use std::collections::BTreeMap;

use serde_json::json;
use serde_yaml::Mapping;

use crate::formats::claude::{hooks, BANNER, CLI_INSTALL};
use crate::frontmatter;
use crate::sources::{Doc, Sources};

/// Directories this format owns entirely.
pub const DIRS: &[&str] = &["copilot-plugins"];

/// Generated files outside any owned directory.
pub const LOOSE: &[&str] = &[".github/plugin/marketplace.json"];

/// Agent frontmatter Copilot CLI defines, beyond name/description/tools.
///
/// Deliberately shorter than the Claude list: Copilot names its auto-delegation
/// switch `infer` and its effort field `reasoningEffort`, and defines nothing
/// for `color`, `maxTurns`, `background`, or `isolation`. Emitting a key it
/// does not read would author a setting that silently does nothing.
///
/// `model` is absent for a different reason: it is authored as a Claude alias
/// (`opus`), which names no Copilot model. Copilot falls back to the session
/// model rather than failing, so emitting it would be a line that reads like a
/// choice and is not one. Letting each role inherit the session model is the
/// same outcome, said honestly.
pub const AGENT_KEYS: &[&str] = &["infer", "reasoningEffort"];

/// Claude spells the plugin directory `${CLAUDE_PLUGIN_ROOT}`; Copilot spells
/// it `${PLUGIN_ROOT}`. A hook command is authored once, so the substitution
/// happens here rather than in two copies of hooks.json.
pub const CLAUDE_PLUGIN_ROOT: &str = "${CLAUDE_PLUGIN_ROOT}";
pub const PLUGIN_ROOT: &str = "${PLUGIN_ROOT}";

pub fn plugin_json(src: &Sources, has_hooks: bool) -> Result<String, String> {
    let p = &src.plugin;
    let mut manifest = json!({
        "name": p.name,
        "version": p.version,
        "description": p.description,
        "author": {"name": p.author.name, "email": p.author.email, "url": p.author.url},
        "homepage": p.homepage,
        "repository": p.repository,
        "license": p.license,
        "keywords": p.keywords,
        "agents": "agents/",
        "skills": "skills/",
        "commands": "commands/",
    });
    if has_hooks {
        manifest["hooks"] = json!("hooks/hooks.json");
    }
    Ok(serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n")
}

pub fn marketplace_json(src: &Sources) -> Result<String, String> {
    let p = &src.plugin;
    let author = json!({"name": p.author.name, "email": p.author.email, "url": p.author.url});
    let manifest = json!({
        "name": "datnguyex",
        "owner": {"name": p.author.name, "email": p.author.email},
        "metadata": {"description": "Agent plugins by Dat Nguyen."},
        "plugins": [{
            "name": p.name,
            "source": format!("./copilot-plugins/{}", p.name),
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

/// Render a plugin skill. Copilot reads the same three keys as the Agent Skills
/// spec, so this is the Claude rendering with no divergence to encode.
pub fn skill(doc: &Doc) -> Result<String, String> {
    let mut meta = Mapping::new();
    meta.insert("name".into(), doc.slug.clone().into());
    meta.insert("description".into(), doc.description().into());
    if let Some(tools) = doc.tools() {
        meta.insert("allowed-tools".into(), tools.into());
    }
    frontmatter::render(&meta, &format!("{BANNER}\n\n{}", doc.body))
}

/// Render a custom agent.
///
/// `tools` is a YAML array here, not the comma-separated string skills use:
/// Copilot documents the agent field as `string[]`, and a bare string would
/// read as one tool named "Read, Grep".
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
        let list: Vec<serde_yaml::Value> = tools
            .split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(Into::into)
            .collect();
        meta.insert("tools".into(), list.into());
    }
    frontmatter::render(&meta, &format!("{BANNER}\n\n{}", doc.body))
}

/// Render a command. Copilot reads description, argument-hint, and
/// allowed-tools from a flat `.md` file, exactly as Claude does.
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
        "\n## Install\n\n```\ncopilot plugin marketplace add datnguye/erdbt-harness\ncopilot plugin install {}@datnguyex\n```\n",
        p.name
    ));
    out.push_str(CLI_INSTALL);
    out
}

/// Commands and agents flatten to a single file, so a sibling has nowhere to go.
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
    let root = format!("copilot-plugins/{}", src.plugin.name);
    let mut out = BTreeMap::new();
    hooks(src_root, &root, &mut out)?;
    for text in out.values_mut() {
        *text = text.replace(CLAUDE_PLUGIN_ROOT, PLUGIN_ROOT);
    }
    let has_hooks = out.contains_key(&format!("{root}/hooks/hooks.json"));
    out.insert(format!("{root}/plugin.json"), plugin_json(src, has_hooks)?);
    let skills_dir = src_root.join(crate::sources::CONTENT).join("skills");
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
                std::fs::read_to_string(extra).map_err(|e| format!("{}: {e}", extra.display()))?;
            out.insert(format!("{root}/skills/{}/{rel}", doc.slug), text);
        }
    }
    for doc in &src.commands {
        reject_extras(doc, "command")?;
        out.insert(format!("{root}/commands/{}.md", doc.slug), command(doc)?);
    }
    for doc in &src.agents {
        reject_extras(doc, "agent")?;
        out.insert(format!("{root}/agents/{}.agent.md", doc.slug), agent(doc)?);
    }
    out.insert(format!("{root}/README.md"), readme(src));
    out.insert(LOOSE[0].to_string(), marketplace_json(src)?);
    Ok(out)
}
