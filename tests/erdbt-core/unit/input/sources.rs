//! Unit tests for loading authored content and expanding shared blocks.

#[path = "../../common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::{doc, TempDir, PLUGIN_YAML};
use erdbt_core::sources::{expand, load, load_dir};
use serde_yaml::Value;

fn shared() -> BTreeMap<String, String> {
    BTreeMap::from([("gate".to_string(), "GATE TEXT".to_string())])
}

#[test]
fn expands_a_marker_on_its_own_line() {
    assert_eq!(
        expand("a\n{{ gate }}\nb\n", &shared()).unwrap(),
        "a\nGATE TEXT\nb\n"
    );
}

#[test]
fn expands_a_marker_on_the_final_line_without_newline() {
    assert_eq!(expand("{{ gate }}", &shared()).unwrap(), "GATE TEXT");
}

#[test]
fn preserves_list_indent() {
    assert_eq!(
        expand("- {{ gate }}\n", &shared()).unwrap(),
        "- GATE TEXT\n"
    );
    assert_eq!(
        expand("  {{ gate }}\n", &shared()).unwrap(),
        "  GATE TEXT\n"
    );
}

/// Prose that merely mentions a marker must survive verbatim, or the authoring
/// docs describing the include syntax would expand themselves.
#[test]
fn leaves_inline_prose_markers_alone() {
    let text = "use {{ gate }} to include it\n";
    assert_eq!(expand(text, &shared()).unwrap(), text);
}

#[test]
fn unknown_block_is_an_error() {
    let err = expand("{{ missing }}\n", &shared()).unwrap_err();
    assert!(
        err.contains("missing"),
        "error should name the block: {err}"
    );
}

#[test]
fn malformed_marker_is_left_alone() {
    for text in ["{{ }}\n", "{{ not valid! }}\n", "{{ unclosed\n"] {
        assert_eq!(expand(text, &shared()).unwrap(), text);
    }
}

#[test]
fn body_without_markers_is_unchanged() {
    assert_eq!(expand("plain\n", &shared()).unwrap(), "plain\n");
}

#[test]
fn description_is_trimmed_and_defaults_empty() {
    assert_eq!(
        doc("s", &[("description", "  x  ".into())], "").description(),
        "x"
    );
    assert_eq!(doc("s", &[], "").description(), "");
}

#[test]
fn tools_accepts_a_string_a_list_or_nothing() {
    assert_eq!(
        doc("s", &[("allowed-tools", "Read".into())], "")
            .tools()
            .unwrap(),
        "Read"
    );
    let list = Value::Sequence(vec!["Read".into(), "Bash".into()]);
    assert_eq!(
        doc("s", &[("tools", list)], "").tools().unwrap(),
        "Read, Bash"
    );
    assert!(doc("s", &[], "").tools().is_none());
    assert!(doc("s", &[("tools", Value::Bool(true))], "")
        .tools()
        .is_none());
}

#[test]
fn string_reads_an_optional_key() {
    assert_eq!(
        doc("s", &[("license", "MIT".into())], "")
            .string("license")
            .unwrap(),
        "MIT"
    );
    assert!(doc("s", &[], "").string("license").is_none());
}

#[test]
fn load_reads_a_whole_source_tree() {
    let dir = TempDir::new("sources-load");
    dir.write("plugin.yaml", PLUGIN_YAML);
    dir.write("content/shared/gate.md", "GATE TEXT\n");
    dir.write(
        "content/skills/alpha/SKILL.md",
        "---\ndescription: a\n---\n\n{{ gate }}\n",
    );
    dir.write("content/skills/alpha/extra.txt", "x");
    dir.write(
        "content/commands/beta/COMMAND.md",
        "---\ndescription: b\n---\n\nbody\n",
    );

    let src = load(dir.path()).unwrap();
    assert_eq!(src.plugin.name, "erdbt");
    assert_eq!(src.skills.len(), 1);
    assert_eq!(src.skills[0].slug, "alpha");
    assert_eq!(
        src.skills[0].body, "GATE TEXT\n",
        "shared block should be expanded at load"
    );
    assert_eq!(
        src.skills[0].extras.len(),
        1,
        "sibling files travel with the skill"
    );
    assert_eq!(src.commands[0].slug, "beta");
}

