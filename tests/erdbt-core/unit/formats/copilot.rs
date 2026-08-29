//! Unit tests for the GitHub Copilot CLI plugin format.
//!
//! These lean on the divergences from the Claude format. Where the two agree —
//! the banner, skill frontmatter, extras keeping their path — one emitter's
//! test already covers the shared behavior.

#[path = "../../common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::{doc, sources, TempDir};
use erdbt_core::formats::claude::{hooks, BANNER};
use erdbt_core::formats::copilot::{
    agent, command, emit, marketplace_json, plugin_json, readme, skill, CLAUDE_PLUGIN_ROOT,
    PLUGIN_ROOT,
};

#[test]
fn emits_the_expected_paths() {
    let out = emit(&sources(), Path::new("/nonexistent")).unwrap();
    for path in [
        "copilot-plugins/erdbt/plugin.json",
        "copilot-plugins/erdbt/skills/alpha/SKILL.md",
        "copilot-plugins/erdbt/commands/beta.md",
        "copilot-plugins/erdbt/agents/gamma.agent.md",
        "copilot-plugins/erdbt/README.md",
        ".github/plugin/marketplace.json",
    ] {
        assert!(out.contains_key(path), "missing {path}");
    }
}

#[test]
fn generated_markdown_carries_the_banner() {
    for (path, text) in emit(&sources(), Path::new("/nonexistent")).unwrap() {
        if path.ends_with(".md") {
            assert!(text.contains(BANNER), "{path} lacks the banner");
        }
    }
}

/// Copilot derives an agent's ID from its filename, and only `.agent.md` or
/// `.md` are read. The extension is the whole contract, so it is asserted.
#[test]
fn agents_ship_as_agent_md_files() {
    let out = emit(&sources(), Path::new("/nonexistent")).unwrap();
    assert!(out.contains_key("copilot-plugins/erdbt/agents/gamma.agent.md"));
    assert!(!out.contains_key("copilot-plugins/erdbt/agents/gamma.md"));
}

/// Copilot types the agent `tools` field as `string[]`. A comma-separated
/// string would read as one tool literally named "Read, Grep" — every listed
/// tool silently unavailable, with no error to say so.
#[test]
fn agent_emits_tools_as_a_yaml_list() {
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
    let rendered = agent(&authored).unwrap();
    assert!(rendered.contains("- Read"), "{rendered}");
    assert!(rendered.contains("- Grep"), "{rendered}");
    assert!(!rendered.contains("tools: Read, Grep"));
}

/// `model: opus` is a Claude alias that names no Copilot model. Copilot would
/// fall back to the session model anyway, so emitting it would be a line that
/// reads like a choice and is not one.
#[test]
fn agent_drops_the_claude_model_alias() {
    let authored = doc(
        "modeler",
        &[("description", "d".into()), ("model", "opus".into())],
        "b\n",
    );
    let rendered = agent(&authored).unwrap();
    assert!(rendered.contains("name: modeler"));
    assert!(!rendered.contains("model: "), "{rendered}");
}

#[test]
fn agent_keeps_the_fields_copilot_defines() {
    let authored = doc(
        "planner",
        &[
            ("description", "d".into()),
            ("infer", false.into()),
            ("reasoningEffort", "high".into()),
        ],
        "b\n",
    );
    let rendered = agent(&authored).unwrap();
    assert!(rendered.contains("infer: false"));
    assert!(rendered.contains("reasoningEffort: high"));
}

