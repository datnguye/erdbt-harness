//! Unit tests for the Claude Code plugin format.

#[path = "../../common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::{doc, sources, TempDir};
use erdbt_core::formats::claude::{
    agent, command, emit, hooks, marketplace_json, plugin_json, readme, skill, BANNER,
};

#[test]
fn emits_the_expected_paths() {
    let out = emit(&sources(), Path::new("/nonexistent")).unwrap();
    for path in [
        "plugins/erdbt/.claude-plugin/plugin.json",
        "plugins/erdbt/skills/alpha/SKILL.md",
        "plugins/erdbt/commands/beta.md",
        "plugins/erdbt/README.md",
    ] {
        assert!(out.contains_key(path), "missing {path}");
    }
}

/// Every generated markdown file must announce itself, or someone edits the
/// render output and silently loses the change on the next render.
#[test]
fn generated_markdown_carries_the_banner() {
    for (path, text) in emit(&sources(), Path::new("/nonexistent")).unwrap() {
        if path.ends_with(".md") {
            assert!(text.contains(BANNER), "{path} lacks the banner");
        }
    }
}

/// The Agent Skills spec hard-errors on unknown keys, so a skill must not
/// inherit stray authoring frontmatter.
#[test]
fn skill_frontmatter_is_limited_to_the_spec() {
    let authored = doc(
        "alpha",
        &[("description", "a".into()), ("internal-note", "x".into())],
        "b\n",
    );
    let rendered = skill(&authored).unwrap();
    assert!(rendered.contains("name: alpha"));
    assert!(!rendered.contains("internal-note"));
}

#[test]
fn skill_includes_tools_when_present() {
    let authored = doc(
        "a",
        &[
            ("description", "d".into()),
            ("allowed-tools", "Read".into()),
        ],
        "b",
    );
    assert!(skill(&authored).unwrap().contains("allowed-tools: Read"));
}

/// `license` is not a permitted SKILL.md key — the Skills API and a claude.ai
/// upload both hard-error on it. Licensing belongs to plugin.json.
#[test]
fn skill_never_emits_a_license_key() {
    let authored = doc(
        "a",
        &[("description", "d".into()), ("license", "MIT".into())],
        "b",
    );
    assert!(!skill(&authored).unwrap().contains("license"));
}

#[test]
fn skill_omits_absent_optional_fields() {
    let rendered = skill(&doc("a", &[("description", "d".into())], "b")).unwrap();
    assert!(!rendered.contains("allowed-tools"));
}

#[test]
fn command_carries_description_and_hint() {
    let rendered = command(&doc(
        "b",
        &[("description", "d".into()), ("argument-hint", "<x>".into())],
        "body",
    ))
    .unwrap();
    assert!(rendered.contains("description: d"));
    assert!(rendered.contains("argument-hint: <x>"));
}

#[test]
fn manifests_are_valid_json_with_matching_identity() {
    let src = sources();
    let plugin: serde_json::Value = serde_json::from_str(&plugin_json(&src).unwrap()).unwrap();
    assert_eq!(plugin["name"], "erdbt");
    assert_eq!(plugin["author"]["email"], "e");

    let market: serde_json::Value = serde_json::from_str(&marketplace_json(&src).unwrap()).unwrap();
    assert_eq!(market["plugins"][0]["name"], "erdbt");
    assert_eq!(market["plugins"][0]["source"], "./plugins/erdbt");
}

/// The install snippet is the plugin's front door; a wrong name there sends
/// users to a plugin that does not exist.
#[test]
fn readme_lists_commands_and_the_install_snippet() {
    let out = readme(&sources());
    assert!(out.contains("/beta"));
    assert!(out.contains("/plugin install erdbt@datnguyex"));
    assert!(out.contains("/plugin marketplace add datnguye/erdbt-harness"));
    assert!(out.contains("cargo install --path ../erdbt-harness/src/erdbt-core"));
}

#[test]
fn hooks_are_copied_verbatim() {
    let dir = TempDir::new("emit-hooks");
    dir.write("content/hooks/x.sh", "#!/bin/sh\necho hi\n");
    let mut out = BTreeMap::new();
    hooks(dir.path(), "plugins/erdbt", &mut out).unwrap();
    assert_eq!(out["plugins/erdbt/hooks/x.sh"], "#!/bin/sh\necho hi\n");
}

#[test]
fn missing_hooks_dir_is_not_an_error() {
    let mut out = BTreeMap::new();
    hooks(Path::new("/nonexistent"), "plugins/erdbt", &mut out).unwrap();
    assert!(out.is_empty());
}