/// Skills and commands are loaded in directory-sorted order so a render is
/// reproducible regardless of filesystem enumeration order.
#[test]
fn docs_load_in_sorted_order() {
    let dir = TempDir::new("sources-order");
    dir.write("plugin.yaml", PLUGIN_YAML);
    for slug in ["zulu", "alpha", "mike"] {
        dir.write(
            &format!("content/skills/{slug}/SKILL.md"),
            "---\ndescription: d\n---\n\nb\n",
        );
    }
    let slugs: Vec<_> = load(dir.path())
        .unwrap()
        .skills
        .iter()
        .map(|d| d.slug.clone())
        .collect();
    assert_eq!(slugs, ["alpha", "mike", "zulu"]);
}

#[test]
fn optional_dirs_may_be_absent() {
    let dir = TempDir::new("sources-bare");
    dir.write("plugin.yaml", PLUGIN_YAML);
    let src = load(dir.path()).unwrap();
    assert!(src.skills.is_empty());
    assert!(src.commands.is_empty());
    assert!(
        load_dir(Path::new("/nonexistent/erdbt"), &shared(), "SKILL.md")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn missing_plugin_yaml_is_an_error() {
    assert!(load(Path::new("/nonexistent/erdbt")).is_err());
}

#[test]
fn malformed_plugin_yaml_is_an_error() {
    let dir = TempDir::new("sources-bad-yaml");
    dir.write("plugin.yaml", "name: erdbt\n");
    assert!(
        load(dir.path()).is_err(),
        "missing required manifest fields must fail"
    );
}

#[test]
fn an_unknown_shared_block_fails_the_load_with_its_path() {
    let dir = TempDir::new("sources-bad-block");
    dir.write("plugin.yaml", PLUGIN_YAML);
    dir.write(
        "content/skills/alpha/SKILL.md",
        "---\ndescription: a\n---\n\n{{ nope }}\n",
    );
    let err = load(dir.path()).unwrap_err();
    assert!(err.contains("nope"));
    assert!(
        err.contains("SKILL.md"),
        "error should name the offending file: {err}"
    );
}

/// plugin.yaml carries the one authored version, so `erdbt bump` has a single
/// line to raise and `claude plugin tag` has something to validate against.
#[test]
fn plugin_yaml_requires_a_version_field() {
    let dir = TempDir::new("sources-version");
    dir.write("plugin.yaml", PLUGIN_YAML);
    assert_eq!(load(dir.path()).unwrap().plugin.version, "0.1.0");

    dir.write("plugin.yaml", &PLUGIN_YAML.replace("version: 0.1.0\n", ""));
    assert!(
        load(dir.path()).is_err(),
        "a versionless plugin.yaml must fail"
    );
}

/// A shared block is usually several lines; indenting only the first would drop
/// the rest out of the list item or block quote that included it.
#[test]
fn expand_indents_every_line_of_a_multi_line_block() {
    let blocks = BTreeMap::from([("b".to_string(), "one\n\ntwo".to_string())]);
    assert_eq!(expand("- {{ b }}\n", &blocks).unwrap(), "- one\n\n  two\n");
    assert_eq!(expand("  {{ b }}\n", &blocks).unwrap(), "  one\n\n  two\n");
}

/// A numbered list item is as much a bare marker as a bulleted one; without it
/// a shared block inside a procedure silently ships as literal `{{ name }}`.
#[test]
fn expands_under_a_numbered_list_item() {
    assert_eq!(
        expand("1. {{ gate }}\n", &shared()).unwrap(),
        "1. GATE TEXT\n"
    );
}

/// Rewriting a CRLF line as LF would leave one file with mixed endings, which
/// `check` then reports as permanent drift on a Windows checkout.
#[test]
fn preserves_the_line_ending_it_found() {
    assert_eq!(
        expand("a\r\n{{ gate }}\r\nb\r\n", &shared()).unwrap(),
        "a\r\nGATE TEXT\r\nb\r\n"
    );
}
