//! Load the authored content: plugin.yaml, skills, commands, agents, and shared blocks.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use crate::frontmatter;

/// Everything the plugin ships lives under one directory — skills, commands,
/// agents, shared blocks, and hooks — so authored content stays visibly
/// separate from the manifest and the Rust that renders it.
pub const CONTENT: &str = "content";

/// One authored markdown document plus its parsed frontmatter.
#[derive(Debug)]
pub struct Doc {
    pub slug: String,
    pub meta: Mapping,
    pub body: String,
    pub extras: Vec<PathBuf>,
}

impl Doc {
    pub fn description(&self) -> String {
        self.meta
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string()
    }

    /// Declared tools as a comma-separated string, if any.
    pub fn tools(&self) -> Option<String> {
        let raw = self
            .meta
            .get("allowed-tools")
            .or_else(|| self.meta.get("tools"))?;
        match raw {
            Value::String(s) => Some(s.clone()),
            Value::Sequence(items) => Some(
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            _ => None,
        }
    }

    pub fn string(&self, key: &str) -> Option<String> {
        self.meta.get(key)?.as_str().map(str::to_string)
    }
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Author,
    pub homepage: String,
    pub repository: String,
    pub license: String,
    pub keywords: Vec<String>,
}

/// Everything authored under src/.
#[derive(Debug)]
pub struct Sources {
    pub plugin: Plugin,
    pub skills: Vec<Doc>,
    pub commands: Vec<Doc>,
    pub agents: Vec<Doc>,
}

/// Replace every `{{ name }}` marker with the shared block it names.
///
/// Authoring stays DRY without a cross-file reference that would dangle once a
/// skill is flattened into a single standalone file.
pub fn expand(body: &str, shared: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::with_capacity(body.len());
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let Some(open) = trimmed.find("{{") else {
            out.push_str(line);
            continue;
        };
        let indent = &trimmed[..open];
        let name = trimmed[open..]
            .trim()
            .strip_prefix("{{")
            .and_then(|m| m.strip_suffix("}}"))
            .map(str::trim)
            .filter(|n| {
                !n.is_empty()
                    && n.chars()
                        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            });
        let marker = indent.trim_end();
        let bare_indent = marker.is_empty()
            || marker.ends_with(['-', '*'])
            || marker
                .trim_start()
                .strip_suffix('.')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        let Some(name) = name.filter(|_| bare_indent) else {
            out.push_str(line);
            continue;
        };
        let block = shared
            .get(name)
            .ok_or_else(|| format!("unknown shared block: {name}"))?;
        let continuation = " ".repeat(indent.chars().count());
        for (i, block_line) in block.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if block_line.is_empty() {
                continue;
            }
            out.push_str(if i == 0 { indent } else { &continuation });
            out.push_str(block_line);
        }
        out.push_str(&line[trimmed.len()..]);
    }
    Ok(out)
}

fn collect_extras(dir: &Path, skip: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_extras(&path, skip, out)?;
        } else if path != skip {
            out.push(path);
        }
    }
    Ok(())
}

fn load_doc(path: &Path, slug: &str, shared: &BTreeMap<String, String>) -> Result<Doc, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (meta, body) = frontmatter::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let body = expand(&body, shared).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut extras = Vec::new();
    collect_extras(path.parent().unwrap_or(Path::new(".")), path, &mut extras)?;
    extras.sort();
    Ok(Doc {
        slug: slug.to_string(),
        meta,
        body,
        extras,
    })
}

pub fn load_dir(
    base: &Path,
    shared: &BTreeMap<String, String>,
    file: &str,
) -> Result<Vec<Doc>, String> {
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut dirs: Vec<PathBuf> = fs::read_dir(base)
        .map_err(|e| format!("{}: {e}", base.display()))?
        .map(|entry| {
            entry
                .map(|e| e.path())
                .map_err(|e| format!("{}: {e}", base.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    dirs.retain(|p| p.is_dir());
    dirs.sort();
    dirs.iter()
        .map(|d| {
            let slug = d
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            load_doc(&d.join(file), &slug, shared)
        })
        .collect()
}

/// Read every source document from the src/ tree.
pub fn load(src: &Path) -> Result<Sources, String> {
    let manifest = src.join("plugin.yaml");
    let raw = fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let plugin: Plugin = serde_yaml::from_str(&raw).map_err(|e| e.to_string())?;

    let mut shared = BTreeMap::new();
    let shared_dir = src.join(CONTENT).join("shared");
    if shared_dir.exists() {
        for entry in fs::read_dir(&shared_dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "md") {
                let stem = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                shared.insert(stem, text.trim().to_string());
            }
        }
    }

    Ok(Sources {
        plugin,
        skills: load_dir(&src.join(CONTENT).join("skills"), &shared, "SKILL.md")?,
        commands: load_dir(&src.join(CONTENT).join("commands"), &shared, "COMMAND.md")?,
        agents: load_dir(&src.join(CONTENT).join("agents"), &shared, "AGENT.md")?,
    })
}