#[test]
fn skill_frontmatter_is_limited_to_the_spec() {
    let authored = doc(
        "alpha",
        &[
            ("description", "a".into()),
            ("license", "MIT".into()),
            ("internal-note", "x".into()),
        ],
        "b\n",
    );
    let rendered = skill(&authored).unwrap();
    assert!(rendered.contains("name: alpha"));
    assert!(!rendered.contains("internal-note"));
    assert!(!rendered.contains("license"));
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

/// The manifest's component paths are what make agents, skills, and commands
/// load at all; without them Copilot finds only its own defaults.
#[test]
fn plugin_json_declares_every_component_path() {
    let manifest: serde_json::Value =
        serde_json::from_str(&plugin_json(&sources(), false).unwrap()).unwrap();
    assert_eq!(manifest["name"], "erdbt");
    assert_eq!(manifest["agents"], "agents/");
    assert_eq!(manifest["skills"], "skills/");
    assert_eq!(manifest["commands"], "commands/");
}

#[test]
fn plugin_json_declares_hooks_only_when_they_ship() {
    let with: serde_json::Value =
        serde_json::from_str(&plugin_json(&sources(), true).unwrap()).unwrap();
    assert_eq!(with["hooks"], "hooks/hooks.json");
    let without: serde_json::Value =
        serde_json::from_str(&plugin_json(&sources(), false).unwrap()).unwrap();
    assert!(without["hooks"].is_null());
}

/// Copilot's marketplace schema differs from Claude's: `metadata.description`
/// rather than a top-level one, and an owner of `{name, email}`. A `source`
/// pointing at the Claude tree would install the wrong plugin directory.
#[test]
fn marketplace_json_matches_the_copilot_schema() {
    let market: serde_json::Value = serde_json::from_str(&marketplace_json(&sources()).unwrap())
        .expect("marketplace.json must be valid JSON");
    assert_eq!(market["name"], "datnguyex");
    assert_eq!(market["owner"]["name"], "n");
    assert!(market["metadata"]["description"].is_string());
    assert_eq!(market["plugins"][0]["source"], "./copilot-plugins/erdbt");
}

#[test]
fn every_manifest_reports_the_authored_version() {
    let src = sources();
    let plugin: serde_json::Value =
        serde_json::from_str(&plugin_json(&src, false).unwrap()).unwrap();
    assert_eq!(plugin["version"], src.plugin.version);
    let market: serde_json::Value = serde_json::from_str(&marketplace_json(&src).unwrap()).unwrap();
    assert_eq!(market["plugins"][0]["version"], src.plugin.version);
}

/// A hook is authored once with Claude's variable. Copilot expands
/// `${PLUGIN_ROOT}` only, so an unsubstituted command resolves to
/// `/hooks/ir-lint.sh` and the hook silently never runs.
#[test]
fn hook_commands_use_the_copilot_plugin_root() {
    let dir = TempDir::new("copilot-hook-root");
    dir.write("src/erdbt-core/content/hooks/hooks.json", "");
    let src_root = dir.path().join("src/erdbt-core");
    std::fs::write(
        src_root.join("content/hooks/hooks.json"),
        format!("bash \"{CLAUDE_PLUGIN_ROOT}/hooks/ir-lint.sh\"\n"),
    )
    .unwrap();
    let out = emit(&sources(), &src_root).unwrap();
    let rendered = &out["copilot-plugins/erdbt/hooks/hooks.json"];
    assert!(rendered.contains(PLUGIN_ROOT), "{rendered}");
    assert!(!rendered.contains(CLAUDE_PLUGIN_ROOT), "{rendered}");
}

/// The install snippet is the plugin's front door, and Copilot's commands are
/// not Claude's — `/plugin install` here would be the wrong instruction.
#[test]
fn readme_lists_commands_and_the_copilot_install_snippet() {
    let out = readme(&sources());
    assert!(out.contains("/beta"));
    assert!(out.contains("copilot plugin install erdbt@datnguyex"));
    assert!(out.contains("copilot plugin marketplace add datnguye/erdbt-harness"));
    assert!(out.contains("cargo install --path ../erdbt-harness/src/erdbt-core"));
}

#[test]
fn skill_extras_keep_their_path_beneath_the_skill() {
    let dir = TempDir::new("copilot-extras");
    let skill_dir = "content/skills/alpha";
    dir.write(&format!("{skill_dir}/references/notes.md"), "first\n");
    dir.write(&format!("{skill_dir}/examples/notes.md"), "second\n");
    let mut authored = doc("alpha", &[("description", "a".into())], "b\n");
    authored.extras = vec![
        dir.path().join(skill_dir).join("examples/notes.md"),
        dir.path().join(skill_dir).join("references/notes.md"),
    ];
    let src = erdbt_core::sources::Sources {
        plugin: common::plugin(),
        skills: vec![authored],
        commands: vec![],
        agents: vec![],
    };
    let out = emit(&src, dir.path()).unwrap();
    assert_eq!(
        out["copilot-plugins/erdbt/skills/alpha/references/notes.md"],
        "first\n"
    );
    assert_eq!(
        out["copilot-plugins/erdbt/skills/alpha/examples/notes.md"],
        "second\n"
    );
}

/// A command flattens to a single file, so a sibling has nowhere to land.
#[test]
fn a_command_with_a_sibling_file_is_refused() {
    let mut authored = doc("beta", &[("description", "d".into())], "b\n");
    authored.extras = vec![Path::new("/x/content/commands/beta/notes.md").to_path_buf()];
    let src = erdbt_core::sources::Sources {
        plugin: common::plugin(),
        skills: vec![],
        commands: vec![authored],
        agents: vec![],
    };
    let err = emit(&src, Path::new("/nonexistent")).unwrap_err();
    assert!(err.contains("would be dropped"), "{err}");
}

#[test]
fn hooks_are_copied_verbatim() {
    let dir = TempDir::new("copilot-emit-hooks");
    dir.write("content/hooks/x.sh", "#!/bin/sh\necho hi\n");
    let mut out = BTreeMap::new();
    hooks(dir.path(), "copilot-plugins/erdbt", &mut out).unwrap();
    assert_eq!(
        out["copilot-plugins/erdbt/hooks/x.sh"],
        "#!/bin/sh\necho hi\n"
    );
}
