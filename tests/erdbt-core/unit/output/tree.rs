//! Unit tests for writing, cleaning, and drift-checking the generated tree.

#[path = "../../common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::fs;

use common::TempDir;
use erdbt_core::tree::{clean, owned_dirs, stale, write};

fn rendered() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("plugins/erdbt/a.md".to_string(), "hello\n".to_string()),
        (
            "plugins/erdbt/skills/s/SKILL.md".to_string(),
            "skill\n".to_string(),
        ),
    ])
}

#[test]
fn write_creates_every_rendered_path() {
    let dir = TempDir::new("tree-write");
    write(&rendered(), dir.path()).unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("plugins/erdbt/a.md")).unwrap(),
        "hello\n"
    );
    assert!(dir.path().join("plugins/erdbt/skills/s/SKILL.md").exists());
}

#[test]
fn a_fresh_write_reports_no_drift() {
    let dir = TempDir::new("tree-fresh");
    write(&rendered(), dir.path()).unwrap();
    assert!(stale(&rendered(), dir.path()).unwrap().is_empty());
}

#[test]
fn stale_reports_a_missing_file() {
    let dir = TempDir::new("tree-missing");
    assert_eq!(
        stale(&rendered(), dir.path()).unwrap().len(),
        2,
        "nothing written yet, so all drift"
    );
}

#[test]
fn stale_reports_an_edited_file() {
    let dir = TempDir::new("tree-edit");
    write(&rendered(), dir.path()).unwrap();
    fs::write(dir.path().join("plugins/erdbt/a.md"), "tampered\n").unwrap();
    assert_eq!(
        stale(&rendered(), dir.path()).unwrap(),
        vec!["plugins/erdbt/a.md"]
    );
}

/// A renamed source would otherwise leave its old output behind forever, so an
/// unexpected file in an owned directory counts as drift.
#[test]
fn stale_reports_an_orphan() {
    let dir = TempDir::new("tree-orphan");
    write(&rendered(), dir.path()).unwrap();
    fs::write(dir.path().join("plugins/erdbt/orphan.md"), "stray\n").unwrap();
    assert_eq!(
        stale(&rendered(), dir.path()).unwrap(),
        vec!["plugins/erdbt/orphan.md"]
    );
}

/// Rewriting clears the owned directory first, so a renamed source cannot
/// leave an orphan behind.
#[test]
fn write_clears_orphans_from_owned_dirs() {
    let dir = TempDir::new("tree-clears");
    write(&rendered(), dir.path()).unwrap();
    fs::write(dir.path().join("plugins/erdbt/orphan.md"), "stray\n").unwrap();
    write(&rendered(), dir.path()).unwrap();
    assert!(!dir.path().join("plugins/erdbt/orphan.md").exists());
    assert!(stale(&rendered(), dir.path()).unwrap().is_empty());
}

#[test]
fn clean_removes_the_generated_tree() {
    let dir = TempDir::new("tree-clean");
    write(&rendered(), dir.path()).unwrap();
    assert_eq!(clean(dir.path()).unwrap(), 2);
    assert!(!dir.path().join("plugins").exists());
}

#[test]
fn clean_on_an_empty_tree_removes_nothing() {
    let dir = TempDir::new("tree-clean-empty");
    assert_eq!(clean(dir.path()).unwrap(), 0);
}

#[test]
fn owned_dirs_covers_only_dirs_the_render_writes_into() {
    assert_eq!(owned_dirs(&rendered()), vec!["plugins"]);
    assert!(owned_dirs(&BTreeMap::new()).is_empty());
}

/// The marketplace manifest lives outside any owned directory, so a render of
/// it alone must not authorise wiping the plugin tree.
#[test]
fn a_file_outside_owned_dirs_claims_none() {
    let only_marketplace = BTreeMap::from([(
        ".claude-plugin/marketplace.json".to_string(),
        "{}".to_string(),
    )]);
    assert!(owned_dirs(&only_marketplace).is_empty());
}

/// `clean` claims to delete generated output; a file emitted outside an owned
/// directory would otherwise survive and ship with no tree behind it.
#[test]
fn clean_removes_loose_generated_files() {
    let dir = TempDir::new("tree-loose");
    let mut out = rendered();
    out.insert(
        ".claude-plugin/marketplace.json".to_string(),
        "{}\n".to_string(),
    );
    write(&out, dir.path()).unwrap();
    assert!(dir.path().join(".claude-plugin/marketplace.json").exists());
    clean(dir.path()).unwrap();
    assert!(!dir.path().join(".claude-plugin/marketplace.json").exists());
}