/// An extra keeps its path relative to the skill, not just its basename:
/// flattening would collide two `references/x.md` and `examples/x.md` into one
/// key, silently shipping whichever sorted last and dropping the other.
#[test]
fn skill_extras_keep_their_path_beneath_the_skill() {
    let dir = TempDir::new("emit-extras");
    let skill_dir = "content/skills/alpha";
    dir.write(&format!("{skill_dir}/ref.md"), "reference\n");
    dir.write(&format!("{skill_dir}/references/notes.md"), "first\n");
    dir.write(&format!("{skill_dir}/examples/notes.md"), "second\n");
    let mut authored = doc("alpha", &[("description", "a".into())], "b\n");
    authored.extras = vec![
        dir.path().join(skill_dir).join("examples/notes.md"),
        dir.path().join(skill_dir).join("ref.md"),
        dir.path().join(skill_dir).join("references/notes.md"),
    ];
    let src = erdbt_core::sources::Sources {
        plugin: common::plugin(),
        skills: vec![authored],
        commands: vec![],
        agents: vec![],
    };
    let out = emit(&src, dir.path()).unwrap();
    assert_eq!(out["plugins/erdbt/skills/alpha/ref.md"], "reference\n");
    assert_eq!(
        out["plugins/erdbt/skills/alpha/references/notes.md"],
        "first\n"
    );
    assert_eq!(
        out["plugins/erdbt/skills/alpha/examples/notes.md"],
        "second\n"
    );
}

/// Claude Code loads `hooks/hooks.json` on its own; declaring it again makes
/// the plugin fail to load with a duplicate-hooks-file error.
#[test]
fn plugin_json_never_declares_the_standard_hooks_file() {
    let manifest: serde_json::Value =
        serde_json::from_str(&plugin_json(&sources()).unwrap()).unwrap();
    assert!(manifest["hooks"].is_null());
}

/// `plugin.yaml` is the one authored version; every manifest that reports one
/// takes it from there, so `claude plugin tag` finds them in agreement.
#[test]
fn every_manifest_reports_the_authored_version() {
    let src = sources();
    let plugin: serde_json::Value = serde_json::from_str(&plugin_json(&src).unwrap()).unwrap();
    assert_eq!(plugin["version"], src.plugin.version);

    let market: serde_json::Value = serde_json::from_str(&marketplace_json(&src).unwrap()).unwrap();
    assert_eq!(market["plugins"][0]["version"], src.plugin.version);
    assert!(
        market["version"].is_null(),
        "version is not a valid top-level marketplace key"
    );
}

/// Plugin agents are refused hooks, mcpServers, and permissionMode, so
/// emitting one would author a setting that silently does nothing.
#[test]
fn agent_drops_fields_plugin_agents_may_not_declare() {
    let authored = doc(
        "modeler",
        &[
            ("description", "d".into()),
            ("model", "opus".into()),
            ("permissionMode", "bypassPermissions".into()),
            ("mcpServers", "slack".into()),
            ("hooks", "x".into()),
        ],
        "b\n",
    );
    let rendered = agent(&authored).unwrap();
    assert!(rendered.contains("name: modeler"));
    assert!(rendered.contains("model: opus"));
    for refused in ["permissionMode", "mcpServers", "hooks"] {
        assert!(
            !rendered.contains(refused),
            "{refused} leaked into an agent"
        );
    }
}

/// An agent restricted to read-only tools must stay restricted; `tools` is the
/// only thing standing between an advisory role and a writer.
#[test]
fn agent_emits_tools_as_a_comma_separated_string() {
    let authored = doc(
        "reviewer",
        &[
            ("description", "d".into()),
            (
                "tools",
                serde_yaml::Value::Sequence(vec!["Read".into(), "Grep".into()]),
            ),
        ],
        "b\n",
    );
    assert!(agent(&authored).unwrap().contains("tools: Read, Grep"));
}

#[test]
fn agents_land_in_the_plugin_agents_dir() {
    let out = emit(&sources(), Path::new("/nonexistent")).unwrap();
    assert!(out.contains_key("plugins/erdbt/agents/gamma.md"));
}

/// A command flattens to `commands/<slug>.md`, so a sibling file has nowhere to
/// land. Dropping it silently would ship a plugin missing a file the author
/// deliberately placed, so the render has to refuse instead.
#[test]
fn a_command_sibling_file_fails_the_render() {
    let dir = TempDir::new("emit-command-extras");
    let mut authored = doc("beta", &[("description", "b".into())], "body\n");
    authored.extras = vec![dir.path().join("content/commands/beta/helper.md")];
    let src = erdbt_core::sources::Sources {
        plugin: common::plugin(),
        skills: vec![],
        commands: vec![authored],
        agents: vec![],
    };
    let err = emit(&src, dir.path()).unwrap_err();
    assert!(err.contains("helper.md"), "{err}");
    assert!(err.contains("command"), "{err}");
}
